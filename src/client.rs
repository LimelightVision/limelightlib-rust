use crate::{LimelightError, LimelightResult};
use reqwest::Client as HttpClient;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{Duration, interval, Interval};  // Added Interval to imports
use serde_json::{json, Value};

#[derive(Clone)]
pub struct LimelightConfig {
    pub host: String,
    pub port: u16,
    pub poll_interval_ms: u64,
}

impl Default for LimelightConfig {
    fn default() -> Self {
        tracing::debug!("Creating default LimelightConfig");
        Self {
            host: "10.0.0.2".to_string(),
            port: 5807,
            poll_interval_ms: 10,
        }
    }
}

pub struct LimelightClient {
    config: Arc<RwLock<LimelightConfig>>,
    http_client: HttpClient,
    latest_result: Arc<RwLock<Option<LimelightResult>>>,
    running: Arc<RwLock<bool>>,
    result_tx: broadcast::Sender<LimelightResult>,
}

impl LimelightClient {
    pub fn new(config: LimelightConfig) -> Self {
        tracing::debug!("Creating new LimelightClient with config: host={}, port={}, interval={}ms", 
            config.host, config.port, config.poll_interval_ms);
        let (result_tx, _) = broadcast::channel(100);
        tracing::debug!("Created broadcast channel with capacity 100");
        Self {
            config: Arc::new(RwLock::new(config)),
            http_client: HttpClient::new(),
            latest_result: Arc::new(RwLock::new(None)),
            running: Arc::new(RwLock::new(false)),
            result_tx,
        }
    }

    pub async fn get_poll_rate(&self) -> u64 {
        self.config.read().await.poll_interval_ms
    }

    pub async fn set_poll_rate(&self, interval_ms: u64) -> Result<(), LimelightError> {
        if interval_ms == 0 {
            return Err(LimelightError::ConfigError("Poll interval cannot be zero".into()));
        }
        
        tracing::debug!("Setting new poll rate to {}ms", interval_ms);
        let mut config = self.config.write().await;
        config.poll_interval_ms = interval_ms;
        
        if *self.running.read().await {
            tracing::debug!("Client is running, restarting to apply new poll rate");
            drop(config);
            self.stop().await;
            self.start().await?;
        }
        
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LimelightResult> {
        tracing::debug!("New subscriber added to broadcast channel");
        self.result_tx.subscribe()
    }

    pub async fn start(&self) -> Result<(), LimelightError> {
        tracing::debug!("Attempting to start LimelightClient");
        let mut running = self.running.write().await;
        if *running {
            tracing::debug!("Client already running, ignoring start request");
            return Ok(());
        }
        tracing::debug!("Setting running state to true");
        *running = true;
        
        let config = self.config.clone();
        let http_client = self.http_client.clone();
        let latest_result = self.latest_result.clone();
        let result_tx = self.result_tx.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            tracing::debug!("Spawned polling task");
            let config_read = config.read().await;
            let mut interval_timer = interval(Duration::from_millis(config_read.poll_interval_ms));
            let base_url = format!("http://{}:{}", config_read.host, config_read.port);
            tracing::debug!("Starting polling loop with URL: {}, interval: {}ms", 
                base_url, config_read.poll_interval_ms);
            drop(config_read);

            let mut last_interval_ms = 0;
            let mut iteration = 0u64;
            while *running.read().await {
                iteration += 1;
                tracing::debug!("Poll iteration {}", iteration);
                interval_timer.tick().await;

                // Only recreate the interval if the poll rate has changed
                let current_config = config.read().await;
                if current_config.poll_interval_ms != last_interval_ms {
                    tracing::debug!("Poll rate changed from {}ms to {}ms", last_interval_ms, current_config.poll_interval_ms);
                    interval_timer = interval(Duration::from_millis(current_config.poll_interval_ms));
                    last_interval_ms = current_config.poll_interval_ms;
                }
                let base_url = format!("http://{}:{}", current_config.host, current_config.port);
                drop(current_config);

                match Self::fetch_results(&http_client, &base_url).await {
                    Ok(result) => {
                        tracing::debug!("Successfully fetched results on iteration {}", iteration);
                        tracing::trace!("Result details: {:?}", result);
                        
                        tracing::debug!("Updating latest_result");
                        *latest_result.write().await = Some(result.clone());
                        
                        tracing::debug!("Broadcasting result to {} receivers", result_tx.receiver_count());
                        if let Err(e) = result_tx.send(result) {
                            tracing::error!("Error broadcasting result on iteration {}: {:?}", iteration, e);
                        } else {
                            tracing::debug!("Successfully broadcast result");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error fetching results on iteration {}: {:?}", iteration, e);
                    }
                }
            }
            tracing::debug!("Polling loop stopped after {} iterations", iteration);
        });

        tracing::debug!("Client started successfully");
        Ok(())
    }

    pub async fn stop(&self) {
        tracing::debug!("Attempting to stop LimelightClient");
        let mut running = self.running.write().await;
        *running = false;
        tracing::debug!("Client stopped, running state set to false");
    }

    async fn build_url(&self, endpoint: &str) -> String {
        let config = self.config.read().await;
        format!("http://{}:{}/{}", config.host, config.port, endpoint)
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T, LimelightError> {
        let url = self.build_url(endpoint).await;
        tracing::debug!("GET request to {}", url);
        
        let response = self.http_client
            .get(&url)
            .timeout(Duration::from_millis(100))
            .send()
            .await?
            .json()
            .await?;
            
        Ok(response)
    }

    async fn post_json<T: serde::Serialize + ?Sized>(
        &self, 
        endpoint: &str, 
        data: &T,
    ) -> Result<bool, LimelightError> {
        let url = self.build_url(endpoint).await;
        tracing::debug!("POST request to {}", url);
        
        let response = self.http_client
            .post(&url)
            .json(data)
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    async fn delete(&self, endpoint: &str) -> Result<bool, LimelightError> {
        let url = self.build_url(endpoint).await;
        tracing::debug!("DELETE request to {}", url);
        
        let response = self.http_client
            .delete(&url)
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    pub async fn get_latest_result(&self) -> Option<LimelightResult> {
        tracing::debug!("Getting latest result");
        let result = self.latest_result.read().await.clone();
        match &result {
            Some(_) => tracing::debug!("Returning cached result"),
            None => tracing::debug!("No cached result available"),
        }
        result
    }

    async fn fetch_results(client: &HttpClient, base_url: &str) -> Result<LimelightResult, LimelightError> {
        let url = format!("{}/results", base_url);
        tracing::debug!("Fetching results from: {}", url);

        tracing::debug!("Sending HTTP GET request");
        let response = client
            .get(&url)
            .timeout(Duration::from_millis(100))
            .send()
            .await;
        
        match response {
            Ok(resp) => {
                let status = resp.status();
                tracing::debug!("Got HTTP response with status: {}", status);
                
                tracing::debug!("Reading response body");
                let text = resp.text().await?;
                tracing::debug!("Raw JSON response (length={}): {}", text.len(), text);
                
                tracing::debug!("Attempting to parse JSON");
                match serde_json::from_str::<LimelightResult>(&text) {
                    Ok(result) => {
                        tracing::debug!("Successfully parsed JSON response");
                        tracing::trace!("Parsed result: {:?}", result);
                        Ok(result)
                    }
                    Err(e) => {
                        tracing::error!("JSON parsing error: {:?}", e);
                        tracing::error!("Failed JSON content: {}", text);
                        Err(LimelightError::JsonError(e))
                    }
                }
            }
            Err(e) => {
                tracing::error!("HTTP request failed: {:?}", e);
                Err(e.into())
            }
        }
    }

    pub async fn get_status(&self) -> Result<Value, LimelightError> {
        self.get_json("status").await
    }

    pub async fn reload_pipeline(&self) -> Result<bool, LimelightError> {
        self.post_json("reload-pipeline", &()).await
    }

    pub async fn switch_pipeline(&self, index: u32) -> Result<bool, LimelightError> {
        self.post_json(&format!("pipeline-switch?index={}", index), &()).await
    }

    pub async fn capture_snapshot(&self, snapname: &str) -> Result<bool, LimelightError> {
        self.post_json(&format!("capture-snapshot?snapname={}", snapname), &()).await
    }

    pub async fn delete_snapshots(&self) -> Result<bool, LimelightError> {
        self.delete("delete-snapshots").await
    }

    pub async fn delete_snapshot(&self, snapname: &str) -> Result<bool, LimelightError> {
        self.delete(&format!("delete-snapshot?snapname={}", snapname)).await
    }

    pub async fn update_python_inputs(&self, inputs: &[f64]) -> Result<bool, LimelightError> {
        const MAX_INPUTS: usize = 32;
        if inputs.is_empty() || inputs.len() > MAX_INPUTS {
            return Err(LimelightError::ConfigError("Invalid number of Python inputs".into()));
        }
        self.post_json("update-pythoninputs", inputs).await
    }

    pub async fn update_robot_orientation(&self, yaw: f64) -> Result<bool, LimelightError> {
        let orientation_data = vec![yaw, 0.0, 0.0, 0.0, 0.0, 0.0];
        self.post_json("update-robotorientation", &orientation_data).await
    }

    pub async fn upload_field_map(&self, field_map: Value, index: Option<u32>) -> Result<bool, LimelightError> {
        let endpoint = match index {
            Some(idx) => format!("upload-fieldmap?index={}", idx),
            None => "upload-fieldmap".to_string(),
        };
        self.post_json(&endpoint, &field_map).await
    }

    pub async fn get_calibration(&self, source: &str) -> Result<Value, LimelightError> {
        self.get_json(&format!("cal-{}", source)).await
    }

    pub async fn get_hardware_report(&self) -> Result<Value, LimelightError> {
        self.get_json("hwreport").await
    }

    // Pipeline Management
    pub async fn get_default_pipeline(&self) -> Result<Value, LimelightError> {
        self.get_json("pipeline-default").await
    }

    pub async fn get_pipeline_at_index(&self, index: u32) -> Result<Value, LimelightError> {
        self.get_json(&format!("pipeline-atindex?index={}", index)).await
    }

    pub async fn update_pipeline(&self, settings: Value, flush: bool) -> Result<bool, LimelightError> {
        self.post_json(&format!("update-pipeline?flush={}", if flush { 1 } else { 0 }), &settings).await
    }

    pub async fn upload_pipeline(&self, pipeline: Value, index: Option<u32>) -> Result<bool, LimelightError> {
        let endpoint = match index {
            Some(idx) => format!("upload-pipeline?index={}", idx),
            None => "upload-pipeline".to_string(),
        };
        self.post_json(&endpoint, &pipeline).await
    }

    // Neural Network Management
    pub async fn upload_neural_network(&self, nn_type: &str, data: &[u8], index: Option<u32>) -> Result<bool, LimelightError> {
        if !["detector", "classifier"].contains(&nn_type) {
            return Err(LimelightError::ConfigError("Invalid neural network type".into()));
        }
        let endpoint = match index {
            Some(idx) => format!("upload-nn?type={}&index={}", nn_type, idx),
            None => format!("upload-nn?type={}", nn_type),
        };
        let url = self.build_url(&endpoint).await;
        
        let response = self.http_client
            .post(&url)
            .body(data.to_vec())
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    pub async fn upload_neural_network_labels(&self, nn_type: &str, labels: &str, index: Option<u32>) -> Result<bool, LimelightError> {
        if !["detector", "classifier"].contains(&nn_type) {
            return Err(LimelightError::ConfigError("Invalid neural network type".into()));
        }
        let endpoint = match index {
            Some(idx) => format!("upload-nnlabels?type={}&index={}", nn_type, idx),
            None => format!("upload-nnlabels?type={}", nn_type),
        };
        let url = self.build_url(&endpoint).await;
        
        let response = self.http_client
            .post(&url)
            .body(labels.to_string())
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    // SnapScript Management
    pub async fn get_snapscript_names(&self) -> Result<Vec<String>, LimelightError> {
        self.get_json("getsnapsscriptnames").await
    }

    // Calibration Management
    pub async fn get_calibration_default(&self) -> Result<Value, LimelightError> {
        self.get_json("cal-default").await
    }

    pub async fn get_calibration_file(&self) -> Result<Value, LimelightError> {
        self.get_json("cal-file").await
    }

    pub async fn get_calibration_eeprom(&self) -> Result<Value, LimelightError> {
        self.get_json("cal-eeprom").await
    }

    pub async fn get_calibration_latest(&self) -> Result<Value, LimelightError> {
        self.get_json("cal-latest").await
    }

    pub async fn update_calibration_eeprom(&self, calibration: Value) -> Result<bool, LimelightError> {
        self.post_json("cal-eeprom", &calibration).await
    }

    pub async fn update_calibration_file(&self, calibration: Value) -> Result<bool, LimelightError> {
        self.post_json("cal-file", &calibration).await
    }

    pub async fn delete_calibration_latest(&self) -> Result<bool, LimelightError> {
        self.delete("cal-latest").await
    }

    pub async fn delete_calibration_eeprom(&self) -> Result<bool, LimelightError> {
        self.delete("cal-eeprom").await
    }

    pub async fn delete_calibration_file(&self) -> Result<bool, LimelightError> {
        self.delete("cal-file").await
    }

    // Snapshot Management
    pub async fn upload_snapshot(&self, snapname: &str, image_data: &[u8]) -> Result<bool, LimelightError> {
        let url = self.build_url(&format!("upload-snapshot?snapname={}", snapname)).await;
        
        let response = self.http_client
            .post(&url)
            .body(image_data.to_vec())
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    pub async fn get_snapshot_manifest(&self) -> Result<Vec<String>, LimelightError> {
        self.get_json("snapshotmanifest").await
    }
}