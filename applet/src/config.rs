use cosmic::cosmic_config::{Config, ConfigGet};
use serde::de::DeserializeOwned;

pub fn load_config<T>(key: &str, config_vers: u64) -> Option<T>
where
    T: DeserializeOwned,
{
    let config = match Config::new("co.uk.cappsy.CosmicAppletLogoMenu", config_vers) {
        Ok(config) => config,
        Err(_e) => Config::system("co.uk.cappsy.CosmicAppletLogoMenu", 1).unwrap(),
    };

    config.get(key).ok()
}
