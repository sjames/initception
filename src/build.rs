use std::fs::File;
use std::io::{Read, Write};
use ttrpc_codegen::Codegen;
use ttrpc_codegen::Customize;
use std::path::PathBuf;
use std::env;

fn main() {

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let protos = vec![
        "src/protocols/application_interface.proto",
    ];

    // Tell Cargo that if the .proto files changed, to rerun this build script.
    protos
        .iter()
        .for_each(|p| println!("cargo:rerun-if-changed={}", &p));

    Codegen::new()
        .out_dir(out_dir)
        .inputs(&protos)
        .include("src/protocols")
        .rust_protobuf()
        .customize(Customize {
            async_all: true,
            ..Default::default()
        })
        .run()
        .expect("Gen async code failed.");

}
