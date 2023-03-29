// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::fields::{TransmissionChannelType, TransmissionGlobalDataPages};
use crate::messages::{AcknowledgedData, AntMessage, BroadcastData, RxMessageType, TxMessage};
use crate::plus::common::datapages::{ModeSettings, RequestDataPage, MANUFACTURER_SPECIFIC_RANGE};
use crate::plus::common::helpers::{MessageHandler, ProfileReference, TransmissionTypeAssignment};
use crate::plus::profiles::heart_rate::{
    BatteryStatus, Capabilities, CumulativeOperatingTime, DataPageNumbers, DefaultDataPage,
    DeviceInformation, ManufacturerInformation, ManufacturerSpecific, PreviousHeartBeat,
    ProductInformation, SwimIntervalSummary, DATA_PAGE_NUMBER_MASK,
};
use crate::plus::{duration_to_search_timeout, NETWORK_RF_FREQUENCY};
use crate::plus::{Channel, ChannelAssignment};

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
    msg_handler: MessageHandler,
    rx_message_callback: Option<fn(&AntMessage)>,
    rx_datapage_callback: Option<fn(Result<RxDataPages, HeartRateError>)>,
}

const HR_REFERENCE: ProfileReference = ProfileReference {
    // TODO type device_type
    device_type: 120,
    channel_type: TransmissionChannelType::IndependentChannel,
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

    pub fn send_datapage(&mut self, dp: TxDataPages, use_ack: bool) -> Result<(), HeartRateError> {
        if self.msg_handler.is_pending() {
            return Err(HeartRateError::PageAlreadyPending());
        }
        let channel = match self.msg_handler.get_channel() {
            ChannelAssignment::UnAssigned() => return Err(HeartRateError::NotAssociated()),
            ChannelAssignment::Assigned(channel) => channel,
        };
        let data = match dp {
            TxDataPages::RequestDataPage(rd) => {
                self.msg_handler
                    .set_sending(AcknowledgedData::new(channel, rd.pack()?).into());
                return Ok(());
            }
            TxDataPages::ModeSettings(ms) => ms.pack(),
            TxDataPages::ManufacturerSpecific(ms) => ms.pack(),
        }?;
        if use_ack {
            self.msg_handler
                .set_sending(AcknowledgedData::new(channel, data).into());
        } else {
            self.msg_handler
                .set_sending(BroadcastData::new(channel, data).into());
        }
        Ok(())
    }
}

impl Channel for HeartRateDisplay {
    fn receive_message(&mut self, msg: &AntMessage) {
        if let Some(f) = self.rx_message_callback {
            f(msg);
        }
        match msg.message {
            RxMessageType::BroadcastData(msg) => self.handle_dp(&msg.payload.data),
            RxMessageType::AcknowledgedData(msg) => self.handle_dp(&msg.payload.data),
            _ => (),
        }
        self.msg_handler.receive_message(msg);
    }

    fn send_message(&mut self) -> Option<TxMessage> {
        self.msg_handler.send_message()
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
}

impl From<packed_struct::PackingError> for HeartRateError {
    fn from(err: packed_struct::PackingError) -> Self {
        Self::BytePatternError(err)
    }
}
