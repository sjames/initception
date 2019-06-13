extern crate nix;
use std::error::Error;
extern crate ipc_channel;

pub mod common;
mod device;
mod zygote;

use crate::common::*;
use crate::device::{make_basic_devices, mount_basics};
use crate::zygote::run_zygote;
// ipc_channel for communicating with zygote

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut context = Context::new(config).unwrap();

    if context.config.true_init {
        mount_basics()?;
        make_basic_devices()?;
    }

    run_zygote(&mut context);

    Ok(())
}
