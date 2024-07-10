use std::{collections::HashMap, fs, process::Output, result};

use bluer::{AdapterEvent, Device, DiscoveryFilter, DiscoveryTransport};
use log;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    Config,
};

use futures::{pin_mut, StreamExt};
use tokio::sync::mpsc::Receiver;
use uhid_virt::{Bus, CreateParams, InputEvent, UHIDDevice};

const FIDO_SERVICE_UUID: &str = "0000fffd-0000-1000-8000-00805f9b34fb";
const FIDO_CONTROL_POINT_UUID: &str = "f1d0fff1-deaa-ecee-b42f-c9ba7ed623bb";
const FIDO_STATUS_UUID: &str = "f1d0fff2-deaa-ecee-b42f-c9ba7ed623bb";
const FIDO_CONTROL_POINT_LENGTH_UUID: &str = "f1d0fff3-deaa-ecee-b42f-c9ba7ed623bb";
const FIDO_SERVICE_REVISION_BITFIELD_UUID: &str = "f1d0fff4-deaa-ecee-b42f-c9ba7ed623bb";

mod hid;

const RDESC: [u8; 34] = [
    0x06, 0xD0, 0xF1, // Usage Page (FIDO alliance HID usage page)
    0x09, 0x01, // Usage (U2FHID usage for top-level collection)
    0xA1, 0x01, // Collection (Application)
    0x09, 0x20, // Usage (Raw IN data report)
    0x15, 0x00, // Logical Minimum (0)
    0x26, 0xFF, 0x00, // Logical Maximum (255)
    0x75, 0x08, // Report Size (8)
    0x95, 0x40, // Report Count (64)
    0x81, 0x02, // Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x09, 0x21, // Usage (Raw OUT data report)
    0x15, 0x00, // Logical Minimum (0)
    0x26, 0xFF, 0x00, // Logical Maximum (255)
    0x75, 0x08, // Report Size (8)
    0x95, 0x40, // Report Count (64)
    0x91,
    0x02, // Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0xC0, // End Collection
];

struct Instance {
    ble_device: Device,
    uhid_device: UHIDDevice<fs::File>,
}

impl Instance {
    fn new(ble_device: Device) -> Instance {
        let rd_data = RDESC.to_vec();

        let create_params = CreateParams {
            name: "PONE Fido2BLE Proxy".to_string(),
            phys: "test_device".to_string(),
            uniq: "".to_string(),
            bus: Bus::USB,
            vendor: 0xAAAA,
            product: 0xAAAA,
            version: 0,
            country: 0,
            rd_data,
        };

        let uhid_device = UHIDDevice::create(create_params).unwrap();

        Self {
            ble_device,
            uhid_device,
        }
    }

    async fn receive_data(mut self) {
        loop {
            let result = self.uhid_device.read();

            match result {
                Ok(event) => match event {
                    uhid_virt::OutputEvent::Output { data } => {
                        log::info!("Received packet");
                        let packet = hid::packet::Packet::new(&data);
                    }
                    _ => {}
                },
                Err(_) => break,
            }
        }
    }
}

struct Driver {
    instances: HashMap<String, Instance>,
}

impl Driver {
    fn new() -> Self {
        return Self {
            instances: HashMap::new(),
        };
    }

    fn handle_device(&mut self, device: Device) {
        let addr = device.address().to_string();

        if !self.instances.contains_key(&addr) {
            log::info!("Device isn't active, starting UHID");

            let inst = Instance::new(device);

            tokio::spawn(inst.receive_data());
            // Track the instance until we are done or dev is gone?
            self.instances.insert(addr, inst);
        }
    }

    pub fn unregister_device(&mut self, res: bool) {}

    async fn run(&mut self) -> bluer::Result<()> {
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

        loop {
            tokio::select! {
                Some(device_event) = device_events.next() => {
                    match device_event {
                        AdapterEvent::DeviceAdded(addr) => {
                            log::info!("Device added: {addr}");

                            let device: Device = adapter.device(addr)?;
                            if device.is_paired().await? {
                                log::info!("Device is paired {addr}");

                                let device_uuids = device.uuids().await?;
                                let device_service_data = device.service_data().await?.unwrap_or_default();

                                let fido_service_uuid = bluer::Uuid::parse_str(FIDO_SERVICE_UUID).unwrap();

                                if device_uuids.unwrap_or_default().contains(&fido_service_uuid) {
                                    log::info!("Found device by uuid {addr}");
                                    self.handle_device(device);
                                } else if device_service_data.contains_key(&fido_service_uuid) {
                                    log::info!("Found device by data {addr}");
                                    self.handle_device(device);
                                }
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
    }
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

    // let mut all_change_events = SelectAll::new();
    let mut driver = Driver::new();
    let result = driver.run().await;

    return result;

    // let device_events = adapter.discover_devices().await?;
}
