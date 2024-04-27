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
use crate::messages::data::BroadcastData;
use crate::messages::{AntMessage, RxMessage, TxMessage, TxMessageChannelConfig};
use crate::plus::common::datapages::{
    DataPageNumbers as CommonDataPageNumbers, ModeSettings, RequestDataPage,
    MANUFACTURER_SPECIFIC_RANGE,
};
use crate::plus::common::msg_handler::{ChannelConfig, MessageHandler};
use crate::plus::profiles::heart_rate::{
    DataPageNumbers, DisplayTxDataPage, Error, HRFeatureCommand, ManufacturerSpecific, Period,
    DATA_PAGE_NUMBER_MASK, DEVICE_TYPE,
};
use crate::plus::NETWORK_RF_FREQUENCY;

use packed_struct::prelude::{packed_bits::Bits, Integer};
use packed_struct::{PackedStruct, PrimitiveEnum};

use std::time::Duration;

/// Main datapage config (0 or 4)
pub enum MainDataPage {
    DefaultDataPage,
    PreviousHeartBeat,
}

pub struct Config {
    /// Device number for the monitor, cannot be 0
    pub device_number: u16,
    /// Transmission type extension for the monitor, cannot be 0
    pub transmission_type_extension: Integer<u8, Bits<4>>,
    /// Default datapage when not in swim or gym mode
    pub main_data_page: MainDataPage,
    /// Support datapage 1?
    pub cumulative_operating_time_supported: bool,
    /// Support datapage 7?
    pub battery_status_supported: bool,
    /// Support datapage 5?
    pub swim_mode_supported: bool,
    /// Support gym mode?
    pub gym_mode_supported: bool,
    /// Total number of manufacturer pages, this is used in secondary page pattern computing
    pub number_manufacturer_pages: u8,
    /// Number of main pages to send before a background cycle, must be <=65 to be spec compliant
    pub background_page_interval: u8,
}

type RxDataPageCallback = fn(Result<DisplayTxDataPage, Error>);
type TxDatapageCallback = fn(&TxDatapage) -> [u8; 8];

/// Collection of datapge transmission state variables
struct PageState {
    count: u8,
    background_count: u8,
    weave_count: u8,
}

const WEAVE_PATTERN: [TxDatapage; 8] = [
    TxDatapage::ManufacturerInformation(),
    TxDatapage::ManufacturerInformation(),
    TxDatapage::SwimIntervalSummary(),
    TxDatapage::ManufacturerInformation(),
    TxDatapage::SwimIntervalSummary(),
    TxDatapage::SwimIntervalSummary(),
    TxDatapage::ManufacturerInformation(),
    TxDatapage::SwimIntervalSummary(),
];

/// A heart rate sensor channel configuration
///
/// When using this profile, mode changes initiaded by display must be triggered by your code. E.g.
/// if display sends [ModeSettings] your code must call [Monitor::set_swim_mode]. This is so your code
/// can update the config once it is ready to handle the new state.
pub struct Monitor {
    msg_handler: MessageHandler,
    rx_message_callback: Option<fn(&AntMessage)>,
    rx_datapage_callback: RxDataPageCallback,
    tx_message_callback: Option<fn() -> Option<TxMessageChannelConfig>>,
    tx_datapage_callback: TxDatapageCallback,
    in_gym_mode: bool,
    in_swim_mode: bool,
    config: Config,
    page_state: PageState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TxDatapage {
    DefaultDataPage(),
    CumulativeOperatingTime(),
    ManufacturerInformation(),
    ProductInformation(),
    PreviousHeartBeat(),
    SwimIntervalSummary(),
    Capabilities(),
    BatteryStatus(),
    ManufacturerSpecific(u8),
}

impl Monitor {
    pub fn new(
        config: Config,
        ant_plus_key_index: u8,
        rx_datapage_callback: RxDataPageCallback,
        tx_datapage_callback: TxDatapageCallback,
    ) -> Self {
        Self {
            rx_message_callback: None,
            rx_datapage_callback,
            tx_message_callback: None,
            tx_datapage_callback,
            msg_handler: MessageHandler::new(&ChannelConfig {
                device_number: config.device_number,
                device_type: DEVICE_TYPE,
                channel_type: ChannelType::BidirectionalMaster,
                network_key_index: ant_plus_key_index,
                transmission_type: TransmissionType::new(
                    TransmissionChannelType::IndependentChannel,
                    TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
                    config.transmission_type_extension,
                ),
                radio_frequency: NETWORK_RF_FREQUENCY,
                timeout_duration: duration_to_search_timeout(Duration::from_secs(30)),
                channel_period: Period::FourHz.into(), // Monitor always uses 4Hz, display may use
                                                       // less
            }),
            config,
            in_gym_mode: false,
            in_swim_mode: false,
            page_state: PageState {
                count: 0,
                background_count: 0,
                weave_count: 0,
            },
        }
    }

    pub fn open(&mut self) {
        self.msg_handler.open();
    }

    pub fn close(&mut self) {
        self.msg_handler.close();
    }

    /// Set callback for users to observe every message this channel observes
    pub fn set_rx_message_callback(&mut self, f: Option<fn(&AntMessage)>) {
        self.rx_message_callback = f;
    }

    /// Set callback for users to observe every message this channel observes
    pub fn set_rx_datapage_callback(&mut self, f: RxDataPageCallback) {
        self.rx_datapage_callback = f;
    }

    /// Set callback for users to send channel specific config messages
    /// is called continously every TX cycle until None is returned
    pub fn set_tx_message_callback(&mut self, f: Option<fn() -> Option<TxMessageChannelConfig>>) {
        self.tx_message_callback = f;
    }

    /// Set callback for users to observe every message this channel observes
    pub fn set_tx_datapage_callback(&mut self, f: TxDatapageCallback) {
        self.tx_datapage_callback = f;
    }

    /// Used to put profile into gym mode
    /// See section 6.3 for how this modifies the transmission pattern.
    ///
    /// Should be set in acknowledgement to an [HRFeatureCommand] message.
    ///
    /// This command will be ignored if [Config::gym_mode_supported] is false.
    pub fn set_gym_mode(&mut self, enabled: bool) {
        self.in_gym_mode = self.config.gym_mode_supported && enabled;
    }

    /// Used to put profile into swim mode
    /// See section 6.3 for how this modifies the transmission pattern.
    ///
    /// Should be set in acknowledgement to a [ModeSettings] message.
    ///
    /// This command will be ignored if [Config::swim_mode_supported] is false.
    pub fn set_swim_mode(&mut self, enabled: bool) {
        self.in_swim_mode = self.config.swim_mode_supported && enabled;
    }

    pub fn reset_state(&mut self) {
        todo!();
    }

    // get result and call callback
    fn handle_dp(&mut self, data: &[u8; 8]) {
        let dp = self.parse_dp(data);
        let f = self.rx_datapage_callback;
        f(dp);
    }

    fn parse_dp(&mut self, data: &[u8; 8]) -> Result<DisplayTxDataPage, Error> {
        let dp_num = data[0] & DATA_PAGE_NUMBER_MASK;
        if let Some(dp) = DataPageNumbers::from_primitive(dp_num) {
            return Ok(match dp {
                DataPageNumbers::HRFeatureCommand => {
                    DisplayTxDataPage::HRFeatureCommand(HRFeatureCommand::unpack(data)?)
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
                    DisplayTxDataPage::ModeSettings(ModeSettings::unpack(data)?)
                }
                CommonDataPageNumbers::RequestDataPage => {
                    // TODO handle properly into cycle
                    DisplayTxDataPage::RequestDataPage(RequestDataPage::unpack(data)?)
                }
                _ => return Err(Error::UnsupportedDataPage(dp_num)),
            });
        }
        if MANUFACTURER_SPECIFIC_RANGE.contains(&dp_num) {
            return Ok(DisplayTxDataPage::ManufacturerSpecific(
                ManufacturerSpecific::unpack(data)?,
            ));
        }
        Err(Error::UnsupportedDataPage(dp_num))
    }

    // Parses current config to identify main datapage, exlcuding manufacturer information in gym
    // mode case
    fn get_main_page(&mut self) -> TxDatapage {
        if self.in_gym_mode {
            if self.in_swim_mode {
                // do interleaved mainpage
                let count = self.page_state.weave_count;
                self.page_state.weave_count = if count >= 7 { 0 } else { count + 1 };
                WEAVE_PATTERN[count as usize]
            } else {
                TxDatapage::ManufacturerInformation()
            }
        } else if self.in_swim_mode {
            TxDatapage::SwimIntervalSummary()
        } else {
            match self.config.main_data_page {
                MainDataPage::DefaultDataPage => TxDatapage::DefaultDataPage(),
                MainDataPage::PreviousHeartBeat => TxDatapage::PreviousHeartBeat(),
            }
        }
    }

    // returns current background page based on background_count, resets count if exceeds value
    //
    // works by incrementing cumulative offset on each optional page to create slices in the count
    fn get_secondary_page(&mut self) -> TxDatapage {
        let count = self.page_state.background_count;
        let mut offset = 0;
        if self.config.cumulative_operating_time_supported {
            if count == 0 {
                return TxDatapage::CumulativeOperatingTime();
            }
            // optional, increase offset if enabled
            offset += 1;
        }
        if count == offset {
            return TxDatapage::ManufacturerInformation();
        } else if count == 1 + offset {
            return TxDatapage::ProductInformation();
        } else if count == 2 + offset {
            return TxDatapage::Capabilities();
        }
        if self.config.battery_status_supported {
            if count == 3 + offset {
                return TxDatapage::BatteryStatus();
            }
            offset += 1;
        }
        if count - 3 - offset < self.config.number_manufacturer_pages {
            TxDatapage::ManufacturerSpecific(count - 3 - offset)
        } else {
            self.page_state.background_count = 0;
            self.get_secondary_page()
        }
    }

    // Datapage sequence state machine
    fn get_next_datapage(&mut self) -> TxDatapage {
        let count = self.page_state.count;
        self.page_state.count += 1;
        if count < self.config.background_page_interval {
            // return main page for first n counts
            self.get_main_page()
        } else if count < self.config.background_page_interval + 4 {
            // return secondary page for 4 counts
            self.get_secondary_page()
        } else {
            // recurse with new state
            self.page_state.count = 0;
            self.page_state.background_count += 1;
            self.get_next_datapage()
        }
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
                let f = self.rx_datapage_callback;
                f(Err(e.into()));
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
            let callback = self.tx_datapage_callback;
            let dp = self.get_next_datapage();
            let msg = BroadcastData::new(channel, callback(&dp)); // TODO handle ack param
            self.msg_handler.tx_sent();
            return Some(msg.into());
        }
        None
    }

    fn set_channel(&mut self, channel: ChannelAssignment) {
        self.msg_handler.set_channel(channel);
    }
}
