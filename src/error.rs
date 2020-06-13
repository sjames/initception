use failure::Fail;

#[derive(Debug, Fail)]
pub enum InitceptionClientError {
    #[fail(display = "No SOCKET address:")]
    NoSocketAddress {},
    #[fail(display = "Error connecting to init process:")]
    ConnectionError {},
}

#[derive(Debug, Fail)]
pub enum InitceptionServerError {
    #[fail(display = "Connection to process failed")]
    ConnectionFailed {},
}
