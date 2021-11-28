use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Device {
    pub name: String,
    pub description: String,
    pub coils: Vec<Coil>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coil {
    pub address: u16,
    pub description: String,
}

/// Value to which a coil should be set
/// it the coil/the device/the bus is
/// resettet.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
