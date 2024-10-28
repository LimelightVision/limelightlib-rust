use limelightlib_rust::{LimelightClient, LimelightConfig};
use std::error::Error;
use tokio::time::Duration;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    
    //tracing_subscriber::fmt()
    //.with_env_filter("limelightlib_rust=debug")
    //.init();
    
    // Create a custom configuration
    let config = LimelightConfig {
        host: "192.168.1.181".to_string(),
        port: 5807,
        poll_interval_ms: 20,
    };

    let client = LimelightClient::new(config);
    let mut results = client.subscribe();

    println!("Starting Limelight client...");
    // Start returns a Result, so we need to handle it
    client.start().await.map_err(|e| Box::new(e) as Box<dyn Error>)?;

    println!("Waiting for results...");
    client.switch_pipeline(0).await?;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        match results.recv().await {
            Ok(result) => {
                println!("Valid target: {}", result.v.unwrap_or(0.0) > 0.0);
                if let Some(tx) = result.tx {
                    println!("Target X offset: {:.2}°", tx);
                }
                if let Some(ty) = result.ty {
                    println!("Target Y offset: {:.2}°", ty);
                }
                if let Some(txnc) = result.txnc {
                    println!("Target X offset (NC): {:.2}°", txnc);
                }
                if let Some(tync) = result.tync {
                    println!("Target Y offset (NC): {:.2}°", tync);
                }
                if let Some(ta) = result.ta {
                    println!("Target area: {:.2}%", ta);
                }

                if let Some(python_out) = &result.python_out {
                    println!("Python Output: {:?}", python_out);
                }
                if let Some(botpose) = &result.botpose {
                    println!("Botpose: {:?}", botpose);
                }
                if let Some(botposeMT2) = &result.botposeMT2 {
                    println!("BotposeMT2: {:?}", botposeMT2);
                }

                for br in &result.barcode {
                    if let (Some(data), Some(fam)) = (&br.data, &br.fam) {
                        println!("Barcode: Data: {}, Family: {}", data, fam);
                    }
                }

                // Classifier results
                for cr in &result.classifier {
                    if let (Some(class), Some(conf)) = (&cr.class, cr.conf) {
                        println!("Classifier: Class: {}, Confidence: {:.2}", class, conf);
                    }
                }

                // Detector results
                for dr in &result.detector {
                    if let (Some(class), Some(conf), Some(ta)) = (&dr.class, dr.conf, dr.ta) {
                        println!("Detector: Class: {},  Confidence: {:.2}, Area: {:.2}", class, conf,ta);
                    }
                }

                // Fiducial results
                for fr in &result.fiducial {
                    if let (Some(id), Some(fam)) = (&fr.f_id, &fr.fam) {
                        println!("Fiducial: ID: {}, Family: {}, X: {:.2}, Y: {:.2}", 
                            id, fam, fr.tx.unwrap_or(0.0), fr.ty.unwrap_or(0.0));
                    }
                }

                // Color results
                for cr in &result.retro {
                    println!("Color: X: {:.2}, Y: {:.2}", 
                        cr.tx.unwrap_or(0.0), cr.ty.unwrap_or(0.0));
                }


                println!("\n-------------------\n");
            }
            Err(e) => {
                eprintln!("Error receiving results: {:?}", e);
                break;
            }
        }
    }

    println!("Stopping client...");
    client.stop().await;

    Ok(())
}