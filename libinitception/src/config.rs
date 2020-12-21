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

// An application that is launched. This is the interface used by applications to provide
// configuration information. Basically this is the contents of the initrc.

use crate::initrc::Cap;
use crate::initrc::Ns;
/// Trait for Application configuration. Applications
/// have to be statically configured and hence the liberal
/// use of static
pub trait ApplicationConfig {
    fn name(&self) -> &'static str;
    fn depends(&self) -> &'static [&'static str];
    fn after(&self) -> &'static [&'static str];
    fn start_params(&self) -> &'static [&'static str];
    fn restart_params(&self) -> &'static [&'static str];
    /// restart_count -> (restarts, delay_ms)
    /// return a tuple with two elements
    /// restarts : How many times to attempt a restart of this application
    /// delay : Delay between restarts
    fn restart_count(&self) -> Option<(u32, u32)>;
    fn class(&self) -> Option<&'static str>;
    fn io_prio(&self) -> Option<&'static str>;
    fn uid(&self) -> u32;
    fn gid(&self) -> u32;
    fn groups(&self) -> &'static [u32];
    fn namespaces(&self) -> &'static [Ns];
    fn workdir(&self) -> Option<&'static str>;
    fn capabilities(&self) -> &'static [Cap];
    fn environment(&self) -> &'static [[&'static str; 2]];

    /// The entry function for this Application.
    fn create(&self, params: &CreateParams) -> Option<std::boxed::Box<dyn Application>>;
}

pub struct CreateParams {}
pub struct RunParams {}

pub trait Application {
    /// This is the main loop of the application. This
    /// function is not expected to return
    fn run(&mut self, params: &RunParams) -> Result<(), Box<dyn std::error::Error>>;
}
