use std::env;
use std::error::Error;
use std::os::unix::io::FromRawFd;
use std::os::unix::net::UnixStream;

fn get_fd(env_variable_key: &str) -> Result<UnixStream, Box<dyn Error>> {
    if let Ok(address) = env::var(env_variable_key) {
        let raw_fd: i32 = address
            .parse()
            .expect(&format!("Malformed socket number in {}", env_variable_key));
        let std_stream = unsafe { UnixStream::from_raw_fd(raw_fd) };

        Ok(std_stream)
    } else {
        Err(Box::new(std::io::Error::from(
            std::io::ErrorKind::AddrNotAvailable,
        )))
    }
}

pub fn get_client_fd() -> Result<UnixStream, Box<dyn Error>> {
    get_fd("NOTIFY_FD")
}

pub fn get_server_fd() -> Result<UnixStream, Box<dyn Error>> {
    get_fd("NOTIFY_FD")
}
