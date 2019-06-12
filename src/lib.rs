extern crate nix;
use std::error::Error;

mod device;
use crate::device::{make_basic_devices, mount_basics};

pub struct Config {
    pub hostname: String,
    pub true_init: bool,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            let hostname = "localhost".to_string();
            Ok(Config {
                hostname: hostname,
                true_init: true,
            })
        } else {
            let hostname = args[1].clone();
            Ok(Config {
                hostname: hostname,
                true_init: false,
            })
        }
    }
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    if config.true_init {
        mount_basics()?;
        make_basic_devices()?;
    }

    Ok(())
}
