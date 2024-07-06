use std::{
    collections::{HashMap, HashSet},
    error::Error,
    string,
};

use bluer::{
    Adapter, AdapterEvent, Address, Device, DeviceEvent, DiscoveryFilter, DiscoveryTransport,
};
use log;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    Config,
};

use futures::{pin_mut, stream::SelectAll, StreamExt};

mod cmd;

const FIDO_SERVICE_UUID: &str = "0000fffd-0000-1000-8000-00805f9b34fb";
const FIDO_CONTROL_POINT_UUID: &str = "f1d0fff1-deaa-ecee-b42f-c9ba7ed623bb";
const FIDO_STATUS_UUID: &str = "f1d0fff2-deaa-ecee-b42f-c9ba7ed623bb";
const FIDO_CONTROL_POINT_LENGTH_UUID: &str = "f1d0fff3-deaa-ecee-b42f-c9ba7ed623bb";
const FIDO_SERVICE_REVISION_BITFIELD_UUID: &str = "f1d0fff4-deaa-ecee-b42f-c9ba7ed623bb";

struct Driver {
    devices: HashMap<String, Device>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appender("stdout")
                .build(log::LevelFilter::Debug),
        )
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();

    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let filter = DiscoveryFilter {
        transport: DiscoveryTransport::Auto,
        ..Default::default()
    };
    adapter.set_discovery_filter(filter).await?;

    println!(
        "Using discovery filter:\n{:#?}\n\n",
        adapter.discovery_filter().await
    );

    let device_events = adapter.discover_devices().await?;
    pin_mut!(device_events);

    // let mut all_change_events = SelectAll::new();

    loop {
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        log::info!("Device added: {addr}");

                        let device = adapter.device(addr)?;
                        if device.is_paired().await? {
                            log::info!("Device is paired {addr}");

                            let device_uuids = device.uuids().await?;
                            let device_service_data = device.service_data().await?.unwrap_or_default();

                            let fido_service_uuid = bluer::Uuid::parse_str(FIDO_SERVICE_UUID).unwrap();
                            if device_uuids.unwrap_or_default().contains(&fido_service_uuid) {
                                log::info!("Found device uuid {addr}")
                            } else if device_service_data.contains_key(&fido_service_uuid) {
                                log::info!("Found by device data {addr}")
                            }

                            // let viking_names = HashSet::from(["Einar", "Olaf", "Harald"]);
                        }
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
                        log::info!("Device removed: {addr}");
                    }
                    _ => ()
                }
            }
            else => break
            // Some((addr, DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {}
        }
    }

    Ok(())
    // let device_events = adapter.discover_devices().await?;
}
