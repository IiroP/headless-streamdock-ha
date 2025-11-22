use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub brightness: u8,
    pub timeout: u64,
    pub buttons: Vec<ButtonConfig>,
    pub knobs: Vec<KnobConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ButtonConfig {
    pub id: u8,
    pub domain: String,
    pub service: String,
    pub entity_id: String,
    pub icon: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KnobConfig {
    pub id: u8,
    pub domain: String,
    pub service: String,
    pub entity_id: String,
    pub key: String,
    pub step: i64,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_text = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_text)?;
    Ok(config)
}
