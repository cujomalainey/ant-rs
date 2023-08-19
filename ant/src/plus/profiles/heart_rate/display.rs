// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::channel::{duration_to_search_timeout, Channel, ChannelAssignment};
use crate::messages::config::{ChannelType, TransmissionChannelType, TransmissionGlobalDataPages};
use crate::messages::{AntMessage, RxMessage, TxMessage, TxMessageChannelConfig, TxMessageData};
use crate::plus::common::datapages::{ModeSettings, RequestDataPage, MANUFACTURER_SPECIFIC_RANGE};
use crate::plus::common::helpers::StateError;
use crate::plus::common::helpers::{MessageHandler, ProfileReference, TransmissionTypeAssignment};
use crate::plus::profiles::heart_rate::{
    BatteryStatus, Capabilities, CumulativeOperatingTime, DataPageNumbers, DefaultDataPage,
    DeviceInformation, ManufacturerInformation, ManufacturerSpecific, PreviousHeartBeat,
    ProductInformation, SwimIntervalSummary, DATA_PAGE_NUMBER_MASK,
};
use crate::plus::NETWORK_RF_FREQUENCY;

use packed_struct::{PackedStruct, PrimitiveEnum};

use std::time::Duration;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RxDataPages {
    DefaultDataPage(DefaultDataPage),
    CumulativeOperatingTime(CumulativeOperatingTime),
    ManufacturerInformation(ManufacturerInformation),
    ProductInformation(ProductInformation),
    PreviousHeartBeat(PreviousHeartBeat),
    SwimIntervalSummary(SwimIntervalSummary),
    Capabilities(Capabilities),
    BatteryStatus(BatteryStatus),
    DeviceInformation(DeviceInformation),
    ManufacturerSpecific(ManufacturerSpecific),
}

pub enum TxDataPages {
    RequestDataPage(RequestDataPage),
    ModeSettings(ModeSettings),
    ManufacturerSpecific(ManufacturerSpecific),
}

pub struct HeartRateDisplay {
    msg_handler: MessageHandler<'static>,
    rx_message_callback: Option<fn(&AntMessage)>,
    rx_datapage_callback: Option<fn(Result<RxDataPages, HeartRateError>)>,
    tx_message_callback: Option<fn() -> Option<TxMessageChannelConfig>>,
    tx_datapage_callback: Option<fn() -> Option<TxMessageData>>,
}

const HR_REFERENCE: ProfileReference = ProfileReference {
    device_type: 120,
    channel_type: ChannelType::BidirectionalSlave,
    transmission_channel_type: TransmissionChannelType::IndependentChannel,
    global_datapages_used: TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
    radio_frequency: NETWORK_RF_FREQUENCY,
    timeout_duration: duration_to_search_timeout(Duration::from_secs(30)),
    channel_period: 8070,
};

impl HeartRateDisplay {
    pub fn new(device: Option<(u16, TransmissionTypeAssignment)>, ant_plus_key_index: u8) -> Self {
        let device = device.unwrap_or((0, TransmissionTypeAssignment::Wildcard()));
        let device_number = device.0;
        let transmission_type = device.1;
        Self {
            rx_message_callback: None,
            rx_datapage_callback: None,
            tx_message_callback: None,
            tx_datapage_callback: None,
            msg_handler: MessageHandler::new(
                device_number,
                transmission_type,
                ant_plus_key_index,
                &HR_REFERENCE,
            ),
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

    pub fn set_rx_datapage_callback(&mut self, f: Option<fn(Result<RxDataPages, HeartRateError>)>) {
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

    fn parse_dp(&mut self, data: &[u8; 8]) -> Result<RxDataPages, HeartRateError> {
        let dp_num = data[0] & DATA_PAGE_NUMBER_MASK;
        if let Some(dp) = DataPageNumbers::from_primitive(dp_num) {
            let parsed = match dp {
                DataPageNumbers::DefaultDataPage => {
                    RxDataPages::DefaultDataPage(DefaultDataPage::unpack(data)?)
                }
                DataPageNumbers::CumulativeOperatingTime => {
                    RxDataPages::CumulativeOperatingTime(CumulativeOperatingTime::unpack(data)?)
                }
                DataPageNumbers::ManufacturerInformation => {
                    RxDataPages::ManufacturerInformation(ManufacturerInformation::unpack(data)?)
                }
                DataPageNumbers::ProductInformation => {
                    RxDataPages::ProductInformation(ProductInformation::unpack(data)?)
                }
                DataPageNumbers::PreviousHeartBeat => {
                    RxDataPages::PreviousHeartBeat(PreviousHeartBeat::unpack(data)?)
                }
                DataPageNumbers::SwimIntervalSummary => {
                    RxDataPages::SwimIntervalSummary(SwimIntervalSummary::unpack(data)?)
                }
                DataPageNumbers::Capabilities => {
                    RxDataPages::Capabilities(Capabilities::unpack(data)?)
                }
                DataPageNumbers::BatteryStatus => {
                    RxDataPages::BatteryStatus(BatteryStatus::unpack(data)?)
                }
                DataPageNumbers::DeviceInformation => {
                    RxDataPages::DeviceInformation(DeviceInformation::unpack(data)?)
                }
                // Add all valid profile specific pages below if they are invalid in this direction
                DataPageNumbers::HRFeatureCommand => {
                    return Err(HeartRateError::UnsupportedDataPage(dp_num))
                }
            };
            return Ok(parsed);
        }
        if MANUFACTURER_SPECIFIC_RANGE.contains(&dp_num) {
            return Ok(RxDataPages::ManufacturerSpecific(
                ManufacturerSpecific::unpack(data)?,
            ));
        }
        Err(HeartRateError::UnsupportedDataPage(dp_num))
    }
}

impl Channel for HeartRateDisplay {
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
                if let Some(mut msg) = callback() {
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

#[derive(Debug, Clone)]
pub enum HeartRateError {
    BytePatternError(packed_struct::PackingError),
    UnsupportedDataPage(u8),
    PageAlreadyPending(),
    NotAssociated(),
    ConfigurationError(StateError),
}

impl From<packed_struct::PackingError> for HeartRateError {
    fn from(err: packed_struct::PackingError) -> Self {
        Self::BytePatternError(err)
    }
}

impl From<StateError> for HeartRateError {
    fn from(err: StateError) -> Self {
        Self::ConfigurationError(err)
    }
}
