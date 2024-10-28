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

    pub async fn update_robot_orientation(&self, yaw: f64) -> Result<bool, LimelightError> {
        let config = self.config.read().await;
        let url = format!("http://{}:{}/update-robotorientation", config.host, config.port);
        tracing::debug!("Updating robot orientation at URL: {}", url);
        tracing::debug!("Orientation parameters: yaw={}", yaw);
        
        let orientation_data = vec![yaw, 0.0, 0.0, 0.0, 0.0, 0.0];
        tracing::debug!("Full orientation data: {:?}", orientation_data);
        
        tracing::debug!("Sending POST request");
        match self.http_client
            .post(&url)
            .json(&orientation_data)
            .timeout(Duration::from_millis(100))
            .send()
            .await 
        {
            Ok(response) => {
                let status = response.status();
                tracing::debug!("Robot orientation update response status: {}", status);
                let success = status.is_success();
                tracing::debug!("Update {} (status code {})", 
                    if success { "succeeded" } else { "failed" }, 
                    status.as_u16());
                Ok(success)
            }
            Err(e) => {
                tracing::error!("Failed to update robot orientation: {:?}", e);
                Err(e.into())
            }
        }
    }

    pub async fn get_status(&self) -> Result<Value, LimelightError> {
        let config = self.config.read().await;
        let url = format!("http://{}:{}/status", config.host, config.port);
        
        tracing::debug!("Fetching status from {}", url);
        let response = self.http_client
            .get(&url)
            .timeout(Duration::from_millis(100))
            .send()
            .await?
            .json()
            .await?;
            
        Ok(response)
    }

    pub async fn reload_pipeline(&self) -> Result<bool, LimelightError> {
        let config = self.config.read().await;
        let url = format!("http://{}:{}/reload-pipeline", config.host, config.port);
        
        tracing::debug!("Reloading pipeline");
        let response = self.http_client
            .post(&url)
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    pub async fn switch_pipeline(&self, index: u32) -> Result<bool, LimelightError> {
        let config = self.config.read().await;
        let url = format!("http://{}:{}/pipeline-switch?index={}", config.host, config.port, index);
        
        tracing::debug!("Switching to pipeline index {}", index);
        let response = self.http_client
            .post(&url)
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    pub async fn capture_snapshot(&self, snapname: &str) -> Result<bool, LimelightError> {
        let config = self.config.read().await;
        let url = format!("http://{}:{}/capture-snapshot?snapname={}", config.host, config.port, snapname);
        
        tracing::debug!("Capturing snapshot with name: {}", snapname);
        let response = self.http_client
            .post(&url)
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    pub async fn delete_snapshots(&self) -> Result<bool, LimelightError> {
        let config = self.config.read().await;
        let url = format!("http://{}:{}/delete-snapshots", config.host, config.port);
        
        tracing::debug!("Deleting all snapshots");
        let response = self.http_client
            .delete(&url)
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    pub async fn delete_snapshot(&self, snapname: &str) -> Result<bool, LimelightError> {
        let config = self.config.read().await;
        let url = format!("http://{}:{}/delete-snapshot?snapname={}", config.host, config.port, snapname);
        
        tracing::debug!("Deleting snapshot: {}", snapname);
        let response = self.http_client
            .delete(&url)
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    pub async fn update_python_inputs(&self, inputs: &[f64]) -> Result<bool, LimelightError> {
        const MAX_INPUTS: usize = 32;
        if inputs.is_empty() || inputs.len() > MAX_INPUTS {
            return Err(LimelightError::ConfigError("Invalid number of Python inputs".into()));
        }

        let config = self.config.read().await;
        let url = format!("http://{}:{}/update-pythoninputs", config.host, config.port);
        
        tracing::debug!("Updating Python inputs with {} values", inputs.len());
        let response = self.http_client
            .post(&url)
            .json(inputs)
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    pub async fn upload_field_map(&self, field_map: Value, index: Option<u32>) -> Result<bool, LimelightError> {
        let config = self.config.read().await;
        let url = if let Some(idx) = index {
            format!("http://{}:{}/upload-fieldmap?index={}", config.host, config.port, idx)
        } else {
            format!("http://{}:{}/upload-fieldmap", config.host, config.port)
        };
        
        tracing::debug!("Uploading field map{}", if let Some(idx) = index { format!(" to index {}", idx) } else { "".to_string() });
        let response = self.http_client
            .post(&url)
            .json(&field_map)
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    pub async fn upload_python(&self, python_code: &str, index: Option<u32>) -> Result<bool, LimelightError> {
        let config = self.config.read().await;
        let url = if let Some(idx) = index {
            format!("http://{}:{}/upload-python?index={}", config.host, config.port, idx)
        } else {
            format!("http://{}:{}/upload-python", config.host, config.port)
        };
        
        tracing::debug!("Uploading Python code{}", if let Some(idx) = index { format!(" to index {}", idx) } else { "".to_string() });
        let response = self.http_client
            .post(&url)
            .body(python_code.to_string())
            .timeout(Duration::from_millis(100))
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }

    pub async fn get_calibration(&self, source: &str) -> Result<Value, LimelightError> {
        let config = self.config.read().await;
        let url = format!("http://{}:{}/cal-{}", config.host, config.port, source);
        
        tracing::debug!("Fetching calibration data from {}", source);
        let response = self.http_client
            .get(&url)
            .timeout(Duration::from_millis(100))
            .send()
            .await?
            .json()
            .await?;
            
        Ok(response)
    }

    pub async fn get_hardware_report(&self) -> Result<Value, LimelightError> {
        let config = self.config.read().await;
        let url = format!("http://{}:{}/hwreport", config.host, config.port);
        
        tracing::debug!("Fetching hardware report");
        let response = self.http_client
            .get(&url)
            .timeout(Duration::from_millis(100))
            .send()
            .await?
            .json()
            .await?;
            
        Ok(response)
    }

}