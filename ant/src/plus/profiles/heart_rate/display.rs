// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::drivers::Driver;
use crate::fields::{ChannelType, DeviceType, TransmissionType, Wildcard};
use crate::messages::{
    AntMessage, AssignChannel, ChannelId, ChannelPeriod, ChannelRfFrequency, OpenChannel,
    RxMessageType, SearchTimeout,
};
use crate::plus::common_datapages::MANUFACTURER_SPECIFIC_RANGE;
use crate::plus::profiles::heart_rate::{
    BatteryStatus, Capabilities, CumulativeOperatingTime, DataPageNumbers, DefaultDataPage,
    DeviceInformation, ManufacturerInformation, ManufacturerSpecific, PreviousHeartBeat,
    ProductInformation, SwimIntervalSummary, DATA_PAGE_NUMBER_MASK,
};
use crate::plus::router::{Channel, ChannelError, NetworkKey, Router, RouterError};
use crate::plus::{duration_to_search_timeout, NETWORK_RF_FREQUENCY};
use core::time::Duration;
use packed_struct::{PackedStruct, PrimitiveEnum};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

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

pub struct HeartRateDisplay<R, W, D: Driver<R, W>> {
    channel: u8,
    device_number: u16,
    transmission_type: TransmissionType,
    router: Weak<RefCell<Router<R, W, D>>>,
    rx_message_callback: Option<fn(&AntMessage)>,
    rx_datapage_callback: Option<fn(Result<DataPages, HeartRateError>)>,
}

impl<R, W, D: Driver<R, W>> HeartRateDisplay<R, W, D> {
    pub fn new(device: Option<(u16, Option<TransmissionType>)>) -> Rc<RefCell<Self>> {
        let device = device.unwrap_or((0, None));
        let device_number = device.0;
        let transmission_type = device.1.unwrap_or_else(TransmissionType::new_wildcard);
        Rc::new(RefCell::new(Self {
            channel: 0,
            device_number,
            transmission_type,
            router: Weak::new(),
            rx_message_callback: None,
            rx_datapage_callback: None,
        }))
    }

    pub fn set_rx_message_callback(&mut self, f: Option<fn(&AntMessage)>) {
        self.rx_message_callback = f;
    }

    pub fn set_rx_datapage_callback(&mut self, f: Option<fn(Result<DataPages, HeartRateError>)>) {
        self.rx_datapage_callback = f;
    }

    pub fn open(&self) -> Result<(), RouterError> {
        let router = self.router.upgrade().unwrap();
        let router = router.borrow();
        let ant_plus_key_index = router.get_key_index(NetworkKey::AntPlusKey);

        let ant_plus_key_index = if let Some(x) = ant_plus_key_index {
            x
        } else {
            return Err(RouterError::ChannelError(ChannelError::NetworkKeyNotSet()));
        };

        // reset state and push config if we got a router
        let assign = AssignChannel::new(
            self.channel,
            ChannelType::BidirectionalSlave,
            ant_plus_key_index,
            None,
        );
        let period = ChannelPeriod::new(self.channel, 8070);
        let channel_id = ChannelId::new(
            self.channel,
            self.device_number,
            DeviceType::new(120.into(), false),
            self.transmission_type,
        ); // TODO type devicetype
        let rf = ChannelRfFrequency::new(self.channel, NETWORK_RF_FREQUENCY);
        let timeout = SearchTimeout::new(
            self.channel,
            duration_to_search_timeout(Duration::from_secs(30)),
        );
        let open = OpenChannel::new(self.channel);
        router.send(&assign)?;
        router.send(&period)?;
        router.send(&channel_id)?;
        router.send(&rf)?;
        router.send(&timeout)?;
        router.send(&open)?;
        Ok(())
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

impl<R, W, D: Driver<R, W>> Channel<R, W, D> for HeartRateDisplay<R, W, D> {
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

    fn set_router(
        &mut self,
        router: Weak<RefCell<Router<R, W, D>>>,
        channel: u8,
    ) -> Result<(), ChannelError> {
        self.reset_state();
        self.channel = channel;
        self.router = router;
        Ok(())
    }

    fn reconnect(&mut self) -> Result<(), ChannelError> {
        todo!()
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
