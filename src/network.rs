/// Configure the network.
///
use crate::context::{ContextReference, RuntimeEntity, ServiceIndex, UnitStatus};
use crate::initrc::{Unit, UnitType};
use ipnetwork::{IpNetwork, Ipv4Network};
use rtnetlink::{new_connection, Error as RtNetlinkError, Handle};
use tokio::io::{self, AsyncBufReadExt, ReadHalf};
use tracing::{debug, error, info, warn};

pub async fn configure_network(
    context: ContextReference,
    index: ServiceIndex,
) -> std::result::Result<(), ()> {
    let ctx = context.read().unwrap();
    let spawnref = ctx.get_ref(&index);
    // we need to write the status into the unit, so we get create a write lock

    let mut entity: &mut RuntimeEntity = &mut spawnref.write().unwrap();
    if let RuntimeEntity::Unit(unit) = &mut entity {
        if let Ok(()) = config_network_from_unit(&unit.unit) {
            //debug!("Mount successful");
            unit.status = UnitStatus::Success;
            Ok(())
        } else {
            unit.status = UnitStatus::Error;
            error!("Unable to mount");
            Err(())
        }
    } else {
        panic!("Only units can have network information");
    }
}

/// Return the IP network and the interface in the
fn parse_params(params: &str) -> Result<IpNetwork, ()> {
    let mut ipstr = None;
    for param in params.split(',') {
        let keyval: Vec<&str> = param.split('=').collect();
        if keyval.len() == 2 {
            match keyval[0] {
                "ip" => ipstr = Some(keyval[1]),
                _ => warn!("Ignoring unknown parameter"),
            }
        } else {
            warn!("Ignoring malformed param string");
        }
    }
    // try to parse the IP
    if let Some(ip) = ipstr {
        if let Ok(ip) = ip.parse::<IpNetwork>() {
            Ok(ip)
        } else {
            error!("Unable to parse IP string : {}", ipstr.unwrap());
            Err(())
        }
    } else {
        Err(())
    }
}

fn config_network_from_unit(unit: &Unit) -> Result<(), ()> {
    if let UnitType::Net = unit.r#type {
        if let Some(params) = &unit.params {
            if let Ok(ip) = parse_params(&params) {
                configure_network_interface(ip, String::from(&unit.device))
            } else {
                Err(())
            }
        } else {
            error!("Params not set");
            Err(())
        }
    } else {
        panic!("Attempt to set network on a wrong unit type");
    }
}

pub fn configure_network_interface(ip: IpNetwork, interface: String) -> Result<(), ()> {
    debug!("Configure network interface {}", &interface);
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    let task = tokio::spawn(async move {
        let mut links = handle
            .link()
            .get()
            .set_name_filter(String::from(&interface))
            .execute();
        if let Ok(Some(link)) = futures::stream::TryStreamExt::try_next(&mut links).await {
            if let Ok(()) = handle
                .address()
                .add(link.header.index, ip.ip(), ip.prefix())
                .execute()
                .await
            {
                if let Ok(()) = handle.link().set(link.header.index).up().execute().await {
                    info!("{} network device is up", &interface);
                    let mut address = handle
                        .address()
                        .get()
                        .set_link_index_filter(link.header.index)
                        .execute();

                    while let Ok(Some(msg)) =
                        futures::stream::TryStreamExt::try_next(&mut address).await
                    {
                        debug!("Address: {:?}", msg);
                    }
                    Ok(())
                } else {
                    error!("Unable to bring up network interface {}", &interface);
                    Err(())
                }
            } else {
                error!("Unable to set ip to {}", &interface);
                Err(())
            }
        } else {
            debug!("No link from try_next");
            Err(())
        }
    });
    Ok(())
}
