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

// Main entry point

#![feature(custom_inner_attributes)]
use getopts::Options;
use std::env;
use tracing::{info, Level};
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
    opts.optopt("e", "", "executable", "executable name");
    opts.optopt("n", "", "not pid 1", "Not launched as PID1 process");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    let key = matches.opt_str("k");
    let identity = matches.opt_str("i");
    let notpid1 = matches.opt_str("n");

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
        if notpid1.is_some() {
            initception::initception::initception_main(false)
        } else {
            // launch as PID1
            initception::initception::initception_main(true)
        }
    }
}
