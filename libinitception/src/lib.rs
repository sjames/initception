pub mod app;
pub mod application_events;
pub mod config;
pub mod initrc;
pub mod app_interface;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
