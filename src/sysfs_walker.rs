/// This process will walk the sysfs, writing into the "uevent" file to trigger uevents into the
/// init process.
use std::error::Error;
use std::io::Write;
use walkdir::WalkDir;

pub fn sysfs_walker_main(_key: Option<String>) -> Result<(), Box<Error>> {
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
