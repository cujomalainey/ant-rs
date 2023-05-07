// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ant::drivers::*;
use ant::messages::config::{
    AssignChannel, ChannelId, ChannelPeriod, ChannelRfFrequency, ChannelType, DeviceType,
    LibConfig, SetNetworkKey, TransmissionType,
};
use ant::messages::control::{OpenChannel, ResetSystem};
use rusb::{Device, DeviceList};

use dialoguer::Select;

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

    let mut driver = UsbDriver::new(device).unwrap();
    let assign = AssignChannel::new(0, ChannelType::BidirectionalSlave, 0, None);
    let key = SetNetworkKey::new(0, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]); // get this
                                                                                       // from
                                                                                       // thisisant.com
    let rf = ChannelRfFrequency::new(0, 57);
    let id = ChannelId::new(
        0,
        0,
        DeviceType::new(120.into(), false),
        TransmissionType::new_wildcard(),
    );
    let period = ChannelPeriod::new(0, 8070);
    let libconfig = LibConfig::new(true, true, true);
    driver
        .send_message(&ResetSystem::new())
        .expect("Failed to reset device");
    driver.send_message(&key).expect("Message failed");
    driver.send_message(&assign).expect("Message failed");
    driver.send_message(&id).expect("Message failed");
    driver.send_message(&period).expect("Message failed");
    driver.send_message(&rf).expect("Message failed");
    driver.send_message(&libconfig).expect("Message failed");
    driver
        .send_message(&OpenChannel::new(0))
        .expect("Message failed");
    loop {
        match driver.get_message() {
            Ok(None) => (),
            msg => println!("{:#?}", msg),
        }
    }
}
