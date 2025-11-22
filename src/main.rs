use image::open;
use mirajazz::{
    device::{Device, DeviceQuery, list_devices},
    error::MirajazzError,
    state::DeviceStateUpdate,
    types::{ImageFormat, ImageMirroring, ImageMode, ImageRotation},
};
use std::collections::HashMap;
use tokio::signal::unix::{SignalKind, signal};

use crate::inputs::process_input;

mod config;
mod hass;
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

    let mut hass_client = hass::init_client().await;

    let config = config::load_config().expect("Failed to load config");
    let buttons_by_id: HashMap<u8, config::ButtonConfig> =
        config.buttons.iter().map(|b| (b.id, b.clone())).collect();
    let knobs_by_id: HashMap<u8, config::KnobConfig> =
        config.knobs.iter().map(|k| (k.id, k.clone())).collect();

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

        // Track brightness so we can dim on inactivity and restore on activity.
        let current_brightness: u8 = config.brightness;
        device.set_brightness(current_brightness).await?;
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
            use tokio::time::{Duration, timeout};

            // We dim after 10 seconds of inactivity.
            let idle_timeout = Duration::from_secs(config.timeout);
            let mut is_dimmed = false;

            loop {
                // Wait for events up to idle_timeout; on timeout, dim if not already dimmed.
                let updates = match timeout(idle_timeout, reader.read(None)).await {
                    Ok(Ok(updates)) => updates,
                    Ok(Err(_)) => break,
                    Err(_) => {
                        if !is_dimmed {
                            if let Err(e) = device.sleep().await {
                                println!("Failed to dim brightness: {e}");
                            } else {
                                is_dimmed = true;
                            }
                        }
                        continue;
                    }
                };

                // We got some updates: ensure brightness is restored if we were dimmed.
                if is_dimmed {
                    if let Err(e) = device.set_brightness(current_brightness).await {
                        println!("Failed to restore brightness: {e}");
                    } else {
                        is_dimmed = false;
                    }
                }

                // Event handler
                for update in updates {
                    match update {
                        DeviceStateUpdate::ButtonDown(i) => {
                            hass::handle_button(&mut hass_client, &buttons_by_id, i).await;
                        }
                        DeviceStateUpdate::EncoderTwist(i, value) => {
                            hass::handle_knob(&mut hass_client, &knobs_by_id, i, value).await;
                        }
                        _ => {}
                    }
                }
            }
        };

        // Ensure controlled exit
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
