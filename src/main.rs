#![feature(custom_inner_attributes)]
use getopts::Options;
use std::env;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

extern crate initception;
use initception::sysfs_walker;
use initception::zygote;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!(".");

    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("i", "", "Identity", "zygote|sysfswalk");
    opts.optopt("k", "", "key", "secret key");
    opts.optopt("e","","executable","executable name");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    let key = matches.opt_str("k");
    let identity = matches.opt_str("i");

    if identity.is_some() {
        match identity.unwrap().as_ref() {
            "zygote" => zygote::zygote_main(key),
            "sysfswalk" => sysfs_walker::sysfs_walker_main(key),
            others => {
                initception::launcher::launch(others)
                /*
                error!("FATAL: Unknown identity for INITCEPTION");
                Err(Box::new(std::io::Error::from(
                    std::io::ErrorKind::InvalidInput,
                )))
                */
            }
        }
    } else {
        info!("I N I T C E P T I O N");
        initception::initception::initception_main()
    }
}
