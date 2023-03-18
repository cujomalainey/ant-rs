// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ant::drivers::*;
use ant::messages::*;

use dialoguer::Select;
use rusb::{Device, DeviceList};

fn main() -> std::io::Result<()> {
    let mut devices: Vec<Device<_>> = DeviceList::new()
        .expect("Unable to lookup usb devices")
        .iter()
        .filter(|x| is_ant_usb_device_from_device(x))
        .collect();

    if devices.is_empty() {
        panic!("No devices found");
    }

    let device = if devices.len() == 1 {
        devices.remove(0)
    } else {
        let selection = Select::new()
            .with_prompt("Multiple devices found, please select a radio to use.")
            .items(
                &devices
                    .iter()
                    .map(|x| x.device_descriptor().unwrap())
                    .map(|x| format!("{:04x}:{:04x}", x.vendor_id(), x.product_id()))
                    .collect::<Vec<String>>(),
            )
            .interact()?;
        devices.remove(selection)
    };

    let usb_driver = UsbSerial::new(device).unwrap();

    let mut driver = SerialDriver::<_, StubPin>::new(usb_driver, None);
    let assign = AssignChannel::new(0, ChannelType::BidirectionalMaster, 0, None);
    // Skip setting the public key so we use th public channel by default
    let rf = ChannelRfFrequency::new(0, 23);
    let id = ChannelId::new(
        0,
        123,
        DeviceType::new(89.into(), false),
        TransmissionType::new(
            TransmissionChannelType::IndependentChannel,
            TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
            0xF.into(),
        ),
    );
    let period = ChannelPeriod::new(0, 1000);
    let mut data = BroadcastData::new(0, [0, 0, 0, 0, 0, 0, 0, 0]);
    driver
        .send_message(&ResetSystem::new())
        .expect("Failed to reset device");
    driver.send_message(&assign).expect("Message failed");
    driver.send_message(&id).expect("Message failed");
    driver.send_message(&period).expect("Message failed");
    driver.send_message(&rf).expect("Message failed");
    driver
        .send_message(&OpenChannel::new(0))
        .expect("Message failed");
    driver.send_message(&data).expect("Message failed");
    loop {
        match driver.get_message() {
            Ok(None) => (),
            Ok(Some(msg)) => match &msg.message {
                RxMessageType::ChannelEvent(msg) => match msg.payload.message_code {
                    MessageCode::EventTx => {
                        data.payload.data[0] = data.payload.data[0].overflowing_add(1).0;
                        println!("Sending [0][0][0][0][0][0][0][{}]!", data.payload.data[0]);
                        driver.send_message(&data).expect("Message failed");
                    }
                    evt => println!("Event {:#?}", evt),
                },
                msg => println!("Got: {:#?}", msg),
            },
            msg => println!("Error: {:#?}", msg),
        }
    }
}
