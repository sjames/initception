/// Parse the ueventd config file at /etc/ueventd.rc
extern crate toml;
use nix::sys::stat::{Mode, SFlag};
use serde::Deserialize;
use std::fs;
use tracing::{debug, error, info, Level};

const UEVENTD_CFG_LOCATION: &'static str = "/etc/ueventd.rc";

#[derive(Deserialize, Debug)]
pub struct UEventRcConfig {
    pub firmware_directories: Option<Vec<String>>,
    pub subsystem: Option<Vec<Subsystem>>,
    pub devices: Option<Vec<(String, u32, String, String)>>,
    pub sysfs: Option<Vec<(String, String, u32, String, String)>>,
}

#[derive(Deserialize, Debug)]
pub struct Subsystem {
    name: String,
    devname: String,
    dirname: String,
}

/// Works only for simple cases where the wildcard is at the end of the string
/// I don't want to pull in the regex crate for this.
fn is_match(pattern: &str, device: &str) -> bool {
    //if the pattern has a *, take it out.
    let pattern: Vec<&str> = pattern.split("*").take(1).collect();
    if device.contains(pattern[0]) {
        true
    } else {
        false
    }
}

impl UEventRcConfig {
    ///return the mode, major and minor number of the device.
    /// dev : full path to device
    pub fn get_device_mode_and_ids(&self, dev: &str) -> Option<(u32, &str, &str)> {
        if let Some(devices) = &self.devices {
            if let Some(dev) = devices.iter().find(|entry| is_match(&entry.0, dev)) {
                return Some((dev.1, dev.2.as_str(), dev.3.as_str()));
            } else {
                return None;
            }
        }
        None
    }
}

// Parse the configuration file at /etc/initrc and return the config
// structure if successful
pub fn load_config() -> Option<UEventRcConfig> {
    if let Ok(s) = fs::read_to_string(UEVENTD_CFG_LOCATION) {
        let config: UEventRcConfig = toml::from_str(&s).unwrap();
        info!("Loaded ueventd.rc");
        Some(config)
    } else {
        error!("Failed to load initrc");
        None
    }
}
