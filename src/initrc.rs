// Parse configuration file
// Isn't serde beautiful?

extern crate either;
extern crate petgraph;
extern crate toml;

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub service: Vec<Option<Service>>,
    pub unit: Option<Vec<Option<Unit>>>,
}

#[derive(Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub enum Cap {
    CAP_CHOWN,
    CAP_DAC_OVERRIDE,
    CAP_DAC_READ_SEARCH,
    CAP_FOWNER,
    CAP_FSETID,
    CAP_KILL,
    CAP_SETGID,
    CAP_SETUID,
    CAP_SETPCAP,
    CAP_LINUX_IMMUTABLE,
    CAP_NET_BIND_SERVICE,
    CAP_NET_BROADCAST,
    CAP_NET_ADMIN,
    CAP_NET_RAW,
    CAP_IPC_LOCK,
    CAP_IPC_OWNER,
    CAP_SYS_MODULE,
    CAP_SYS_RAWIO,
    CAP_SYS_CHROOT,
    CAP_SYS_PTRACE,
    CAP_SYS_PACCT,
    CAP_SYS_ADMIN,
    CAP_SYS_BOOT,
    CAP_SYS_NICE,
    CAP_SYS_RESOURCE,
    CAP_SYS_TIME,
    CAP_SYS_TTY_CONFIG,
    CAP_MKNOD,
    CAP_LEASE,
    CAP_AUDIT_WRITE,
    CAP_AUDIT_CONTROL,
    CAP_SETFCAP,
    CAP_MAC_OVERRIDE,
    CAP_MAC_ADMIN,
    CAP_SYSLOG,
    CAP_WAKE_ALARM,
    CAP_BLOCK_SUSPEND,
    CAP_AUDIT_READ,
}

#[derive(Deserialize, Debug)]
pub enum Ns {
    Pid,
    Net,
    Mount,
    Uts,
    Ipc,
    User,
}

#[derive(Deserialize, Debug)]
pub enum Type {
    Notify,
}

#[derive(Deserialize, Debug)]
pub struct Service {
    pub name: String,
    pub path: PathBuf,
    pub depends: Option<Vec<String>>,
    pub after: Option<String>,
    pub start_params: Option<Vec<String>>,
    pub restart_params: Option<Vec<String>>,
    pub restart: Option<Restart>,
    pub class: Option<String>,
    pub io_prio: Option<String>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub groups: Option<Vec<u32>>,
    pub namespaces: Option<Vec<Ns>>,
    pub workdir: Option<String>,
    pub capabilities: Option<Vec<Cap>>,
    pub env: Option<Vec<[String; 2]>>,
    pub r#type: Option<Type>,
}

impl Service {
    pub fn is_notify_type(&self) -> bool {
        if let Some(t) = &self.r#type {
            match t {
                Type::Notify => true,
            }
        } else {
            false
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Restart {
    #[serde(default = "default_count")]
    pub count: u32,
    #[serde(default = "default_period_ms")]
    pub period_ms: u32,
}
// functions to set default values
fn default_count() -> u32 {
    1
}
fn default_period_ms() -> u32 {
    1000
}

#[derive(Deserialize, Debug, Clone)]
pub enum UnitType {
    Mount,
    Net,
}

#[derive(Deserialize, Debug)]
pub struct Unit {
    pub r#type: UnitType, // Mandatory field
    pub name: String,
    pub depends: Option<Vec<String>>,
    pub device: String,
    pub params: Option<String>,
    pub data: Option<String>,
    pub flags: Option<String>,
}

// Parse the configuration file at /etc/initrc and return the config
// structure if successful
pub fn load_config() -> Option<Config> {
    if let Ok(s) = fs::read_to_string("/etc/initrc") {
        let config: Config = toml::from_str(&s).unwrap();
        println!("Loaded initrc");
        Some(config)
    } else {
        println!("Failed to load initrc");
        None
    }
}
