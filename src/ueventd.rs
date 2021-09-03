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

use crate::common::{DeviceChangeInfo, TaskMessage};
use crate::userids;
use crate::uventrc_parser;

use netlink_sys::{Protocol, Socket, SocketAddr};
use nix::sys::stat::makedev;
use nix::sys::stat::{mknod, mode_t, Mode, SFlag};
use std::convert::TryFrom;
use std::ffi::CString;
use std::fmt;
use std::path::Path;
use std::str;
use tracing::{debug, error};
use uventrc_parser::UEventRcConfig;

#[derive(PartialEq)]
pub enum Action {
    Unknown,
    Add,
    Change,
    Remove,
}

pub struct UEvent {
    action: Action,
    dev_path: String,
    maybe_subsystem: Option<String>,
    maybe_firmware: Option<String>,
    maybe_major: Option<u64>,
    maybe_minor: Option<u64>,
    maybe_devname: Option<String>,
    maybe_partitionnum: Option<i32>,
    maybe_partitionname: Option<String>,
    maybe_modalias: Option<String>,
}

impl UEvent {
    pub fn get_dev_type(&self) -> SFlag {
        if let Some(subsystem) = &self.maybe_subsystem {
            if subsystem == "block" {
                SFlag::S_IFBLK
            } else {
                SFlag::S_IFCHR
            }
        } else {
            SFlag::S_IFCHR
        }
    }

    pub fn is_subsystem(&self, name: &str) -> bool {
        if let Some(subsystem) = &self.maybe_subsystem {
            subsystem == name
        } else {
            false
        }
    }
}

impl fmt::Display for UEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Action:{} Devpath: {} Devname: {} Subsystem: {} Major: {} Minor: {}",
            match self.action {
                Action::Add => String::from("Add"),
                Action::Remove => String::from("Remove"),
                Action::Change => String::from("Change"),
                _ => String::from("Unknown"),
            },
            self.dev_path,
            self.maybe_devname
                .as_ref()
                .unwrap_or(&String::from("Unknown")),
            self.maybe_subsystem
                .as_ref()
                .unwrap_or(&String::from("Unknown")),
            self.maybe_major.unwrap_or(0),
            self.maybe_minor.unwrap_or(0)
        )
    }
}

impl TryFrom<&[u8]> for UEvent {
    type Error = &'static str;
    fn try_from(buf: &[u8]) -> Result<UEvent, Self::Error> {
        let lines = buf.split(|b| *b == 0u8).skip(1);

        let mut uevent = UEvent {
            action: Action::Unknown,
            dev_path: String::new(),
            maybe_firmware: None,
            maybe_subsystem: None,
            maybe_major: None,
            maybe_minor: None,
            maybe_devname: None,
            maybe_partitionnum: None,
            maybe_partitionname: None,
            maybe_modalias: None,
        };

        for line in lines {
            let tokens: Vec<&[u8]> = line.split(|b| *b == b'=').collect();
            if tokens.len() == 2 {
                // must have exactly two fields
                let key = str::from_utf8(tokens[0]).unwrap();
                let value = str::from_utf8(tokens[1]).unwrap();

                match key {
                    "ACTION" => {
                        uevent.action = match value {
                            "add" => Action::Add,
                            "remove" => Action::Remove,
                            "change" => Action::Change,
                            _ => Action::Unknown,
                        }
                    }
                    "DEVPATH" => uevent.dev_path = String::from(value),
                    "SUBSYSTEM" => uevent.maybe_subsystem = Some(String::from(value)),
                    "MAJOR" => uevent.maybe_major = Some(value.parse().unwrap_or(0)),
                    "MINOR" => uevent.maybe_minor = Some(value.parse().unwrap_or(0)),
                    "DEVNAME" => uevent.maybe_devname = Some(String::from(value)),
                    "FIRMWARE" => uevent.maybe_firmware = Some(String::from(value)),
                    "PARTN" => uevent.maybe_partitionnum = Some(value.parse().unwrap_or(0)),
                    "PARTNAME" => uevent.maybe_partitionname = Some(String::from(value)),
                    "MODALIAS" => uevent.maybe_modalias = Some(String::from(value)),
                    _ => {}
                }
            } else if tokens.len() > 2 {
                error!("number of fields != 2");
                error!("Line:{:?}", tokens);
            }
        }

        if uevent.action != Action::Unknown {
            Ok(uevent)
        } else {
            Err("Unable to parse uevent")
        }
    }
}

/// This function calls blocking functions.
pub async fn uevent_main(tx: std::sync::mpsc::Sender<TaskMessage>) {
    debug!("uevent_main started");
    let kernel_unicast: SocketAddr = SocketAddr::new(0, 0xFFFF_FFFF);
    if let Some(uevent_cfg) = uventrc_parser::load_config() {
        let mut socket = Socket::new(Protocol::KObjectUevent).unwrap();
        debug!("uevent processing started");
        if let Err(err) = socket.bind(&kernel_unicast) {
            error!("Unable to bind socket due to {}", err);
        } else {
            let mut buf = vec![0; 1024 * 1];

            if tx.send(TaskMessage::UeventReady).is_err() {
                panic!("Receiver dropped");
            }
            loop {
                if let Ok((n, _addr)) = &socket.recv_from(&mut buf).await {
                    if let Ok(uevent) = UEvent::try_from(&buf[0..*n]) {
                        println!("{}", uevent);
                        if let Ok(changeinfo) = handle_uevent(uevent, &uevent_cfg) {
                            if tx.send(TaskMessage::DeviceChanged(changeinfo)).is_err() {
                                panic!("Receiver dropped");
                            }
                        }
                    } else {
                        error!("Could not parse uevent");
                    }
                }
            }
        }
    } else {
        panic!("Unable to parse ueventd config");
    }
    debug!("uevent_main ended");
}

fn handle_uevent(event: UEvent, cfg: &UEventRcConfig) -> Result<DeviceChangeInfo, ()> {
    match event.action {
        Action::Add => handle_add(event, cfg),
        Action::Remove => handle_remove(event, cfg),
        _ => Err(()),
    }
}

fn handle_remove(_event: UEvent, _cfg: &UEventRcConfig) -> Result<DeviceChangeInfo, ()> {
    Err(())
}

// returns true if the directory was created
fn make_dir_if_needed(dev: &Path) -> bool {
    if let Some(parent) = dev.parent() {
        matches!(std::fs::create_dir_all(parent), Ok(()))
    } else {
        false
    }
}

/// Return true if this event is meant for a subsystem device
fn is_subsystem_device(_event: &UEvent, _cfg: &UEventRcConfig) -> bool {
    false
}

fn handle_add(event: UEvent, cfg: &UEventRcConfig) -> Result<DeviceChangeInfo, ()> {
    if is_subsystem_device(&event, cfg) {
        // deal with this as a subsystem device
        Err(())
    } else if event.is_subsystem("net") {
        // net events will be sent everytime. No special
        // processing needed for net devices as this is
        // handled by the network module drive from the
        // main loop
        //println!("NET SUBSYSTEM : {}", event);
        let path = Path::new(&event.dev_path);
        if let Some(devname) = path.file_name() {
            Ok(DeviceChangeInfo::Added(String::from(
                devname.to_str().unwrap(),
            )))
        } else {
            Err(())
        }
    } else {
        // deal with this as a normal device
        if let Some(devname) = &event.maybe_devname {
            let devpath = format!("/dev/{}", devname);
            let dev = Path::new(&devpath);
            //info!("devpath:{}", devpath);
            if let Some((mode, user, group)) = cfg.get_device_mode_and_ids(&devpath) {
                if !dev.exists() && make_dir_if_needed(&dev) {
                    // reaching out to the unsafe function
                    // as I cannot figure out how to set the Mode directly.
                    unsafe {
                        let name =
                            CString::new(devpath.as_str()).expect("Unable to allocate CString");
                        libc::mknod(
                            name.as_ptr(),
                            (event.get_dev_type().bits() | mode) as mode_t,
                            makedev(event.maybe_major.unwrap(), event.maybe_minor.unwrap()),
                        );
                    }
                }
                // Now set the owner.
                let uid = userids::Ids::get_uid(user).unwrap();
                let gid = userids::Ids::get_gid(group).unwrap();

                debug!("Setting {} to uid:{}  gid:{}", &devpath, uid, gid);
                if let Err(e) = nix::unistd::chown(
                    dev.as_os_str(),
                    Some(nix::unistd::Uid::from_raw(uid)),
                    Some(nix::unistd::Gid::from_raw(gid)),
                ) {
                    error!("Could not set file ownership for {} due to {}", &devpath, e);
                    return Err(());
                }
                Ok(DeviceChangeInfo::Added(devpath))
            } else {
                // did not find a match, we ignore this device
                //info!("Ignoring device {}", devname);
                Err(())
            }
        } else {
            //debug!("ignoring add without device name");
            Err(())
        }
    }
}
