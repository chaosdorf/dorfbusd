use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(JsonSchema))]
pub struct Config {
    pub devices: BTreeMap<String, DeviceConfig>,
    pub coils: BTreeMap<String, CoilConfig>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(JsonSchema))]
pub struct DeviceConfig {
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Address of the modbus device
    pub modbus_address: u8,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(JsonSchema))]
pub struct CoilConfig {
    /// Name of the relais card
    pub device: String,
    /// Address of the coil
    pub address: u16,
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub default_status: ResetCoilStatus,
    #[serde(default)]
    pub tags: BTreeSet<String>,
}

/// Value to which a coil should be set if the coil/the device/the bus is resetted.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, derive(JsonSchema))]
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
