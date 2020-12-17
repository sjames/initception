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





use ttrpc_codegen::Codegen;
use ttrpc_codegen::Customize;

fn main() {
    //let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_dir = "src/application/src_gen";
    let protos = vec!["src/protocols/application_interface.proto"];

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
