use std::env;
use std::process;

fn main() {
    println!("INITCEPTION");
    let args: Vec<String> = env::args().collect();

    let config = initception::common::Config::new(&args).unwrap_or_else(|err| {
        println!("Unable to parse command line arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = initception::run(config) {
        println!("Application error: {}", e);
    }
}
