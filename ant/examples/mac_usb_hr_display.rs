// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ant::drivers::{is_ant_usb_device_from_device, UsbDriver};
use ant::messages::config::SetNetworkKey;
use ant::plus::profiles::heart_rate::{Display, Period};
use ant::router::Router;
use dialoguer::Select;
use rusb::{Device, DeviceList};

use std::cell::RefCell;
use std::rc::Rc;

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

    let driver = UsbDriver::new(device).unwrap();

    let mut router = Router::new(driver).unwrap();
    let snk = SetNetworkKey::new(0, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]); // Get this from thisisant.com
    router.send(&snk).expect("failed to set network key");
    let hr = Rc::new(RefCell::new(Display::new(None, 0, Period::FourHz)));
    // hr.borrow_mut()
    //     .set_rx_datapage_callback(Some(|x| println!("{:#?}", x)));
    hr.borrow_mut()
        .set_rx_message_callback(Some(|x| println!("{:#?}", x)));
    router.add_channel(hr.clone()).expect("Add channel failed");
    hr.borrow_mut().open();
    loop {
        router.process().unwrap();
    }
}
