// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ant::channel::mpsc::{RxChannel, TxChannel};
use ant::drivers::{is_ant_usb_device_from_device, UsbDriver};
use ant::messages::config::SetNetworkKey;
use ant::plus::profiles::heart_rate::{
    Capabilities, CommonData, Features, MainDataPage, ManufacturerInformation,
    ManufacturerSpecific, Monitor, MonitorConfig, PreviousHeartBeat, ProductInformation,
    TxDatapage,
};
use ant::router::Router;

// Needed for `pack` function calls
use dialoguer::Select;
use packed_struct::PackedStruct;
use rusb::{Device, DeviceList};

use std::sync::mpsc::channel;

// This function creates datapages for the purposes of this example, the generated datapages are
// not to spec in any form or fashion and are strictly here to show how generation works
fn make_datapage(dp: &TxDatapage) -> [u8; 8] {
    match dp {
        TxDatapage::ManufacturerInformation() => {
            ManufacturerInformation::new(false, 0xff, 0xffff, CommonData::new(1234, 123, 60)).pack()
        }
        TxDatapage::ManufacturerSpecific(x) => ManufacturerSpecific::new(
            (0x72 + x).into(),
            false,
            [0, 0, 0],
            CommonData::new(1234, 123, 60),
        )
        .pack(),
        TxDatapage::ProductInformation() => {
            ProductInformation::new(false, 0xff, 0xff, 0xff, CommonData::new(1234, 123, 60)).pack()
        }
        TxDatapage::PreviousHeartBeat() => {
            PreviousHeartBeat::new(false, 0xff, 0x01234, CommonData::new(1234, 123, 60)).pack()
        }
        TxDatapage::Capabilities() => Capabilities::new(
            false,
            Features::new(false, true, false, false, true, false),
            Features::new(false, true, false, false, false, false),
            CommonData::new(1234, 123, 60),
        )
        .pack(),
        _ => panic!("Datapage implementation missing"),
    }
    .unwrap()
}

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
            );
        let selection = selection.interact().expect("Invalid selection");
        devices.remove(selection)
    };

    let driver = UsbDriver::new(device).unwrap();

    let (router_tx, monitor_rx) = channel();
    let (monitor_tx, router_rx) = channel();

    let mut router = Router::new(
        driver,
        RxChannel {
            receiver: router_rx,
        },
    )
    .unwrap();
    let snk = SetNetworkKey::new(0, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]); // Get this from thisisant.com
    let channel = router
        .add_channel(TxChannel { sender: router_tx })
        .expect("Add channel failed");
    router.send(&snk).expect("failed to set network key");
    let config = MonitorConfig {
        device_number: 12345,
        transmission_type_extension: 12.into(),
        main_data_page: MainDataPage::PreviousHeartBeat,
        cumulative_operating_time_supported: false,
        battery_status_supported: true,
        swim_mode_supported: false,
        gym_mode_supported: false,
        number_manufacturer_pages: 2,
        background_page_interval: 64,
        ant_plus_key_index: 0,
        channel: channel,
    };
    let mut hr = Monitor::new(
        config,
        TxChannel { sender: monitor_tx },
        RxChannel {
            receiver: monitor_rx,
        },
        |x| println!("{:#?}", x),
        |datapage| make_datapage(datapage),
    );
    // hr.borrow_mut()
    //     .set_rx_datapage_callback(Some(|x| println!("{:#?}", x)));
    hr.set_rx_message_callback(Some(|x| println!("{:#?}", x)));
    hr.open();
    loop {
        router.process().unwrap();
        hr.process().unwrap();
    }
}
