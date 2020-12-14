// An application that is launched 
//

use crate::initrc::Ns;
use crate::initrc::Cap;
/// Trait for Application configuration. Applications
/// have to be statically configured and hence the liberal
/// use of static
pub trait ApplicationConfig{
    fn name(&self) -> &'static str;
    fn depends(&self) -> &'static [&'static str];
    fn after(&self) -> &'static [&'static str];
    fn start_params(&self) -> &'static [&'static str];
    fn restart_params(&self) -> &'static [&'static str];
    fn class(&self) -> Option<&'static str>;
    fn io_prio(&self) -> Option<&'static str>;
    fn uid(&self) -> u32;
    fn gid(&self) -> u32;
    fn groups(&self) -> &'static [u32];
    fn namespaces(&self) -> &'static [Ns];
    fn workdir(&self) -> Option<&'static str>;
    fn capabilities(&self) -> &'static [Cap];
    fn environment(&self) -> &'static [[&'static str;2]];

    /// The entry function for this Application.
    fn create(&self, params:&CreateParams) ->  Option<std::boxed::Box<dyn Application>>;
}

pub struct CreateParams {

}
pub struct RunParams {

}

pub trait Application {
    /// This is the main loop of the application. This 
    /// function is not expected to return
    fn run(&mut self,params:&RunParams) -> Result<(), Box<dyn std::error::Error>>;
}
