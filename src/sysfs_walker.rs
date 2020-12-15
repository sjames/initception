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

/// This process will walk the sysfs, writing into the "uevent" file to trigger uevents into the
/// init process.
use std::error::Error;
use std::io::Write;
use walkdir::WalkDir;

pub fn sysfs_walker_main(_key: Option<String>) -> Result<(), Box<dyn Error>> {
    println!("Sysfs walker started");
    let uevent = std::ffi::OsStr::new("uevent");
    for entry in WalkDir::new("/sys/devices")
        .into_iter()
        .filter_map(|e| e.ok())
    {
        //println!("{}", entry.path().display());

        if entry.file_name() == uevent {
            //println!("Found uevent");
            if let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(entry.path()) {
                if let Ok(_written) = file.write("add\n".as_bytes()) {
                } else {
                    println!("Cannot write into uevent");
                }
            }
        }
    }

    Ok(())
}

pub fn launch_sysfs_walker() -> Result<(), Box<dyn Error>> {
    println!("Launch sysfs walker");
    let path = std::fs::read_link("/proc/self/exe").expect("Unable to read /proc/self/exe");

    println!("Going to exec {:?}", &path);

    let cmd = std::process::Command::new(path)
        .arg("-i")
        .arg("sysfswalk")
        .arg("-k")
        .arg("key")
        .spawn();

    match cmd {
        Err(e) => return Err(Box::new(e)),
        Ok(child) => println!("Sysfs walker launched PID {}", child.id()),
    }

    Ok(())
}
