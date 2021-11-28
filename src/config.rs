use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub devices: BTreeMap<String, Device>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Device {
    pub description: String,
    pub modbus_address: u16,
    pub coils: Vec<Coil>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Coil {
    pub address: u16,
    pub description: String,
    pub default_status: ResetCoilStatus,
}

/// Value to which a coil should be set
/// it the coil/the device/the bus is
/// resettet.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ResetCoilStatus {
    On,
    Off,
    DoNotSet,
}

impl Default for ResetCoilStatus {
    fn default() -> Self {
        ResetCoilStatus::DoNotSet
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[test]
    fn parse_default_config() {
        let _config: Config = toml::from_str(include_str!("../example-config.toml")).unwrap();
    }
}
