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
#[allow(clippy::all)]
use crate::context::{ContextReference, RuntimeEntity, ServiceIndex, UnitStatus};
use crate::initrc::{Unit, UnitType};
use nix::mount::{mount, MsFlags};

use tracing::{debug, error, info, warn};

// perform a mount operation
pub fn do_mount(context: ContextReference, index: ServiceIndex) -> std::result::Result<(), ()> {
    {
        let ctx = context.read().unwrap();
        let spawnref = ctx.get_ref(&index);
        // we need to write the status into the unit, so we get create a write lock
        let mut entity: &mut RuntimeEntity = &mut spawnref.write().unwrap();
        if let RuntimeEntity::Unit(unit) = &mut entity {
            if let Ok(()) = mount_unit(&unit.unit) {
                //debug!("Mount successful");
                unit.status = UnitStatus::Success;
                Ok(())
            } else {
                unit.status = UnitStatus::Error;
                error!("Unable to mount");
                Err(())
            }
        } else {
            panic!("Only units can be mounted");
        }
    }
}

fn parse_flags(flags: &str) -> Option<MsFlags> {
    let mut msflags: MsFlags =  unsafe {MsFlags::from_bits_unchecked(0)};//  MsFlags::MS_RDONLY ^ MsFlags::MS_RDONLY; // hmm.. is there no other way to initialize the flag?
    info!("msflags initialized to {:?}", msflags);
    for flag in flags.split(',') {
        let keyval: Vec<&str> = flag.split('=').collect();
        if keyval.len() == 2 {
            msflags = msflags
                | match keyval[0] {
                    "RDONLY" => MsFlags::MS_RDONLY,
                    "NOSUID" => MsFlags::MS_NOSUID,
                    "NODEV" => MsFlags::MS_NODEV,
                    "NOEXEC" => MsFlags::MS_NOEXEC,
                    "SYNCHRONOUS" => MsFlags::MS_SYNCHRONOUS,
                    "REMOUNT" => MsFlags::MS_REMOUNT,
                    "MANDLOCK" => MsFlags::MS_MANDLOCK,
                    "DIRSYNC" => MsFlags::MS_DIRSYNC,
                    "NOATIME" => MsFlags::MS_NOATIME,
                    "NODIRATIME" => MsFlags::MS_NODIRATIME,
                    "BIND" => MsFlags::MS_BIND,
                    "MOVE" => MsFlags::MS_MOVE,
                    "REC" => MsFlags::MS_REC,
                    "SILENT" => MsFlags::MS_SILENT,
                    "POSIXACL" => MsFlags::MS_POSIXACL,
                    "UNBINDABLE" => MsFlags::MS_UNBINDABLE,
                    "PRIVATE" => MsFlags::MS_PRIVATE,
                    "SLAVE" => MsFlags::MS_SLAVE,
                    "SHARED" => MsFlags::MS_SHARED,
                    "RELATIME" => MsFlags::MS_RELATIME,
                    "KERNMOUNT" => MsFlags::MS_KERNMOUNT,
                    "I_VERSION" => MsFlags::MS_I_VERSION,
                    "STRICTATIME" => MsFlags::MS_STRICTATIME,
                    "ACTIVE" => MsFlags::MS_ACTIVE,
                    "NOUSER" => MsFlags::MS_NOUSER,
                    "RMT_MASK" => MsFlags::MS_RMT_MASK,
                    "MGC_VAL" => MsFlags::MS_MGC_VAL,
                    "MGC_MSK" => MsFlags::MS_MGC_MSK,
                    //_ => MsFlags::MS_RDONLY ^ MsFlags::MS_RDONLY,
                    _ => unsafe{MsFlags::from_bits_unchecked(0)},
                };
        } else {
            warn!("Ignoring malformed flags");
        }
    }
    Some(msflags)
}

/// return the target and fstype parameters
fn parse_params(params: &str) -> (Option<&str>, Option<&str>) {
    let mut target = None;
    let mut fstype = None;
    for param in params.split(',') {
        let keyval: Vec<&str> = param.split('=').collect();
        if keyval.len() == 2 {
            match keyval[0] {
                "target" => target = Some(keyval[1]),
                "fstype" => fstype = Some(keyval[1]),
                _ => warn!("Ignoring unknown parameter"),
            }
        } else {
            warn!("Ignoring malformed param string");
        }
    }
    (target, fstype)
}

fn mount_unit(unit: &Unit) -> Result<(), &'static str> {
    if let UnitType::Mount = unit.r#type {
        if let Some(params) = &unit.params {
            let (target, fstype) = parse_params(&params);
            let flags = parse_flags(&params);
            let mut data = None;
            let device: Option<&[u8]> = Some(&unit.device.as_bytes());
            if let Some(d) = &unit.data {
                data = Some(d.as_bytes());
            };
            let mut fstype_b = None;
            if let Some(fstype) = fstype {
                fstype_b = Some(fstype.as_bytes());
            }
            debug!(
                "Mount: {:?} {:?} {:?} {:?} {:?}",
                &device,
                &target,
                &fstype,
                &flags.unwrap(),
                data
            );
            if let Some(target) = target {
                if let Err(e) = mount(device, target.as_bytes(), fstype_b, flags.unwrap(), data) {
                    error!("Error mounting device {:?} due to {}", device, e);
                    return Err("Mount error");
                } else {
                    return Ok(());
                }
            }
        }
    } else {
        panic!("mount attempted on an a type {:?}", unit.r#type);
    }

    Err("Unknown")
}
