// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::channel::duration_to_search_timeout;
use crate::channel::{ChanError, RxHandler, TxHandler};
use crate::messages::config::{
    ChannelType, TransmissionChannelType, TransmissionGlobalDataPages, TransmissionType,
};
use crate::messages::{AntMessage, RxMessage, TxMessage, TxMessageChannelConfig, TxMessageData};
use crate::plus::common::datapages::MANUFACTURER_SPECIFIC_RANGE;
use crate::plus::common::msg_handler::{ChannelConfig, MessageHandler};
use crate::plus::profiles::heart_rate::{
    BatteryStatus, Capabilities, CumulativeOperatingTime, DataPageNumbers, DefaultDataPage,
    DeviceInformation, Error, ManufacturerInformation, ManufacturerSpecific, MonitorTxDataPage,
    Period, PreviousHeartBeat, ProductInformation, SwimIntervalSummary, DATA_PAGE_NUMBER_MASK,
    DEVICE_TYPE,
};
use crate::plus::NETWORK_RF_FREQUENCY;

use packed_struct::prelude::{packed_bits::Bits, Integer};
use packed_struct::{PackedStruct, PrimitiveEnum};

use std::time::Duration;

pub struct Display<T: TxHandler<TxMessage>, R: RxHandler<AntMessage>> {
    msg_handler: MessageHandler,
    rx_message_callback: Option<fn(&AntMessage)>,
    rx_datapage_callback: Option<fn(Result<MonitorTxDataPage, Error>)>,
    tx_message_callback: Option<fn() -> Option<TxMessageChannelConfig>>,
    tx_datapage_callback: Option<fn() -> Option<TxMessageData>>,
    tx: T,
    rx: R,
}

pub struct DisplayConfig {
    pub channel: u8,
    pub device_number: u16,
    pub device_number_extension: Integer<u8, Bits<4>>,
    pub ant_plus_key_index: u8,
    pub period: Period,
}

impl<T: TxHandler<TxMessage>, R: RxHandler<AntMessage>> Display<T, R> {
    pub fn new(
        conf: DisplayConfig,
        // TODO make this a type
        tx: T,
        rx: R,
    ) -> Self {
        let transmission_type = if conf.device_number_extension == 0.into() {
            TransmissionType::new_wildcard()
        } else {
            TransmissionType::new(
                TransmissionChannelType::IndependentChannel,
                TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
                conf.device_number_extension,
            )
        };
        let channel_config = ChannelConfig {
            channel: conf.channel,
            device_number: conf.device_number,
            device_type: DEVICE_TYPE,
            channel_type: ChannelType::BidirectionalSlave,
            network_key_index: conf.ant_plus_key_index,
            transmission_type,
            radio_frequency: NETWORK_RF_FREQUENCY,
            timeout_duration: duration_to_search_timeout(Duration::from_secs(30)),
            channel_period: conf.period.into(),
        };
        Self {
            rx_message_callback: None,
            rx_datapage_callback: None,
            tx_message_callback: None,
            tx_datapage_callback: None,
            msg_handler: MessageHandler::new(&channel_config),
            tx,
            rx,
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

    pub fn set_rx_datapage_callback(&mut self, f: Option<fn(Result<MonitorTxDataPage, Error>)>) {
        self.rx_datapage_callback = f;
    }

    pub fn set_tx_message_callback(&mut self, f: Option<fn() -> Option<TxMessageChannelConfig>>) {
        self.tx_message_callback = f;
    }

    pub fn set_tx_datapage_callback(&mut self, f: Option<fn() -> Option<TxMessageData>>) {
        self.tx_datapage_callback = f;
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

    fn parse_dp(&mut self, data: &[u8; 8]) -> Result<MonitorTxDataPage, Error> {
        let dp_num = data[0] & DATA_PAGE_NUMBER_MASK;
        if let Some(dp) = DataPageNumbers::from_primitive(dp_num) {
            let parsed = match dp {
                DataPageNumbers::DefaultDataPage => {
                    MonitorTxDataPage::DefaultDataPage(DefaultDataPage::unpack(data)?)
                }
                DataPageNumbers::CumulativeOperatingTime => {
                    MonitorTxDataPage::CumulativeOperatingTime(CumulativeOperatingTime::unpack(
                        data,
                    )?)
                }
                DataPageNumbers::ManufacturerInformation => {
                    MonitorTxDataPage::ManufacturerInformation(ManufacturerInformation::unpack(
                        data,
                    )?)
                }
                DataPageNumbers::ProductInformation => {
                    MonitorTxDataPage::ProductInformation(ProductInformation::unpack(data)?)
                }
                DataPageNumbers::PreviousHeartBeat => {
                    MonitorTxDataPage::PreviousHeartBeat(PreviousHeartBeat::unpack(data)?)
                }
                DataPageNumbers::SwimIntervalSummary => {
                    MonitorTxDataPage::SwimIntervalSummary(SwimIntervalSummary::unpack(data)?)
                }
                DataPageNumbers::Capabilities => {
                    MonitorTxDataPage::Capabilities(Capabilities::unpack(data)?)
                }
                DataPageNumbers::BatteryStatus => {
                    MonitorTxDataPage::BatteryStatus(BatteryStatus::unpack(data)?)
                }
                DataPageNumbers::DeviceInformation => {
                    MonitorTxDataPage::DeviceInformation(DeviceInformation::unpack(data)?)
                }
                // Add all valid profile specific pages below if they are invalid in this direction
                DataPageNumbers::HRFeatureCommand => {
                    return Err(Error::UnsupportedDataPage(dp_num))
                }
            };
            return Ok(parsed);
        }
        if MANUFACTURER_SPECIFIC_RANGE.contains(&dp_num) {
            return Ok(MonitorTxDataPage::ManufacturerSpecific(
                ManufacturerSpecific::unpack(data)?,
            ));
        }
        Err(Error::UnsupportedDataPage(dp_num))
    }

    pub fn process(&mut self) -> Result<(), ChanError> {
        // TODO handle closed channel
        while let Ok(msg) = self.rx.try_recv() {
            if let Some(f) = self.rx_message_callback {
                f(&msg);
            }
            match msg.message {
                RxMessage::BroadcastData(msg) => self.handle_dp(&msg.payload.data),
                RxMessage::AcknowledgedData(msg) => self.handle_dp(&msg.payload.data),
                _ => (),
            }
            match self.msg_handler.receive_message(&msg) {
                Ok(_) => (),
                Err(e) => {
                    if let Some(f) = self.rx_datapage_callback {
                        f(Err(e.into()));
                    }
                }
            }
        }

        // TODO handle errors
        if let Some(msg) = self.msg_handler.send_message() {
            self.tx.try_send(msg)?;
        }
        if let Some(callback) = self.tx_message_callback {
            if let Some(mut msg) = callback() {
                msg.set_channel(self.msg_handler.get_channel());
                self.tx.try_send(msg.into())?;
            }
        }
        if self.msg_handler.is_tx_ready() {
            if let Some(callback) = self.tx_datapage_callback {
                if let Some(mut msg) = callback() {
                    msg.set_channel(self.msg_handler.get_channel());
                    self.msg_handler.tx_sent();
                    self.tx.try_send(msg.into())?;
                }
            }
        }
        Ok(())
    }
}
