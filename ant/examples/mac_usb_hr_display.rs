// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ant::drivers::{SerialDriver, StubPin};
use ant::plus::profiles::heart_rate::HeartRateDisplay;
use ant::plus::router::*;
use ant::usb::{is_ant_usb_device_from_device, UsbSerial};
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

    let usb_driver = UsbSerial::new(device).unwrap();

    let driver = SerialDriver::<_, StubPin>::new(usb_driver, None);
    let router = Router::new(driver).unwrap();
    let hr = HeartRateDisplay::new(None);
    router
        .borrow_mut()
        .set_key(
            NetworkKey::AntPlusKey,
            &[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77], // Get this from thisisant.com
        )
        .expect("set key failed");
    router
        .borrow_mut()
        .add_channel(hr.clone())
        .expect("Add channel failed");
    hr.borrow().open().expect("Open channel failed");
    loop {
        router.borrow().process().unwrap();
    }
}
