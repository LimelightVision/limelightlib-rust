# LimelightLib-Rust

A Rust client library for async interfacing with Limelight smart cameras.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
limelightlib-rust = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use limelightlib_rust::{LimelightClient, LimelightConfig};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with custom configuration
    let config = LimelightConfig {
        host: "10.0.0.2".to_string(),
        port: 5807,
        poll_interval_ms: 20,
    };
    
    let client = LimelightClient::new(config);
    client.start().await?;
    
    // Subscribe to vision processing results
    let mut results = client.subscribe();
    
    while let Ok(result) = results.recv().await {
        // Process standard targeting results
        if let Some(tx) = result.tx {
            println!("Target X offset: {:.2}°", tx);
        }
        
        // Process AprilTag results
        for tag in result.fiducial {
            if let Some(id) = tag.f_id {
                println!("Detected AprilTag ID: {}", id);
            }
        }

        // Process MegaTag2 results
        if let Some(botpose) = &result.botposeMT2 {
            println!("MegaTag2 Robot Pose: {:?}", botpose);
        }
    }
    
    Ok(())
}
```

## Core Features Guide

### Vision Processing Results

Access comprehensive vision processing results:

```rust
if let Some(result) = client.get_latest_result().await {
    // Basic target information
    if result.v.unwrap_or(0.0) > 0.0 {
        println!("Valid target found!");
        println!("X Offset: {:?}°", result.tx);
        println!("Y Offset: {:?}°", result.ty);
        println!("Target Area: {:?}%", result.ta);
        
        // Non-cross-filtered results
        println!("X Offset (No Cross): {:?}°", result.txnc);
        println!("Y Offset (No Cross): {:?}°", result.tync);
    }
    
    // Process barcode results
    for barcode in &result.barcode {
        println!("Barcode: {:?} (Family: {:?})", barcode.data, barcode.fam);
    }
    
    // Process classifier results
    for class in &result.classifier {
        println!("Class: {:?} (ID: {:?})", class.class, class.class_id);
    }
    
    // Process detector results
    for detection in &result.detector {
        println!("Detection: {:?} (Conf: {:?})", detection.class, detection.conf);
    }
}
```

### Pipeline Management

Complete pipeline control and configuration:

```rust
// Basic pipeline operations
client.switch_pipeline(1).await?;
client.reload_pipeline().await?;

// Get pipeline configurations
let default_pipeline = client.get_default_pipeline().await?;
let specific_pipeline = client.get_pipeline_at_index(0).await?;

// Update pipeline settings
let settings = serde_json::json!({
    "type": "apriltag",
    "parameters": { /* ... */ }
});
client.update_pipeline(settings, true).await?; // true to flush changes

// Upload complete pipeline
let pipeline = serde_json::json!({ /* pipeline config */ });
client.upload_pipeline(pipeline, Some(0)).await?;
```

### Neural Network Management

Upload and configure custom neural networks:

```rust
// Upload detector network
let detector_data = std::fs::read("detector.tflite")?;
client.upload_neural_network("detector", &detector_data, Some(0)).await?;

// Upload classifier network
let classifier_data = std::fs::read("classifier.tflite")?;
client.upload_neural_network("classifier", &classifier_data, Some(0)).await?;

// Upload network labels
let labels = "class1\nclass2\nclass3";
client.upload_neural_network_labels("detector", labels, Some(0)).await?;
```

### Camera Calibration

Comprehensive calibration management:

```rust
// Get calibration data
let default_cal = client.get_calibration_default().await?;
let file_cal = client.get_calibration_file().await?;
let eeprom_cal = client.get_calibration_eeprom().await?;
let latest_cal = client.get_calibration_latest().await?;

// Update calibration
let calibration = serde_json::json!({ /* calibration data */ });
client.update_calibration_file(calibration).await?;
client.update_calibration_eeprom(calibration).await?;

// Delete calibration data
client.delete_calibration_latest().await?;
client.delete_calibration_file().await?;
client.delete_calibration_eeprom().await?;
```



### Hardware Management

Access device information and status:

```rust
// Get device status and reports
let status = client.get_status().await?;
let hardware_report = client.get_hardware_report().await?;
```

### Robot Pose Estimation

Comprehensive pose estimation support:

```rust
if let Some(result) = client.get_latest_result().await {
    // Standard pose estimation
    if let Some(pose) = &result.botpose {
        println!("Robot Pose: {:?}", pose);
    }
    
    // IMU-Fused MegaTag2 pose estimation (requires robot orientation updates)
    if let Some(pose_mt2) = &result.botposeMT2 {
        println!("MegaTag2 Pose: {:?}", pose_mt2);
    }
    
    // WPI field space poses (blue/red alliance)
    if let Some(pose_blue) = &result.botpose_wpiblue {
        println!("WPI Blue Alliance Pose: {:?}", pose_blue);
    }
    
    // Pose quality metrics
    println!("Tag Count: {:?}", result.botpose_tagcount);
    println!("Pose Span: {:?}", result.botpose_span);
    println!("Average Distance: {:?}", result.botpose_avgdist);
    println!("Average Area: {:?}", result.botpose_avgarea);
}

// Update robot orientation
client.update_robot_orientation(45.0).await?;
```

### SnapScript Integration

Manage Python processing pipelines:

```rust
// Get available SnapScript names
let scripts = client.get_snapscript_names().await?;

// Update Python inputs
let inputs = vec![1.0, 2.0, 3.0];
client.update_python_inputs(&inputs).await?;

// Get Python outputs from results
if let Some(result) = client.get_latest_result().await {
    if let Some(outputs) = &result.python_out {
        println!("Python outputs: {:?}", outputs);
    }
}
```

### Snapshot Management

Comprehensive snapshot control:

```rust
// Capture and manage snapshots
client.capture_snapshot("calibration").await?;
client.upload_snapshot("custom_image", &image_data).await?;
let snapshot_list = client.get_snapshot_manifest().await?;
client.delete_snapshot("old_image").await?;
client.delete_snapshots().await?; // Delete all
```

## Advanced Configuration

### Custom Poll Rate

```rust
// Change polling rate to 50ms
client.set_poll_rate(50).await?;

// Get current poll rate
let current_rate = client.get_poll_rate().await;
```

### Error Handling

```rust
pub enum LimelightError {
    HttpError(reqwest::Error),
    WebSocketError(tokio_tungstenite::tungstenite::Error),
    JsonError(serde_json::Error),
    UrlError(url::ParseError),
    ConfigError(String),
    TimeoutError,
    NotRunning,
}
```

## Logging

Enable debug logging:

```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_env_filter("limelightlib_rust=debug")
    .init();
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.

## See Also

- [Limelight Documentation](https://docs.limelightvision.io/)
- [Limelight Vision](https://limelightvision.io/)
