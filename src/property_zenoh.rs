use zenoh::net::*;

pub struct Properties {
    session : Session,
}

impl Properties {

    pub async fn new() -> Self {
        let config = zenoh::net::config::ConfigProperties::default();
     
        let prop_server = if let Ok(session) = open(config).await {
            Properties {
                session,
            }
        } else {
            panic!("Unable to create session");
        };

        prop_server
    }
}
