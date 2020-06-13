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
