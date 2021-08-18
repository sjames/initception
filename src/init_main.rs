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

use getopts::Options;
use std::env;

use crate::initception;
use crate::sysfs_walker;
use crate::zygote;
use libinitception::config::{ApplicationConfig, CreateParams, RunParams};

use tracing::{debug, error, info};

pub enum FurtherProcessing {
    // This has been handled, do nothing
    DoNothing,
    // this is the first launch, initialize. The flag indicates if
    // we need to perform the early mounts
    FirstLaunch(bool),
    // launch the requested application
    LaunchProcess(String),
}

/// Check if the process was launched to handle internal processing.
/// this function will not return if the execution is meant for
/// internal use of initception.
pub fn handle_internal() -> Result<FurtherProcessing,std::io::Error> {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("i", "", "Identity", "zygote|sysfswalk");
    opts.optopt("k", "", "key", "secret key");
    opts.optopt("e", "", "executable", "executable name");
    opts.optflag("n", "", "not pid 1");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    let key = matches.opt_str("k");
    let identity = matches.opt_str("i");
    let notpid1 = matches.opt_present("n");

    if identity.is_some() {
        match identity.unwrap().as_ref() {
            "zygote" => {
                zygote::zygote_main(key).unwrap();
                Ok(FurtherProcessing::DoNothing)
            },
            "sysfswalk" => {
                sysfs_walker::sysfs_walker_main(key).unwrap();
                Ok(FurtherProcessing::DoNothing)
            },
            others => {
                Ok(FurtherProcessing::LaunchProcess(others.to_owned()))
            }
        }
    } else {
        Ok(FurtherProcessing::FirstLaunch(notpid1))
    }

}

/// main library entry point
pub fn init_main(configs: &[&dyn ApplicationConfig]) -> Result<(), Box<dyn std::error::Error>>
//where F: FnOnce(&str) -> Result<(), Box<dyn std::error::Error>>
{
    println!("Init main entered");
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("i", "", "Identity", "zygote|sysfswalk");
    opts.optopt("k", "", "key", "secret key");
    opts.optopt("e", "", "executable", "executable name");
    opts.optflag("n", "", "not pid 1");

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
                if let Err(e) = launch_app(configs, others) {
                    panic!("Launch of {} failed due to {}", others, e);
                } else {
                    Ok(())
                }
            }
        }
    } else {
        info!("I N I T C E P T I O N");
        initception::initception_main_static(configs, !notpid1)
    }
}

// look for the application in the application configuration array and launch it
fn launch_app(
    configs: &[&dyn ApplicationConfig],
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(cfg) = configs.iter().find(|c| {
        println!("Searching:{}", c.name());

        c.name() == name
    }) {
        let params = CreateParams {
            // No create params for now
        };
        let run_params = RunParams {
            // No run params for now
        };
        let mut app = cfg.create(&params).unwrap();
        app.run(&run_params)
    } else {
        error!("Did not find application {}", name);
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            name,
        )))
    }
}
