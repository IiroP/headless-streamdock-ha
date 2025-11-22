use dotenv::dotenv;
use hass_rs::client::HassClient;
use serde_json::json;
use std::{collections::HashMap, env::var};

use crate::config;

pub async fn init_client() -> HassClient {
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
    hass_client
}

pub async fn handle_button(
    client: &mut HassClient,
    buttons: &HashMap<u8, config::ButtonConfig>,
    i: u8,
) {
    let btn = buttons.get(&i);
    if let Some(b) = btn {
        client
            .call_service(
                b.domain.clone(),
                b.service.clone(),
                Some(json!({"entity_id": b.entity_id.clone()})),
            )
            .await
            .unwrap_or_else(|_| println!("Failed to trigger action for button {i}"));
    } else {
        println!("No config for button {i}")
    }
}

pub async fn handle_knob(
    client: &mut HassClient,
    knobs: &HashMap<u8, config::KnobConfig>,
    i: u8,
    value: i8,
) {
    let knob = knobs.get(&i);
    if let Some(k) = knob {
        if value > 0 {
            client
                .call_service(
                    k.domain.clone(),
                    k.service.clone(),
                    Some(json!({"entity_id": k.entity_id.clone(), k.key.clone(): k.step})),
                )
                .await
                .expect("Unable to increase value");
        } else {
            client
                .call_service(
                    k.domain.clone(),
                    k.service.clone(),
                    Some(json!({"entity_id": k.entity_id.clone(), k.key.clone(): -k.step})),
                )
                .await
                .expect("Unable to decrease value");
        }
    } else {
        println!("No config for knob {i}")
    }
}
