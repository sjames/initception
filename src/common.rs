extern crate caps;
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
}

impl Context {
    pub fn new(config: Config) -> Result<Context, &'static str> {
        Ok(Context { config })
    }
}

pub fn drop_caps(caps_to_drop: &[Capability]) -> Result<(), &'static str> {
    for cap in caps_to_drop.iter() {
        if let Ok(()) = caps::drop(None, caps::CapSet::Permitted, *cap) {

        } else {
            panic!("Unable to drop caps");
        }
    }

    Ok(())
}
