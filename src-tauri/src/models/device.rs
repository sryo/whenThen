use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromecastDeviceInfo {
    pub id: String,
    pub name: String,
    pub model: String,
    pub address: String,
    pub port: u16,
    pub status: DeviceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Discovered,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub id: String,
    pub name: String,
    pub model: String,
    pub address: String,
    pub port: u16,
}

impl DiscoveredDevice {
    pub fn to_info(&self, status: DeviceStatus) -> ChromecastDeviceInfo {
        ChromecastDeviceInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            model: self.model.clone(),
            address: self.address.clone(),
            port: self.port,
            status,
        }
    }
}
