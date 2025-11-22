use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub buttons: Vec<ButtonConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ButtonConfig {
    pub id: u8,
    pub domain: String,
    pub service: String,
    pub entity_id: String,
    pub icon: String,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_text = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_text)?;
    Ok(config)
}
