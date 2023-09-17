// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::channel::{duration_to_search_timeout, Channel, ChannelAssignment};
use crate::messages::config::{
    ChannelType, TransmissionChannelType, TransmissionGlobalDataPages, TransmissionType,
};
use crate::messages::{AntMessage, RxMessage, TxMessage, TxMessageChannelConfig, TxMessageData};
use crate::plus::common::datapages::{
    DataPageNumbers as CommonDataPageNumbers, ModeSettings, RequestDataPage,
    MANUFACTURER_SPECIFIC_RANGE,
};
use crate::plus::common::msg_handler::{ChannelConfig, MessageHandler};
use crate::plus::profiles::heart_rate::{
    DataPageNumbers, DisplayTxDataPages, Error, HRFeatureCommand, ManufacturerSpecific,
    MonitorTxDataPages, Period, DATA_PAGE_NUMBER_MASK,
};
use crate::plus::NETWORK_RF_FREQUENCY;

use packed_struct::prelude::{packed_bits::Bits, Integer};
use packed_struct::{PackedStruct, PrimitiveEnum};

use std::time::Duration;

pub struct Monitor {
    msg_handler: MessageHandler,
    rx_message_callback: Option<fn(&AntMessage)>,
    rx_datapage_callback: Option<fn(Result<DisplayTxDataPages, Error>)>,
    tx_message_callback: Option<fn() -> Option<TxMessageChannelConfig>>,
    tx_datapage_callback: Option<fn(&MonitorTxDataPages) -> Option<TxMessageData>>,
}

pub struct HrMonitorConfig {
    device_number: u16,
    transmission_type_extension: Integer<u8, Bits<4>>,
    channel_period: Period,
}

impl Monitor {
    pub fn new(config: HrMonitorConfig, ant_plus_key_index: u8) -> Self {
        Self {
            rx_message_callback: None,
            rx_datapage_callback: None,
            tx_message_callback: None,
            tx_datapage_callback: None,
            msg_handler: MessageHandler::new(&ChannelConfig {
                device_number: 56, // TODO set from config
                device_type: 120,
                channel_type: ChannelType::BidirectionalMaster,
                network_key_index: ant_plus_key_index,
                transmission_type: TransmissionType::new(
                    TransmissionChannelType::IndependentChannel,
                    TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
                    5.into(),
                ), // TODO set from config
                radio_frequency: NETWORK_RF_FREQUENCY,
                timeout_duration: duration_to_search_timeout(Duration::from_secs(30)),
                channel_period: Period::FourHz.into(), // Monitor always uses 4Hz, display may use
                                                       // less
            }),
        }
    }

    pub fn open(&mut self) {
        self.msg_handler.open();
    }

    pub fn close(&mut self) {
        self.msg_handler.close();
    }

    pub fn get_device_id(&self) -> u16 {
        self.msg_handler.get_device_id()
    }

    pub fn set_rx_message_callback(&mut self, f: Option<fn(&AntMessage)>) {
        self.rx_message_callback = f;
    }

    pub fn set_rx_datapage_callback(&mut self, f: Option<fn(Result<DisplayTxDataPages, Error>)>) {
        self.rx_datapage_callback = f;
    }

    pub fn reset_state(&mut self) {
        // TODO
    }

    // get result and call callback
    fn handle_dp(&mut self, data: &[u8; 8]) {
        let dp = self.parse_dp(data);
        if let Some(f) = self.rx_datapage_callback {
            f(dp);
        }
    }

    fn parse_dp(&mut self, data: &[u8; 8]) -> Result<DisplayTxDataPages, Error> {
        let dp_num = data[0] & DATA_PAGE_NUMBER_MASK;
        if let Some(dp) = DataPageNumbers::from_primitive(dp_num) {
            return Ok(match dp {
                DataPageNumbers::HRFeatureCommand => {
                    DisplayTxDataPages::HRFeatureCommand(HRFeatureCommand::unpack(data)?)
                }
                // Add all valid profile specific pages below if they are invalid in this direction
                DataPageNumbers::DefaultDataPage
                | DataPageNumbers::CumulativeOperatingTime
                | DataPageNumbers::ManufacturerInformation
                | DataPageNumbers::ProductInformation
                | DataPageNumbers::PreviousHeartBeat
                | DataPageNumbers::SwimIntervalSummary
                | DataPageNumbers::Capabilities
                | DataPageNumbers::BatteryStatus
                | DataPageNumbers::DeviceInformation => {
                    return Err(Error::UnsupportedDataPage(dp_num))
                }
            });
        }
        if let Some(dp) = CommonDataPageNumbers::from_primitive(dp_num) {
            return Ok(match dp {
                CommonDataPageNumbers::ModeSettings => {
                    DisplayTxDataPages::ModeSettings(ModeSettings::unpack(data)?)
                }
                CommonDataPageNumbers::RequestDataPage => {
                    // TODO handle properly into cycle
                    DisplayTxDataPages::RequestDataPage(RequestDataPage::unpack(data)?)
                }
                _ => return Err(Error::UnsupportedDataPage(dp_num)),
            });
        }
        if MANUFACTURER_SPECIFIC_RANGE.contains(&dp_num) {
            return Ok(DisplayTxDataPages::ManufacturerSpecific(
                ManufacturerSpecific::unpack(data)?,
            ));
        }
        Err(Error::UnsupportedDataPage(dp_num))
    }

    fn get_next_datapage(&mut self) -> MonitorTxDataPages {
        todo!();
    }
}

impl Channel for Monitor {
    fn receive_message(&mut self, msg: &AntMessage) {
        if let Some(f) = self.rx_message_callback {
            f(msg);
        }
        match msg.message {
            RxMessage::BroadcastData(msg) => self.handle_dp(&msg.payload.data),
            RxMessage::AcknowledgedData(msg) => self.handle_dp(&msg.payload.data),
            _ => (),
        }
        match self.msg_handler.receive_message(msg) {
            Ok(_) => (),
            Err(e) => {
                if let Some(f) = self.rx_datapage_callback {
                    f(Err(e.into()));
                }
            }
        }
    }

    fn send_message(&mut self) -> Option<TxMessage> {
        let msg = self.msg_handler.send_message();
        if msg.is_some() {
            return msg;
        }
        let channel = if let ChannelAssignment::Assigned(channel) = self.msg_handler.get_channel() {
            channel
        } else {
            return None;
        };
        if let Some(callback) = self.tx_message_callback {
            if let Some(mut msg) = callback() {
                msg.set_channel(channel);
                return Some(msg.into());
            }
        }
        if self.msg_handler.is_tx_ready() {
            if let Some(callback) = self.tx_datapage_callback {
                let dp = self.get_next_datapage();
                if let Some(mut msg) = callback(&dp) {
                    msg.set_channel(channel);
                    self.msg_handler.tx_sent();
                    return Some(msg.into());
                }
            }
        }
        None
    }

    fn set_channel(&mut self, channel: ChannelAssignment) {
        self.msg_handler.set_channel(channel);
    }
}
