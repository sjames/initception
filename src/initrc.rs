/*
    Copyright 2020 Sojan James
    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at
        http://www.apache.org/licenses/LICENSE-2.0
    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

// Parse configuration file
// Isn't serde beautiful?

extern crate either;
extern crate petgraph;
extern crate toml;

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::application::app;
use crate::application::config::ApplicationConfig;
#[derive(Deserialize, Debug)]
pub struct Config {
    pub service: Vec<Option<Service>>,
    pub unit: Option<Vec<Option<Unit>>>,
}

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
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

pub trait ServiceTrait<'a> {
    fn get_name(&self) -> &str;
    fn get_path(&self) -> &Path;
    fn get_depends(&self) -> Option<&[&str]>;
    fn get_after(&self) -> Option<&str>;
    fn get_start_params(&self) -> Option<&[&str]>;
    fn get_restart_params(&self) -> Option<&[&str]>;
    fn get_restart(&self) -> Option<&Restart>;
    fn get_class(&self) -> Option<&str>;
    fn get_io_prio(&self) -> Option<&str>;
    fn get_uid(&self) -> u32;
    fn get_gid(&self) -> u32;
    fn get_groups(&self) -> Option<&[u32]>;
    fn get_namespaces(&self) -> Option<&[Ns]>;
    fn get_workdir(&self) -> Option<&str>;
    fn get_capabilities(&self) -> Option<&[Cap]>;
    fn get_env(&self) -> Option<&[[&str; 2]]>;
    fn get_type(&self) -> Option<Type>;
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
    // set to true if this configuration is static. The executable path is not used
    #[serde(skip_serializing)]
    pub is_static: bool,
}
#[derive(PartialEq)]
pub enum ServiceType {
    Normal,
    LifecycleManager,
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

    /// Some services have special privileges. These services are
    /// identified by special prefixes in their names.
    pub fn get_service_type(&self) -> ServiceType {
        match self.name.as_str() {
            app::APPNAME_LIFECYCLE_MANAGER => ServiceType::LifecycleManager,
            _ => ServiceType::Normal,
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

// Todo: The service struct stores copies. Check if this can be improved.
impl From<&dyn ApplicationConfig> for Service {
    fn from(cfg: &dyn ApplicationConfig) -> Self {
        Service {
            name: cfg.name().into(),
            path: PathBuf::new(),
            depends: Some(cfg.depends().iter().map(|d| String::from(*d)).collect()),
            after: None,
            start_params: Some(
                cfg.start_params()
                    .iter()
                    .map(|d| String::from(*d))
                    .collect(),
            ),
            restart_params: Some(
                cfg.restart_params()
                    .iter()
                    .map(|d| String::from(*d))
                    .collect(),
            ),
            restart: if let Some(restart) = cfg.restart_count() {
                Some(Restart {
                    count: restart.0,
                    period_ms: restart.1,
                })
            } else {
                None
            },
            class: cfg.class().map(String::from),
            io_prio: cfg.io_prio().map(String::from),
            uid: Some(cfg.uid()),
            gid: Some(cfg.uid()),
            groups: Some(cfg.groups().iter().copied().collect()),
            namespaces: Some(cfg.namespaces().to_vec()),
            workdir: cfg.workdir().map(String::from),
            capabilities: Some(cfg.capabilities().to_vec()),
            env: {
                let mut env = Vec::<[String; 2]>::new();
                for e in cfg.environment() {
                    env.push([String::from(e[0]), String::from(e[1])]);
                }
                Some(env)
            },
            r#type: Some(Type::Notify),
            // if created from ApplicationConfiguration, this is always static
            is_static: true,
        }
    }
}
