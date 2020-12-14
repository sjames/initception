
use getopts::Options;
use std::env;

use crate::initception;
use crate::sysfs_walker;
use crate::zygote;
use crate::application::config::{ApplicationConfig, CreateParams, RunParams};

use tracing::{error, info, Level, debug};

/// main library entry point


pub fn init_main( configs : &[&dyn ApplicationConfig]) -> Result<(), Box<dyn std::error::Error>> 
    //where F: FnOnce(&str) -> Result<(), Box<dyn std::error::Error>>
{
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("i", "", "Identity", "zygote|sysfswalk");
    opts.optopt("k", "", "key", "secret key");
    opts.optopt("e","","executable","executable name");
    opts.optflag("n","","not pid 1");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    let key = matches.opt_str("k");
    let identity = matches.opt_str("i");
    let notpid1 = matches.opt_present("n");

    if identity.is_some() {
        match identity.unwrap().as_ref() {
            "zygote" => zygote::zygote_main(key),
            "sysfswalk" => sysfs_walker::sysfs_walker_main(key),
            others => {
                debug!("Launching application : {}", others);
                if let Err(e) = launch_app(configs,others) {
                    panic!("Launch of {} failed due to {}", others, e);
                } else {
                    Ok(())
                }
            }
        }
    } else {
        info!("I N I T C E P T I O N");
        initception::initception_main_static(configs,!notpid1)    
    }
}

// look for the application in the application configuration array and launch it
fn launch_app(configs : &[&dyn ApplicationConfig], name:&str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(cfg) = configs.into_iter().find(|c| {
        c.name() == name
    }) {
        let params = CreateParams {
            // No create params for now
        };
        let run_params = RunParams {
            // No run params for now
        };
        let mut app = cfg.create(&params).unwrap();
        app.run(&run_params);
    }
    error!("Did not find application {}", name);
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound,name)))
}