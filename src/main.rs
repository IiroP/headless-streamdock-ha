use dotenv::dotenv;
use hass_rs::client::HassClient;
use image::open;
use mirajazz::{
    device::{Device, DeviceQuery, list_devices},
    error::MirajazzError,
    state::DeviceStateUpdate,
    types::{ImageFormat, ImageMirroring, ImageMode, ImageRotation},
};
use serde_json::json;
use std::{collections::HashMap, env::var};
use tokio::signal::unix::{SignalKind, signal};

use crate::inputs::process_input;

mod config;
mod inputs;

const QUERY: DeviceQuery = DeviceQuery::new(65440, 1, 0x6603, 0x1003);

const IMAGE_FORMAT: ImageFormat = ImageFormat {
    mode: ImageMode::JPEG,
    size: (60, 60),
    rotation: ImageRotation::Rot90,
    mirror: ImageMirroring::None,
};

#[tokio::main]
async fn main() -> Result<(), MirajazzError> {
    // SIGINT = Ctrl+C
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    // SIGTERM = kill or systemd stop
    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    dotenv().ok();
    let ha_url: String = var("HA_URL").expect("HA_URL env variable is missing");
    let ha_token: String = var("HA_TOKEN").expect("HA_TOKEN env variable is missing");
    let mut hass_client = HassClient::new(&ha_url)
        .await
        .expect("Failed to connect to HA");

    hass_client
        .auth_with_longlivedtoken(&ha_token)
        .await
        .expect("Failed to authenticate with HA");

    let config = config::load_config().expect("Failed to load config");
    let buttons_by_id: HashMap<u8, config::ButtonConfig> =
        config.buttons.iter().map(|b| (b.id, b.clone())).collect();

    for dev in list_devices(&[QUERY]).await? {
        println!(
            "Connecting to {:04X}:{:04X}, {}",
            dev.vendor_id,
            dev.product_id,
            dev.serial_number.clone().unwrap()
        );

        // Connect to the device
        let device = Device::connect(&dev, 3, 9, 3).await?;

        // Print out some info from the device
        println!("Connected to '{}'", device.serial_number());

        device.set_brightness(50).await?;
        device.clear_all_button_images().await?;

        println!("Key count: {}", device.key_count());
        // Write it to the device
        for b in &config.buttons {
            let image = open(format!("examples/{}", b.icon))
                .expect(format!("Failed to open image {}", b.icon).as_str());
            device
                .set_button_image(b.id, IMAGE_FORMAT, image.clone())
                .await?;
        }

        // Flush
        device.flush().await?;

        let reader = device.get_reader(process_input);

        let main_loop = async {
            loop {
                let updates = match reader.read(None).await {
                    Ok(updates) => updates,
                    Err(_) => break,
                };

                for update in updates {
                    //println!("Received input: {:?}", update);
                    match update {
                        DeviceStateUpdate::ButtonDown(i) => {
                            let btn = buttons_by_id.get(&i);
                            if let Some(b) = btn {
                                hass_client
                                    .call_service(
                                        b.domain.clone(),
                                        b.service.clone(),
                                        Some(json!({"entity_id": b.entity_id.clone()})),
                                    )
                                    .await
                                    .unwrap_or_else(|_| {
                                        println!("Failed to trigger action for button {i}")
                                    });
                            } else {
                                println!("No config for button {i}")
                            }
                        }
                        DeviceStateUpdate::EncoderTwist(value, value2) => {
                            if value2 > 0 {
                                hass_client
                                .call_service(
                                    "homeassistant".to_string(),
                                    "turn_on".to_string(),
                                    Some(
                                        json!({"entity_id": "light.valot", "brightness_step": 10}),
                                    ),
                                )
                                .await
                                .expect("Unable to increase brightness");
                            } else {
                                hass_client
                                .call_service(
                                    "homeassistant".to_string(),
                                    "turn_on".to_string(),
                                    Some(
                                        json!({"entity_id": "light.valot", "brightness_step": -10}),
                                    ),
                                )
                                .await
                                .expect("Unable to decrease brightness");
                            }
                        }
                        _ => {}
                    }
                }
            }
        };

        tokio::select! {
            _ = main_loop => {},
            _ = sigint.recv() => {
                println!("Received SIGINT")
            },
            _ = sigterm.recv() => {
                println!("Received SIGTERM")
            }
        }

        println!("Exiting...");

        drop(reader);

        device.flush().await?;
        device.shutdown().await?;
    }

    Ok(())
}
