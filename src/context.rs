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

use crate::mount;
use crate::network;
use crate::process::{launch_service, stop_service};
use libinitception::initrc::{load_config, Service, ServiceType, Unit, UnitType};

use regex::Regex;
use std::os::unix::net::UnixStream;
use tracing::{debug, error, info, warn};
use unshare::ChildEvent;

use crate::servers::application_client::ApplicationServiceProxy;

use libinitception::config::ApplicationConfig;

use std::collections::HashMap;
use std::time::Instant;

pub enum RuntimeEntity {
    Service(SpawnedService),
    Unit(SpawnedUnit),
}

impl RuntimeEntity {
    pub fn is_service(&self) -> Option<ServiceType> {
        match self {
            RuntimeEntity::Service(s) => Some(s.get_service_type()),
            _ => None,
        }
    }

    pub fn is_normal_service(&self) -> bool {
        match self {
            RuntimeEntity::Service(s) => s.get_service_type() == ServiceType::Normal,
            _ => false,
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, RuntimeEntity::Unit(_))
    }

    pub fn get_name(&self) -> Option<&String> {
        match self {
            RuntimeEntity::Service(s) => Some(&s.service.name),
            RuntimeEntity::Unit(u) => Some(&u.unit.name),
        }
    }

    pub fn get_service_status(&self) -> Option<RunningState> {
        match self {
            RuntimeEntity::Service(s) => Some(s.state),
            RuntimeEntity::Unit(_u) => None,
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
    pub fn take_client_fd(&mut self) -> Option<UnixStream> {
        match self {
            RuntimeEntity::Service(s) => s.client_fd.take(),
            RuntimeEntity::Unit(_u) => None,
        }
    }

    pub fn take_server_fd(&mut self) -> Option<UnixStream> {
        match self {
            RuntimeEntity::Service(s) => s.server_fd.take(),
            RuntimeEntity::Unit(_u) => None,
        }
    }

    pub fn set_service_proxy(&mut self, proxy: ApplicationServiceProxy) {
        match self {
            RuntimeEntity::Service(s) => s.proxy = Some(proxy),
            RuntimeEntity::Unit(_u) => panic!("Attempt to set proxy on Unit"),
        }
    }

    pub fn set_terminate_signal_channel(&mut self, channel: tokio::sync::oneshot::Sender<()>) {
        match self {
            RuntimeEntity::Service(s) => s.appserver_terminate_handler = Some(channel),
            RuntimeEntity::Unit(_u) => panic!("Attempt to set proxy on Unit"),
        }
    }

    pub fn add_property_filter(&mut self, filter: &str) -> Result<(), ()> {
        match self {
            RuntimeEntity::Service(s) => s.add_property_filter(filter),
            RuntimeEntity::Unit(_u) => panic!("Attempt to set proxy on Unit"),
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
    pub proxy: Option<ApplicationServiceProxy>,
    // This is the socket to communicate with the application server
    pub client_fd: Option<UnixStream>,
    // socket to host the application manager server
    pub server_fd: Option<UnixStream>,
    pub appserver_terminate_handler: Option<tokio::sync::oneshot::Sender<()>>,
    last_watchdog: Option<Instant>,
    property_filters: Option<Vec<Regex>>,
}

impl SpawnedService {
    pub fn cleanup_resources(&mut self) {
        self.state = RunningState::Stopped;
        if let Some(sender) = self.appserver_terminate_handler.take() {
            if sender.send(()).is_err() {
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
        self.last_watchdog
    }

    pub fn get_service_type(&self) -> ServiceType {
        self.service.get_service_type()
    }

    pub fn add_property_filter(&mut self, regex_string: &str) -> Result<(), ()> {
        if let Some(filter) = self.property_filters.as_mut() {
            if let Ok(regex) = Regex::new(regex_string) {
                filter.push(regex);
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn check_if_property_match(&self, prop_key: &str) -> bool {
        if let Some(filters) = &self.property_filters {
            for filter in filters {
                if filter.is_match(prop_key) {
                    return true;
                }
            }
            return false;
        } else {
            false
        }
    }

    pub async fn notify_property_change(&mut self, key: &str, value: &str) {
        if let Some(proxy) = self.proxy.as_mut() {
            if let Err(_ret) = proxy.property_changed(key, value).await {
                error!("Error sending property change to service. Ignored");
            }
        }
    }
}
pub struct Context {
    children: Graph<RuntimeEntityReference, u32>,
    properties: HashMap<String, String>,
}

impl Context {
    pub fn get_ref(&self, node_index: &NodeIndex) -> RuntimeEntityReference {
        // let client = ApplicationInterfaceAsyncRPCClient::new(BincodeAsyncClientTransport::new());

        self.children[*node_index].clone()
    }

    pub fn get_property(&self, key: &str) -> Option<String> {
        self.properties.get(key).map(|s| String::from(s))
    }

    pub fn write_property_unchecked(&mut self, key: String, value: String) -> Option<String> {
        self.properties.insert(key, value)
    }

    pub fn contains_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    /// get the name of the runtime entity. it can either be a service or a unit.
    pub fn get_name(&self, node_index: NodeIndex) -> Option<String> {
        let entity: &RuntimeEntity = &self.children[node_index].read().unwrap();
        match entity {
            RuntimeEntity::Service(service) => Some(service.service.name.clone()),
            RuntimeEntity::Unit(unit) => Some(unit.unit.name.clone()),
        }
    }

    pub fn is_running(&self, node_index: NodeIndex) -> bool {
        let entity: &RuntimeEntity = &self.children[node_index].read().unwrap();
        match entity {
            RuntimeEntity::Service(service) => {
                service.state == RunningState::Running
                    || service.state == RunningState::WaitForConnect
            }
            RuntimeEntity::Unit(_unit) => false,
        }
    }

    pub fn get_all_services(&self) -> Vec<String> {
        let names = self
            .children
            .raw_nodes()
            .iter()
            .filter_map(|n| {
                let node = n.weight.read().unwrap();
                if node.is_normal_service() {
                    Some(String::from(node.get_name().unwrap()))
                } else {
                    None
                }
            })
            .collect();
        names
    }

    pub fn get_service_status(&self, name: &str) -> Option<RunningState> {
        self.children.node_indices().find_map(|n| {
            let node = self.children[n].read().unwrap();
            if node.is_normal_service() && node.get_name().unwrap() == name {
                node.get_service_status()
            } else {
                None
            }
        })
    }

    pub fn get_service_index(&self, name: &str) -> Option<ServiceIndex> {
        self.children.node_indices().find_map(|n| {
            let node = self.children[n].read().unwrap();
            if node.is_normal_service() && node.get_name().unwrap() == name {
                Some(n)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum RunningState {
    Unknown,
    WaitForConnect, // waiting for process to connect and confirm running state
    Running,
    Paused,
    Stopped,
    /*
    Killed,
    Zombie,
    */
}

impl RunningState {
    pub fn is_alive(&self) -> bool {
        *self == RunningState::Running
            || *self == RunningState::Paused
            || *self == RunningState::WaitForConnect
    }
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
            properties: HashMap::new(),
        }
    }

    // Add a service from an ApplicationConfiguration
    pub fn add_service(&mut self, config: &dyn ApplicationConfig) {
        let service: Service = config.into();
        let spawn = SpawnedService {
            service,
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
            property_filters: None,
        };

        self.children
            .add_node(std::sync::Arc::new(std::sync::RwLock::new(
                RuntimeEntity::Service(spawn),
            )));
    }

    pub fn add_unit(&mut self, unit: Unit) {
        let spawn = SpawnedUnit {
            unit,
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
                properties: HashMap::new(),
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
                    property_filters: None,
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
                        Some(s) => s.is_empty(),
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
                entity.is_service().is_some()
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
            } else if let Some(restart) = &service.service.restart {
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

    pub async fn kill_service_dep(&self, index: ServiceIndex) -> Result<(), nix::Error> {
        stop_service(self.children[index].clone()).await
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
                        types.push(unit.unit.r#type.clone());
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .collect::<Vec<ServiceIndex>>();
        if units.is_empty() {
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
                        self.set_state(index, RunningState::Stopped);
                    }
                }
                ChildEvent::Stop(pid, signal) => {
                    info!("Child stopped pid:{} signal:{:?}", pid, signal);

                    if let Some(index) = self.get_service_by_pid(pid) {
                        stopped.push(index);
                        self.set_state(index, RunningState::Paused);
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

pub async fn kill_service(
    context: ContextReference,
    index: ServiceIndex,
) -> Result<(), nix::Error> {
    let runtime = context.read().unwrap().children[index].clone();
    crate::process::stop_service(runtime).await
}

pub async fn pause_service(
    context: ContextReference,
    index: ServiceIndex,
) -> Result<(), nix::Error> {
    let runtime = context.read().unwrap().children[index].clone();
    crate::process::pause_service(runtime).await
}

pub async fn resume_service(
    context: ContextReference,
    index: ServiceIndex,
) -> Result<(), nix::Error> {
    let runtime = context.read().unwrap().children[index].clone();
    crate::process::pause_service(runtime).await
}
