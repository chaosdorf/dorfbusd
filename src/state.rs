pub struct DeviceStatus {
    pub seen: bool,
    pub coils: BTreeMap<u16, CoilStatus>,
}

pub enum CoilStatus {
    On,
    Off,
    Unknown,
}
