// An application that is launched 
//

use crate::initrc::Ns;
use crate::initrc::Cap;
/// Application configurations have to be static
pub trait ApplicationConfig{
    fn name() -> &'static str;
    fn depends() -> &'static [&'static str];
    fn after() -> &'static [&'static str];
    fn start_params() -> &'static [&'static str];
    fn restart_params() -> &'static [&'static str];
    fn class() -> Option<&'static str>;
    fn io_prio() -> Option<&'static str>;
    fn uid() -> u32;
    fn gid() -> u32;
    fn groups() -> &'static [&'static str];
    fn namespaces() -> &'static [Ns];
    fn workdir() -> Option<&'static str>;
    fn capabilities() -> &'static [Cap];
    fn environment() -> &'static [[&'static str;2]];

    /// The entry function for this Application.
    fn create(params:&CreateParams) ->  Option<std::boxed::Box<dyn Application>>;
}

pub struct CreateParams {

}
pub struct RunParams {

}


pub trait Application {
    /// This is the main loop of the application. This 
    /// function is not expected to return
    fn run(&mut self,params:&RunParams) -> i32;
}
