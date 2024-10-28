# LimelightLib-Rust

A Rust client library for async interfacing with Limelight smart cameras.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
limelightlib-rust = "0.1.0"
```

## Quick Start

```rust
use limelightlib_rust::{LimelightClient, LimelightConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with default configuration (host: "10.0.0.2", port: 5807)
    let client = LimelightClient::new(LimelightConfig::default());
    
    // Start the polling loop
    client.start().await?;
    
    // Subscribe to results
    let mut rx = client.subscribe();
    
    // Process incoming results
    while let Ok(result) = rx.recv().await {
        if let Some(tx) = result.tx {
            println!("Target X offset: {}", tx);
        }
    }
    
    Ok(())
}
```

## Configuration

```rust
let config = LimelightConfig {
    host: "10.0.0.2".to_string(),
    port: 5807,
    poll_interval_ms: 10,
};
let client = LimelightClient::new(config);
```

## Core Features

### Result Polling

The client automatically polls the Limelight device at configured intervals:

```rust
// Change polling rate
client.set_poll_rate(20).await?; // 20ms interval

// Get latest result without subscription
if let Some(result) = client.get_latest_result().await {
    println!("Latest target area: {:?}", result.ta);
}
```

### Vision Processing Results

The library provides structured types for different vision processing results:

```rust
// Example of processing different result types
if let Some(result) = client.get_latest_result().await {
    // Process AprilTag/Fiducial results
    for fiducial in result.fiducial {
        println!("AprilTag ID: {:?}", fiducial.f_id);
        println!("Position: tx={:?}, ty={:?}", fiducial.tx, fiducial.ty);
    }
    
    // Process detector results (neural network detection)
    for detection in result.detector {
        println!("Detected class: {:?}", detection.class);
        println!("Confidence: {:?}", detection.conf);
    }
    
    // Process classifier results
    for classification in result.classifier {
        println!("Classification: {:?}", classification.class);
        println!("Class ID: {:?}", classification.class_id);
    }
}
```

### Pipeline Management

Control and manage Limelight pipelines:

```rust
// Switch to a different pipeline
client.switch_pipeline(1).await?;

// Reload the current pipeline
client.reload_pipeline().await?;

// Upload new Python code to a pipeline
let python_code = r#"
def process(image):
    # Custom processing code
    return processed_image
"#;
client.upload_python(python_code, Some(0)).await?;
```

### Robot Pose Estimation

Access robot pose estimation data:

```rust
if let Some(result) = client.get_latest_result().await {
    // Get robot pose in field space
    if let Some(pose) = result.botpose {
        println!("Robot X: {}, Y: {}, Z: {}", pose[0], pose[1], pose[2]);
        println!("Robot Roll: {}, Pitch: {}, Yaw: {}", pose[3], pose[4], pose[5]);
    }
    
    // Get pose quality metrics
    println!("Tag Count: {:?}", result.botpose_tagcount);
    println!("Average Distance: {:?}", result.botpose_avgdist);
}
```

### Snapshot Management

Manage Limelight snapshots:

```rust
// Capture a snapshot
client.capture_snapshot("calibration_view").await?;

// Delete specific snapshot
client.delete_snapshot("calibration_view").await?;

// Delete all snapshots
client.delete_snapshots().await?;
```

### Hardware Information

Access device information and status:

```rust
// Get hardware status
let status = client.get_status().await?;
println!("Device Status: {:?}", status);

// Get hardware report
let report = client.get_hardware_report().await?;
println!("Hardware Report: {:?}", report);

// Get camera calibration
let calibration = client.get_calibration("camera").await?;
println!("Camera Calibration: {:?}", calibration);
```

## Error Handling

The library provides comprehensive error handling through the `LimelightError` enum:

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

## Advanced Usage

### Python Input Updates

Send custom inputs to Python SnapScript pipelines:

```rust
let inputs = vec![1.0, 2.0, 3.0];
client.update_python_inputs(&inputs).await?;
```

### Robot Orientation Updates

Update your robot orientation for better pose estimation with MegaTag2:

```rust
client.update_robot_orientation(45.0).await?; // 45 degree yaw
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built for the Limelight Vision Processing System
- Powered by Tokio async runtime
- Uses reqwest for HTTP communication

