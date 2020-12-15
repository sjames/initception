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

// Not sure why we need this. It may be useful to launch some
// early applications
extern crate caps;

use std::error::Error;

pub fn zygote_main(key: Option<String>) -> Result<(), Box<Error>> {
    println!("Zygote started");
    Ok(())
}

pub fn launch_zygote() -> Result<(), Box<dyn Error>> {
    println!("Launch zygote");
    let path = std::fs::read_link("/proc/self/exe").expect("Unable to read /proc/self/exe");

    println!("Going to exec {:?}", &path);

    let cmd = std::process::Command::new(path)
        .arg("-i")
        .arg("zygote")
        .arg("-k")
        .arg("key")
        .spawn();

    match cmd {
        Err(e) => return Err(Box::new(e)),
        Ok(child) => println!("Zygote launched PID {}", child.id()),
    }

    Ok(())
}
