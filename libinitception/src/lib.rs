pub mod app;
pub mod application_events;
pub mod config;
pub mod initrc;
//mod src_gen;
pub mod app_interface;

//pub use src_gen::application_interface;
//pub use src_gen::application_interface_ttrpc::ApplicationServiceClient;
//pub use src_gen::application_interface_ttrpc::{
//    create_application_manager, create_lifecycle_server, ApplicationManager, LifecycleServer,
//};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
