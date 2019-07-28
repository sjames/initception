extern crate caps;
use crate::zygote::ZycoteClientContext;
use caps::Capability;

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

pub struct Context {
    pub config: Config, // The context will own the config
    pub zygote_client_context: Option<ZycoteClientContext>,
}

impl Context {
    pub fn new(config: Config) -> Result<Context, &'static str> {
        Ok(Context {
            config,
            zygote_client_context: None,
        })
    }
}

pub fn drop_caps(caps_to_drop: &[Capability]) -> Result<(), caps::errors::Error> {
    for cap in caps_to_drop.iter() {
        match caps::drop(None, caps::CapSet::Permitted, *cap) {
            Ok(()) => {}
            Err(e) => {
                println!("Unable to drop caps {:?} {:?}", e, *cap);
                return Err(e);
            }
        }
    }
    Ok(())
}
