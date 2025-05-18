use cosmic::cosmic_config::{Config, ConfigGet, ConfigSet};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Display;

pub fn update_config<T>(config: Config, key: &str, value: T)
where
    T: Serialize + Display + Clone,
{
    let _config_set = config.set(key, value.clone());
    let config_tx = config.transaction();
    let _tx_result = config_tx.commit();
}

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
