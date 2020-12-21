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

/*
Process related.
Use unshare to launch processes or containers.
*/

use crate::context::{RunningState, RuntimeEntity, RuntimeEntityReference};
use libinitception::initrc::{Cap, Ns, Type};

use unshare::{Capability, Fd, Namespace};

use std::os::unix::io::IntoRawFd;
use std::os::unix::net::UnixStream;

use tracing::{debug, info, error};

use libinitception::app::{NOTIFY_APP_CLIENT_FD, NOTIFY_APP_SERVER_FD};

fn create_self_command(name: &str) -> unshare::Command {
    let path = std::fs::read_link("/proc/self/exe").expect("Unable to read /proc/self/exe");

    let mut cmd = unshare::Command::new(&path);
    cmd.arg0(name);
    cmd.arg("-i").arg(name);
    cmd
}

pub async fn resume_service(spawned_ref: RuntimeEntityReference) -> Result<(), nix::Error> {

    let mut entity: &mut RuntimeEntity = &mut spawned_ref.write().unwrap();
    let err = if let &mut RuntimeEntity::Service(spawn) = &mut entity {
        if spawn.state.is_alive() {
            if let Some(child) = &mut spawn.child {
                // send sigint first, then SIGTERM
                info!("Sending SIGCONT to process with PID : {}",child.pid());
                if let  Err(e) = child.signal(unshare::Signal::SIGCONT) {
                    error!("Failed to send SIGCONT to PID:{} ({})", child.pid(),e);
                } 
                spawn.state = RunningState::Running;
                Ok(())
            } else {
                error!("Task is not alive");
                Err(nix::Error::invalid_argument())    
            }
        } else {
            Err(nix::Error::invalid_argument())    
        }
    } else {
        Err(nix::Error::invalid_argument())
    };

    if let Ok(_e) = err {

        let proxy = {
            let mut entity: &mut RuntimeEntity = &mut spawned_ref.write().unwrap();
            if let &mut RuntimeEntity::Service(spawn) = &mut entity {
                if let Some(proxy) = &mut spawn.proxy {
                    // This clone is cheap as the proxy is basically a wrapper for a sender end of 
                    // a channel.
                    Some(proxy.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(mut proxy) = proxy {
            let timeout = std::time::Duration::from_millis(2000);
            debug!("Sending resume notification to application ");
            let _e = proxy.resume(timeout).await;
            debug!("Sent resume notification to application ");   
        } else {
            info!("No application proxy available");
        }
    Ok(())
    } else {
        err
    }
}



pub async fn pause_service(spawned_ref: RuntimeEntityReference) -> Result<(), nix::Error> {
    let proxy = {
        let mut entity: &mut RuntimeEntity = &mut spawned_ref.write().unwrap();
        if let &mut RuntimeEntity::Service(spawn) = &mut entity {
            if let Some(proxy) = &mut spawn.proxy {
                // This clone is cheap as the proxy is basically a wrapper for a sender end of 
                // a channel.
                Some(proxy.clone())
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(mut proxy) = proxy {
        let timeout = std::time::Duration::from_millis(2000);
        debug!("Sending pause notification to application ");
        let _e = proxy.pause(timeout).await;
        debug!("Sent pause notification to application ");   
    } else {
        info!("No application proxy available");
    }

    let mut entity: &mut RuntimeEntity = &mut spawned_ref.write().unwrap();
    if let &mut RuntimeEntity::Service(spawn) = &mut entity {
        if spawn.state.is_alive() {
            if let Some(child) = &mut spawn.child {
                // send sigint first, then SIGTERM
                info!("Sending SIGSTOP to process with PID : {}",child.pid());
                if let  Err(e) = child.signal(unshare::Signal::SIGSTOP) {
                    error!("Failed to send SIGSTOP to PID:{} ({})", child.pid(),e);
                } 
                Ok(())
            } else {
                error!("Task is not alive");
                Err(nix::Error::invalid_argument())    
            }
        } else {
            Err(nix::Error::invalid_argument())    
        }
    } else {
        Err(nix::Error::invalid_argument())
    }
}


pub async fn stop_service(spawned_ref: RuntimeEntityReference) -> Result<(), nix::Error> {
    let proxy = {
        let mut entity: &mut RuntimeEntity = &mut spawned_ref.write().unwrap();
        if let &mut RuntimeEntity::Service(spawn) = &mut entity {
            if let Some(proxy) = &mut spawn.proxy {
                // This clone is cheap as the proxy is basically a wrapper for a sender end of 
                // a channel.
                Some(proxy.clone())
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(mut proxy) = proxy {
        let timeout = std::time::Duration::from_millis(2000);
        debug!("Sending stop notification to application ");
        let _e = proxy.stop(timeout).await;
        debug!("Sent stop notification to application ");   
    } else {
        info!("No application proxy available");
    }

    let mut entity: &mut RuntimeEntity = &mut spawned_ref.write().unwrap();
    if let &mut RuntimeEntity::Service(spawn) = &mut entity {
        if spawn.state.is_alive() {
            if let Some(child) = &mut spawn.child {
                // send sigint first, then SIGTERM
                info!("Sending SIGINT and SIGTERM to process with PID : {}",child.pid());
                if let  Err(e) = child.signal(unshare::Signal::SIGINT) {
                    error!("Failed to send SIGINT to PID:{} ({})", child.pid(),e);
                } else if let Err(e) = child.kill() {
                    error!("Failed to send SIGTERM to PID:{} ({})", child.pid(), e);
                }
                Ok(())
            } else {
                error!("Task is not alive");
                Err(nix::Error::invalid_argument())    
            }
        } else {
            Err(nix::Error::invalid_argument())    
        }
    } else {
        Err(nix::Error::invalid_argument())
    }
}


/// launch a process, returining the Child structure for the newly
/// launched child process
pub fn launch_service(spawned_ref: RuntimeEntityReference) -> Result<(), nix::Error> {
    let mut entity: &mut RuntimeEntity = &mut spawned_ref.write().unwrap();

    if let &mut RuntimeEntity::Service(spawn) = &mut entity {
        let service = &mut spawn.service;
        let mut cmd = if service.is_static {
            create_self_command(&service.name)
        } else {
            unshare::Command::new(&service.path)
        };
        let mut namespaces = Vec::<Namespace>::new();
        let mut keepcaps = Vec::<Capability>::new();

        if let Some(workdir) = &service.workdir {
            cmd.current_dir(workdir);
        } else {
            cmd.current_dir("/");
        }

        if let Some(uid) = service.uid {
            cmd.uid(uid);
        }

        if let Some(group) = service.gid {
            cmd.gid(group);
        }

        if let Some(groups) = &service.groups {
            cmd.groups(groups.clone()); //TODO: why is clone needed here?
        }
        if let Some(nspaces) = &service.namespaces {
            //println!("Before ns loop");
            for ns in nspaces {
                match ns {
                    Ns::Pid => namespaces.push(Namespace::Pid),
                    Ns::Net => namespaces.push(Namespace::Net),
                    Ns::Mount => namespaces.push(Namespace::Mount),
                    Ns::Uts => namespaces.push(Namespace::Uts),
                    Ns::Ipc => namespaces.push(Namespace::Ipc),
                    Ns::User => namespaces.push(Namespace::User),
                }
            }
            cmd.unshare(&namespaces);
        }
        if let Some(caps) = &service.capabilities {
            for cap in caps {
                match cap {
                    Cap::CAP_CHOWN => keepcaps.push(Capability::CAP_CHOWN),
                    Cap::CAP_DAC_OVERRIDE => keepcaps.push(Capability::CAP_MAC_OVERRIDE),
                    Cap::CAP_DAC_READ_SEARCH => keepcaps.push(Capability::CAP_DAC_READ_SEARCH),
                    Cap::CAP_FOWNER => keepcaps.push(Capability::CAP_FOWNER),
                    Cap::CAP_FSETID => keepcaps.push(Capability::CAP_FSETID),
                    Cap::CAP_KILL => keepcaps.push(Capability::CAP_KILL),
                    Cap::CAP_SETGID => keepcaps.push(Capability::CAP_SETGID),
                    Cap::CAP_SETUID => keepcaps.push(Capability::CAP_SETUID),
                    Cap::CAP_SETPCAP => keepcaps.push(Capability::CAP_SETPCAP),
                    Cap::CAP_LINUX_IMMUTABLE => keepcaps.push(Capability::CAP_LINUX_IMMUTABLE),
                    Cap::CAP_NET_BIND_SERVICE => keepcaps.push(Capability::CAP_NET_BIND_SERVICE),
                    Cap::CAP_NET_BROADCAST => keepcaps.push(Capability::CAP_NET_BROADCAST),
                    Cap::CAP_NET_ADMIN => keepcaps.push(Capability::CAP_NET_ADMIN),
                    Cap::CAP_NET_RAW => keepcaps.push(Capability::CAP_NET_RAW),
                    Cap::CAP_IPC_LOCK => keepcaps.push(Capability::CAP_IPC_LOCK),
                    Cap::CAP_IPC_OWNER => keepcaps.push(Capability::CAP_IPC_OWNER),
                    Cap::CAP_SYS_MODULE => keepcaps.push(Capability::CAP_SYS_MODULE),
                    Cap::CAP_SYS_RAWIO => keepcaps.push(Capability::CAP_SYS_RAWIO),
                    Cap::CAP_SYS_CHROOT => keepcaps.push(Capability::CAP_SYS_CHROOT),
                    Cap::CAP_SYS_PTRACE => keepcaps.push(Capability::CAP_SYS_PTRACE),
                    Cap::CAP_SYS_PACCT => keepcaps.push(Capability::CAP_SYS_PACCT),
                    Cap::CAP_SYS_ADMIN => keepcaps.push(Capability::CAP_SYS_ADMIN),
                    Cap::CAP_SYS_BOOT => keepcaps.push(Capability::CAP_SYS_BOOT),
                    Cap::CAP_SYS_NICE => keepcaps.push(Capability::CAP_SYS_NICE),
                    Cap::CAP_SYS_RESOURCE => keepcaps.push(Capability::CAP_SYS_RESOURCE),
                    Cap::CAP_SYS_TIME => keepcaps.push(Capability::CAP_SYS_TIME),
                    Cap::CAP_SYS_TTY_CONFIG => keepcaps.push(Capability::CAP_SYS_TTY_CONFIG),
                    Cap::CAP_MKNOD => keepcaps.push(Capability::CAP_MKNOD),
                    Cap::CAP_LEASE => keepcaps.push(Capability::CAP_LEASE),
                    Cap::CAP_AUDIT_WRITE => keepcaps.push(Capability::CAP_AUDIT_WRITE),
                    Cap::CAP_AUDIT_CONTROL => keepcaps.push(Capability::CAP_AUDIT_WRITE),
                    Cap::CAP_SETFCAP => keepcaps.push(Capability::CAP_SETFCAP),
                    Cap::CAP_MAC_OVERRIDE => keepcaps.push(Capability::CAP_MAC_OVERRIDE),
                    Cap::CAP_MAC_ADMIN => keepcaps.push(Capability::CAP_MAC_ADMIN),
                    Cap::CAP_SYSLOG => keepcaps.push(Capability::CAP_SYSLOG),
                    Cap::CAP_WAKE_ALARM => keepcaps.push(Capability::CAP_WAKE_ALARM),
                    Cap::CAP_BLOCK_SUSPEND => keepcaps.push(Capability::CAP_BLOCK_SUSPEND),
                    Cap::CAP_AUDIT_READ => keepcaps.push(Capability::CAP_AUDIT_READ),
                }
            }
            cmd.keep_caps(keepcaps.iter());
        }

        cmd.stdout(unshare::Stdio::inherit());
        cmd.stdin(unshare::Stdio::inherit());
        cmd.stderr(unshare::Stdio::inherit());

        if let Some(env) = &service.env {
            for e in env {
                cmd.env(&e[0], &e[1]);
            }
        }

        // Socket for the application to connect back to the application manager
        let (my_client_sock, child_client_socket) = match UnixStream::pair() {
            Ok((sock1, sock2)) => (sock1, sock2),
            Err(e) => {
                panic!("Couldn't create a pair of sockets for app server: {:?}", e);
            }
        };

        let raw_fd = child_client_socket.into_raw_fd();
        spawn.server_fd = Some(my_client_sock);

        cmd.env(NOTIFY_APP_CLIENT_FD, format!("{}", raw_fd));
        cmd.file_descriptor(raw_fd, Fd::inherit());

        //
        //
        // Socket for the application to host the application server
        let (mysock, childsocket) = match UnixStream::pair() {
            Ok((sock1, sock2)) => (sock1, sock2),
            Err(e) => {
                panic!(
                    "Couldn't create a pair of sockets for app manager server: {:?}",
                    e
                );
            }
        };

        // raw_fd goes to the child process
        let raw_fd = childsocket.into_raw_fd();
        spawn.client_fd = Some(mysock);

        cmd.env(NOTIFY_APP_SERVER_FD, format!("{}", raw_fd));
        cmd.file_descriptor(raw_fd, Fd::inherit());

        debug!("Before spawn");
        let child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                error!("Error: {}", e);
                return Err(nix::Error::Sys(nix::errno::Errno::UnknownErrno));
            }
        };

        debug!("Launched service : {}", service.name);
        // let mut spawned_service = spawned_service.write().unwrap();
        spawn.child = Some(child);
        if let RunningState::Running = spawn.state {
            panic!("Trying to launch an already running service");
        } else {
            // If the service is of type notify, we set to running only
            // after it connects back to us
            if let Some(state) = &spawn.service.r#type {
                match state {
                    Type::Notify => spawn.state = RunningState::WaitForConnect,
                }
            } else {
                
            }
            spawn.start_count += 1;
        }

        Ok(())
    } else {
        Err(nix::Error::invalid_argument())
    }
}
