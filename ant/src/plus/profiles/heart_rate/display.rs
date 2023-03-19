// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::fields::{ChannelType, DeviceType, TransmissionType, Wildcard};
use crate::messages::{
    AntMessage, AssignChannel, ChannelId, ChannelPeriod, ChannelRfFrequency, CloseChannel,
    OpenChannel, RxMessageType, SearchTimeout, TxMessage,
};
use crate::plus::common_datapages::MANUFACTURER_SPECIFIC_RANGE;
use crate::plus::profiles::heart_rate::{
    BatteryStatus, Capabilities, CumulativeOperatingTime, DataPageNumbers, DefaultDataPage,
    DeviceInformation, ManufacturerInformation, ManufacturerSpecific, PreviousHeartBeat,
    ProductInformation, SwimIntervalSummary, DATA_PAGE_NUMBER_MASK,
};

use crate::plus::router::{Channel, ChannelAssignment};
use crate::plus::{duration_to_search_timeout, NETWORK_RF_FREQUENCY};
use packed_struct::{PackedStruct, PrimitiveEnum};

use std::time::Duration;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum DataPages {
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

enum ConfigureState {
    Assign,
    Period,
    Id,
    Rf,
    Timeout,
    Done,
}

enum ChannelStateCommand {
    Open,
    Close,
}

pub struct HeartRateDisplay {
    channel: ChannelAssignment,
    device_number: u16,
    transmission_type: TransmissionType,
    network_key_index: u8,
    rx_message_callback: Option<fn(&AntMessage)>,
    rx_datapage_callback: Option<fn(Result<DataPages, HeartRateError>)>,
    configure_state: ConfigureState,
    set_channel_state: Option<ChannelStateCommand>,
}

impl HeartRateDisplay {
    pub fn new(device: Option<(u16, Option<TransmissionType>)>, ant_plus_key_index: u8) -> Self {
        let device = device.unwrap_or((0, None));
        let device_number = device.0;
        let transmission_type = device.1.unwrap_or_else(TransmissionType::new_wildcard);
        Self {
            channel: ChannelAssignment::UnAssigned(),
            device_number,
            transmission_type,
            network_key_index: ant_plus_key_index,
            rx_message_callback: None,
            rx_datapage_callback: None,
            configure_state: ConfigureState::Assign,
            set_channel_state: None,
        }
    }

    pub fn open(&mut self) {
        self.set_channel_state = Some(ChannelStateCommand::Open);
    }

    pub fn close(&mut self) {
        self.set_channel_state = Some(ChannelStateCommand::Close);
    }

    pub fn set_rx_message_callback(&mut self, f: Option<fn(&AntMessage)>) {
        self.rx_message_callback = f;
    }

    pub fn set_rx_datapage_callback(&mut self, f: Option<fn(Result<DataPages, HeartRateError>)>) {
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

    fn parse_dp(&mut self, data: &[u8; 8]) -> Result<DataPages, HeartRateError> {
        let dp_num = data[0] & DATA_PAGE_NUMBER_MASK;
        if let Some(dp) = DataPageNumbers::from_primitive(dp_num) {
            let parsed = match dp {
                DataPageNumbers::DefaultDataPage => {
                    DataPages::DefaultDataPage(DefaultDataPage::unpack(data)?)
                }
                DataPageNumbers::CumulativeOperatingTime => {
                    DataPages::CumulativeOperatingTime(CumulativeOperatingTime::unpack(data)?)
                }
                DataPageNumbers::ManufacturerInformation => {
                    DataPages::ManufacturerInformation(ManufacturerInformation::unpack(data)?)
                }
                DataPageNumbers::ProductInformation => {
                    DataPages::ProductInformation(ProductInformation::unpack(data)?)
                }
                DataPageNumbers::PreviousHeartBeat => {
                    DataPages::PreviousHeartBeat(PreviousHeartBeat::unpack(data)?)
                }
                DataPageNumbers::SwimIntervalSummary => {
                    DataPages::SwimIntervalSummary(SwimIntervalSummary::unpack(data)?)
                }
                DataPageNumbers::Capabilities => {
                    DataPages::Capabilities(Capabilities::unpack(data)?)
                }
                DataPageNumbers::BatteryStatus => {
                    DataPages::BatteryStatus(BatteryStatus::unpack(data)?)
                }
                DataPageNumbers::DeviceInformation => {
                    DataPages::DeviceInformation(DeviceInformation::unpack(data)?)
                }
                // Add all valid pages below if they are invalid in this direction
                DataPageNumbers::HRFeatureCommand => {
                    return Err(HeartRateError::UnsupportedDataPage(dp_num))
                }
            };
            return Ok(parsed);
        }
        if MANUFACTURER_SPECIFIC_RANGE.contains(&dp_num) {
            return Ok(DataPages::ManufacturerSpecific(
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
            RxMessageType::BroadcastData(msg) => self.handle_dp(&msg.payload.data),
            RxMessageType::AcknowledgedData(msg) => self.handle_dp(&msg.payload.data),
            _ => (),
        }
    }

    fn send_message(&mut self) -> Option<TxMessage> {
        let channel = match self.channel {
            ChannelAssignment::UnAssigned() => return None,
            ChannelAssignment::Assigned(channel) => channel,
        };

        match self.configure_state {
            ConfigureState::Assign => {
                self.configure_state = ConfigureState::Period;
                return Some(TxMessage::AssignChannel(AssignChannel::new(
                    channel,
                    ChannelType::BidirectionalSlave,
                    self.network_key_index,
                    None,
                )));
            }
            ConfigureState::Period => {
                self.configure_state = ConfigureState::Id;
                // TODO const the period
                return Some(TxMessage::ChannelPeriod(ChannelPeriod::new(channel, 8070)));
            }
            ConfigureState::Id => {
                self.configure_state = ConfigureState::Rf;
                return Some(TxMessage::ChannelId(ChannelId::new(
                    channel,
                    self.device_number,
                    DeviceType::new(120.into(), false),
                    self.transmission_type,
                ))); // TODO type devicetype
            }
            ConfigureState::Rf => {
                self.configure_state = ConfigureState::Timeout;
                return Some(TxMessage::ChannelRfFrequency(ChannelRfFrequency::new(
                    channel,
                    NETWORK_RF_FREQUENCY,
                )));
            }
            ConfigureState::Timeout => {
                self.configure_state = ConfigureState::Done;
                return Some(TxMessage::SearchTimeout(SearchTimeout::new(
                    channel,
                    duration_to_search_timeout(Duration::from_secs(30)),
                )));
            }
            ConfigureState::Done => (),
        }
        if let Some(command) = &self.set_channel_state {
            let msg = match command {
                ChannelStateCommand::Open => TxMessage::OpenChannel(OpenChannel::new(channel)),
                ChannelStateCommand::Close => TxMessage::CloseChannel(CloseChannel::new(channel)),
            };
            self.set_channel_state = None;
            return Some(msg);
        }
        None
    }

    fn set_channel(&mut self, channel: ChannelAssignment) {
        self.channel = channel;
        self.configure_state = ConfigureState::Assign;
    }
}

// TODO extend to user errors
#[derive(Debug, Clone)]
pub enum HeartRateError {
    Bleh,
    BytePatternError(packed_struct::PackingError),
    UnsupportedDataPage(u8),
}

impl From<packed_struct::PackingError> for HeartRateError {
    fn from(err: packed_struct::PackingError) -> Self {
        Self::BytePatternError(err)
    }
}
