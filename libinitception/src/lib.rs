pub mod app;
pub mod config;
pub mod initrc;
pub mod application_events;
mod src_gen;

pub use src_gen::application_interface;
pub use src_gen::application_interface_ttrpc::ApplicationServiceClient;
pub use src_gen::application_interface_ttrpc::{
    ApplicationManager,
    LifecycleServer,
    create_application_manager, 
    create_lifecycle_server
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
