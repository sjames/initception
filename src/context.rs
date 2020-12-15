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

use petgraph::prelude::NodeIndex;
use petgraph::Direction;
use petgraph::Graph;

use crate::initrc::{load_config, Service, Unit, UnitType};
use crate::mount;
use crate::network;
use crate::process::launch_service;

use std::os::unix::io::FromRawFd;
use std::os::unix::net::UnixStream;
use tracing::{debug, info, warn};
use unshare::ChildEvent;

use crate::application::src_gen::application_interface_ttrpc::ApplicationServiceClient;
use ttrpc::r#async::Client;

use crate::application::config::ApplicationConfig;

use std::time::Instant;

pub enum RuntimeEntity {
    Service(SpawnedService),
    Unit(SpawnedUnit),
}

impl RuntimeEntity {
    pub fn is_service(&self) -> bool {
        match *self {
            RuntimeEntity::Service(_) => true,
            _ => false,
        }
    }
    pub fn is_unit(&self) -> bool {
        match *self {
            RuntimeEntity::Unit(_) => true,
            _ => false,
        }
    }
    pub fn cleanup_resources(&mut self) {
        match self {
            RuntimeEntity::Service(s) => s.cleanup_resources(),
            RuntimeEntity::Unit(_u) => {}
        }
    }

    pub fn record_watchdog(&mut self) {
        match self {
            RuntimeEntity::Service(s) => s.record_watchdog(),
            RuntimeEntity::Unit(_u) => {}
        }
    }

    pub fn get_last_watchdog(&mut self) -> Option<Instant> {
        match self {
            RuntimeEntity::Service(s) => s.get_last_watchdog(),
            RuntimeEntity::Unit(_u) => None,
        }
    }
}

#[derive(Debug)]
pub struct SpawnedUnit {
    pub unit: Unit,
    pub status: UnitStatus,
}

#[derive(Debug)]
pub enum UnitStatus {
    Success,
    Error,
    Unknown,
}

pub struct SpawnedService {
    pub service: Service,
    pub child: Option<unshare::Child>,
    pub start_count: u32, // how many times this has been started
    pub state: RunningState,
    pub exit_status: Option<unshare::ExitStatus>,
    pub uuid: Option<String>, // The UUid for this instance of the application
    pub proxy: Option<ApplicationServiceClient>,
    // This is the socket to communicate with the application server
    pub client_fd: Option<UnixStream>,
    // socket to host the application manager server
    pub server_fd: Option<UnixStream>,
    pub appserver_terminate_handler: Option<tokio::sync::oneshot::Sender<()>>,
    last_watchdog: Option<Instant>,
}

impl SpawnedService {
    pub fn cleanup_resources(&mut self) {
        if let Some(sender) = self.appserver_terminate_handler.take() {
            if let Err(_) = sender.send(()) {
                panic!("Receiver dropped");
            }
        }
        if let Some(_proxy) = self.proxy.take() {
            // proxy will get dropped here.
        }

        if let Some(_fd) = self.client_fd.take() {
            // let it go out of scope
        }

        if let Some(_fd) = self.server_fd.take() {
            // let it go out of scope
        }
    }

    pub fn record_watchdog(&mut self) {
        self.last_watchdog = Some(Instant::now());
    }

    pub fn get_last_watchdog(&self) -> Option<Instant> {
        self.last_watchdog.clone()
    }
}
pub struct Context {
    children: Graph<RuntimeEntityReference, u32>,
}

impl Context {
    pub fn get_ref(&self, node_index: &NodeIndex) -> RuntimeEntityReference {
        // let client = ApplicationInterfaceAsyncRPCClient::new(BincodeAsyncClientTransport::new());

        self.children[*node_index].clone()
    }

    /// get the name of the runtime entity. it can either be a service or a unit.
    pub fn get_name(&self, node_index: NodeIndex) -> Option<String> {
        let entity: &RuntimeEntity = &self.children[node_index].read().unwrap();
        match entity {
            RuntimeEntity::Service(service) => Some(service.service.name.clone()),
            RuntimeEntity::Unit(unit) => Some(unit.unit.name.clone()),
        }
    }
}

#[derive(Debug)]
pub enum RunningState {
    Unknown,
    WaitForConnect, // waiting for process to connect and confirm running state
    Running,
    Stopped,
    Killed,
    Zombie,
}

pub type ServiceIndex = NodeIndex;
pub type SpawnReference = std::sync::Arc<std::sync::RwLock<SpawnedService>>;
pub type RuntimeEntityReference = std::sync::Arc<std::sync::RwLock<RuntimeEntity>>; // SpawnReference to be replaced by RuntimeEntityReference
pub type ContextReference = std::sync::Arc<std::sync::RwLock<Context>>;

impl<'a> Context {
    // Create empty context
    pub fn new() -> Context {
        Context {
            children: Graph::new(),
        }
    }

    // Add a service from an ApplicationConfiguration
    pub fn add_service(&mut self, config: &dyn ApplicationConfig) {
        let service: Service = config.into();
        let spawn = SpawnedService {
            service: service,
            child: None,
            start_count: 0,
            state: RunningState::Unknown,
            exit_status: None,
            uuid: None,
            proxy: None,
            client_fd: None,
            server_fd: None,
            appserver_terminate_handler: None,
            last_watchdog: None,
        };

        self.children
            .add_node(std::sync::Arc::new(std::sync::RwLock::new(
                RuntimeEntity::Service(spawn),
            )));
    }

    pub fn add_unit(&mut self, unit: Unit) {
        let spawn = SpawnedUnit {
            unit: unit,
            status: UnitStatus::Unknown,
        };
        self.children
            .add_node(std::sync::Arc::new(std::sync::RwLock::new(
                RuntimeEntity::Unit(spawn),
            )));
    }

    /// convert the Config structure to a context structure. The context structure includes
    /// a graph of the services.
    pub fn create_context() -> Option<Context> {
        if let Some(cfg) = load_config() {
            let mut context = Context {
                children: Graph::new(),
            };
            for it in cfg.service {
                let spawn = SpawnedService {
                    service: it.unwrap(),
                    child: None,
                    start_count: 0,
                    state: RunningState::Unknown,
                    exit_status: None,
                    uuid: None,
                    proxy: None,
                    client_fd: None,
                    server_fd: None,
                    appserver_terminate_handler: None,
                    last_watchdog: None,
                };
                context
                    .children
                    .add_node(std::sync::Arc::new(std::sync::RwLock::new(
                        RuntimeEntity::Service(spawn),
                    )));
            }
            if let Some(units) = cfg.unit {
                for it in units {
                    if let Some(item) = it {
                        let spawn = SpawnedUnit {
                            unit: item,
                            status: UnitStatus::Unknown,
                        };
                        context
                            .children
                            .add_node(std::sync::Arc::new(std::sync::RwLock::new(
                                RuntimeEntity::Unit(spawn),
                            )));
                    }
                }
            }

            let mut edges: Vec<(NodeIndex, NodeIndex)> = Vec::new();

            // now fix up the dependencies
            for node_index in context.children.node_indices() {
                let runtime_entity: &RuntimeEntity = &context.children[node_index].read().unwrap();
                match runtime_entity {
                    RuntimeEntity::Service(service) => {
                        if let Some(deps) = &service.service.depends {
                            for dep in deps {
                                let it_depends: Vec<NodeIndex> = context
                                    .children
                                    .node_indices()
                                    .filter(|&n| {
                                        n != node_index && dep == &context.get_name(n).unwrap()
                                    })
                                    .collect();
                                for d in it_depends {
                                    //println!("{} depends on {:?}",&context.children[node_index].service.name,d);
                                    edges.push((node_index, d))
                                }
                            }
                        }
                    }
                    RuntimeEntity::Unit(unit) => {
                        if let Some(deps) = &unit.unit.depends {
                            for dep in deps {
                                let it_depends: Vec<NodeIndex> = context
                                    .children
                                    .node_indices()
                                    .filter(|&n| {
                                        n != node_index && dep == &context.get_name(n).unwrap()
                                    })
                                    .collect();
                                for d in it_depends {
                                    //println!("{} depends on {:?}",&context.children[node_index].service.name,d);
                                    edges.push((node_index, d))
                                }
                            }
                        }
                    }
                }
            }

            for edge in edges {
                context.children.add_edge(edge.0, edge.1, 1);
            }

            match petgraph::algo::toposort(&context.children, None) {
                Ok(sorted) => {
                    //println!("{:#?}", &context.children);
                    println!("Toposorted:{:?}", sorted);
                }
                Err(cycle) => {
                    //println!("{:#?}", &context.children);
                    println!("Cycle:{:?}", cycle.node_id());
                    panic!("Dependency Cycle in initrc");
                }
            }

            Some(context)
        } else {
            None
        }
    }

    pub fn fixup_dependencies(&mut self) {
        // now fix up the dependencies
        let mut edges: Vec<(NodeIndex, NodeIndex)> = Vec::new();
        for node_index in self.children.node_indices() {
            let runtime_entity: &RuntimeEntity = &self.children[node_index].read().unwrap();
            match runtime_entity {
                RuntimeEntity::Service(service) => {
                    if let Some(deps) = &service.service.depends {
                        for dep in deps {
                            let it_depends: Vec<NodeIndex> = self
                                .children
                                .node_indices()
                                .filter(|&n| n != node_index && dep == &self.get_name(n).unwrap())
                                .collect();
                            for d in it_depends {
                                //println!("{} depends on {:?}",&context.children[node_index].service.name,d);
                                edges.push((node_index, d))
                            }
                        }
                    }
                }
                RuntimeEntity::Unit(unit) => {
                    if let Some(deps) = &unit.unit.depends {
                        for dep in deps {
                            let it_depends: Vec<NodeIndex> = self
                                .children
                                .node_indices()
                                .filter(|&n| n != node_index && dep == &self.get_name(n).unwrap())
                                .collect();
                            for d in it_depends {
                                //println!("{} depends on {:?}",&context.children[node_index].service.name,d);
                                edges.push((node_index, d))
                            }
                        }
                    }
                }
            }
        }

        for edge in edges {
            self.children.add_edge(edge.0, edge.1, 1);
        }

        match petgraph::algo::toposort(&self.children, None) {
            Ok(sorted) => {
                //println!("{:#?}", &context.children);
                println!("Toposorted:{:?}", sorted);
            }
            Err(cycle) => {
                //println!("{:#?}", &context.children);
                println!("Cycle:{:?}", cycle.node_id());
                panic!("Dependency Cycle in initrc");
            }
        }
    }

    /// Get the index of services that are started initially. These are the services that
    /// do not depend on any other services or any other triggers
    pub fn get_initial_services(&self) -> Vec<ServiceIndex> {
        self.children
            .node_indices()
            .filter(|&c| {
                let entity: &RuntimeEntity = &self.children[c].read().unwrap();
                if let RuntimeEntity::Service(service) = entity {
                    match &service.service.depends {
                        None => true,
                        Some(s) => s.len() == 0,
                    }
                } else {
                    false
                }
            })
            .collect()
    }

    /// return a service index given the pid. Return None if a matching service was not
    /// found
    fn get_service_by_pid(&self, pid: i32) -> Option<ServiceIndex> {
        let service_ids = self
            .children
            .node_indices()
            .filter(|&n| {
                let entity: &RuntimeEntity = &self.children[n].read().unwrap();
                if let RuntimeEntity::Service(service) = entity {
                    if let Some(child) = &service.child {
                        child.pid() == pid
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .collect::<Vec<ServiceIndex>>();
        if service_ids.len() > 1 {
            panic!("More than one service with the same pid found");
        } else if service_ids.len() == 1 {
            Some(service_ids[0])
        } else {
            None
        }
    }

    /// returns a reference to a service. Panics if the index is out of bounds.
    pub fn get_runtime_entity(&'a self, service: ServiceIndex) -> Option<RuntimeEntityReference> {
        Some(self.children[service].clone())
    }

    pub fn get_service(&'a self, service: ServiceIndex) -> Option<RuntimeEntityReference> {
        let entity: &RuntimeEntity = &self.children[service].read().unwrap();
        if let RuntimeEntity::Service(_service) = entity {
            Some(self.children[service].clone())
        } else {
            None
        }
    }

    // get the dependant services
    pub fn get_immediate_dependant_services(&self, index: ServiceIndex) -> Vec<ServiceIndex> {
        self.children
            .neighbors_directed(index, Direction::Incoming)
            .filter(|i| {
                let entity: &RuntimeEntity = &self.children[*i].read().unwrap();
                entity.is_service()
            })
            .collect()
    }
    /// returns a vector of the immediate dependants of a service. Every service that has this service
    /// in its "Depends" field will be returned.
    pub fn get_immediate_dependants(&self, index: ServiceIndex) -> Vec<ServiceIndex> {
        self.children
            .neighbors_directed(index, Direction::Incoming)
            .collect()
    }

    /// Check if this service should be restarted. Return the number of milliseconds
    /// to wait for, before performing the restart. If restart is not required, return None
    pub fn check_restart(&self, index: ServiceIndex) -> Option<u32> {
        //first make sure that this is not already running
        debug!("Check restart");
        let entity: &RuntimeEntity = &self.children[index].read().unwrap();
        if let RuntimeEntity::Service(service) = entity {
            if let RunningState::Running = service.state {
                // bail as this should not happen
                panic!("Unexpected to relaunch an already running service");
            } else {
                if let Some(restart) = &service.service.restart {
                    if restart.count == 0 || restart.count > service.start_count {
                        return Some(restart.period_ms);
                    } else {
                        // nothing to do, already restarted enough of times
                        info!(
                            "Service : {} restarted {} times. Not trying again",
                            service.service.name, restart.count
                        );
                        return None;
                    }
                }
            }
        } else {
            panic!("Only services can be restarted");
        }

        None
    }

    pub fn is_notify_type(&self, index: ServiceIndex) -> bool {
        let entity: &RuntimeEntity = &self.children[index].read().unwrap();
        if let RuntimeEntity::Service(service) = entity {
            service.service.is_notify_type()
        } else {
            false
        }
    }

    pub fn launch_service(&self, index: ServiceIndex) -> Result<(), nix::Error> {
        launch_service(self.children[index].clone()) // launch_service does the checks.
    }

    fn set_state(&mut self, index: ServiceIndex, state: RunningState) {
        let entity: &mut RuntimeEntity = &mut self.children[index].write().unwrap();
        if let RuntimeEntity::Service(service) = entity {
            service.state = state;
        } else {
            panic!("Tried to set exit status on a non Service")
        }
    }

    fn set_exit_status(&mut self, index: ServiceIndex, exit_status: unshare::ExitStatus) {
        let entity: &mut RuntimeEntity = &mut self.children[index].write().unwrap();
        if let RuntimeEntity::Service(service) = entity {
            service.exit_status = Some(exit_status);
        } else {
            panic!("Tried to set exit status on a non Service")
        }
    }

    /*
        fn get_unit_as_ref(&self, index: ServiceIndex) -> Option<&Unit> {
            let entity: &RuntimeEntity = &self.children[index].read().unwrap();
            if let RuntimeEntity::Unit(unit) = entity {
                Some(&unit.unit)
            } else {
                panic!("Tried to get unit on Service")
            }
        }
    */

    /// Find a matching device in the list of units and its type.  If there are
    /// multiple matches only the first one is returned
    pub fn find_device(&self, device: &str) -> Option<(ServiceIndex, UnitType)> {
        let mut types: Vec<UnitType> = Vec::new();
        let units = self
            .children
            .node_indices()
            .filter(|&n| {
                let entity: &RuntimeEntity = &self.children[n].read().unwrap();
                if let RuntimeEntity::Unit(unit) = entity {
                    if unit.unit.device == device {
                        &types.push(unit.unit.r#type.clone());
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .collect::<Vec<ServiceIndex>>();
        if units.len() == 0 {
            None
        } else if units.len() == 1 {
            Some((units[0], types[0].clone()))
        } else {
            warn!("More than one unit for device {}", device);
            Some((units[0], types[0].clone()))
        }
    }

    /// Attempt to "execute" a unit. If it is a netork unit, try
    /// to configure the network. If a mount unit, try to mount
    pub async fn do_unit(context: ContextReference, device: String) -> Result<ServiceIndex, ()> {
        let mut unit_index: Option<ServiceIndex> = None;
        let mut unit_type: Option<UnitType> = None;
        {
            let read_context = context.read().unwrap();
            if let Some(index) = read_context.find_device(&device) {
                unit_index = Some(index.0);
                unit_type = Some(index.1);
            }
        } // let the context go out of scope here.
        if let Some(index) = unit_index {
            match unit_type {
                Some(UnitType::Mount) => {
                    if let Ok(()) = mount::do_mount(context, index) {
                        Ok(index)
                    } else {
                        Err(())
                    }
                }
                Some(UnitType::Net) => {
                    if let Ok(()) = network::configure_network(context, index).await {
                        Ok(index)
                    } else {
                        Err(())
                    }
                }
                None => Err(()),
            }
        } else {
            Err(())
        }
    }

    /// return a tuple of vectors of killed, stopped and continued service ids.
    /// This function also updates the state inside the context we store for each
    /// service. Call this function when a SIGCHLD signal is received
    pub fn process_child_events(
        &mut self,
    ) -> (Vec<ServiceIndex>, Vec<ServiceIndex>, Vec<ServiceIndex>) {
        let mut killed = Vec::<ServiceIndex>::new();
        let mut stopped = Vec::<ServiceIndex>::new();
        let mut continued = Vec::<ServiceIndex>::new();

        for evt in unshare::child_events() {
            match evt {
                ChildEvent::Death(pid, status) => {
                    info!("Child dead pid:{} status:{}", pid, status);

                    if let Some(index) = self.get_service_by_pid(pid) {
                        killed.push(index);
                        self.set_exit_status(index, status);
                        self.set_state(index, RunningState::Killed);
                    }
                }
                ChildEvent::Stop(pid, signal) => {
                    info!("Child stopped pid:{} signal:{:?}", pid, signal);

                    if let Some(index) = self.get_service_by_pid(pid) {
                        stopped.push(index);
                        self.set_state(index, RunningState::Stopped);
                    }
                }
                ChildEvent::Continue(pid) => {
                    info!("Child continued pid:{}", pid);
                    if let Some(index) = self.get_service_by_pid(pid) {
                        continued.push(index);
                        self.set_state(index, RunningState::Running);
                    }
                }
            }
        }

        (killed, stopped, continued)
    }
}
