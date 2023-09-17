// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Based off V2.5 of the Heart Rate specification

mod datapages;
mod display;
mod monitor;

pub use datapages::*;
pub use display::*;
pub use monitor::*;

use crate::plus::common::datapages::{ModeSettings, RequestDataPage};
use crate::plus::common::msg_handler::StateError;

#[derive(Debug, Default)]
pub enum Period {
    #[default]
    FourHz,
    TwoHz,
    OneHz,
}

impl From<Period> for u16 {
    fn from(p: Period) -> u16 {
        match p {
            Period::FourHz => 8070,
            Period::TwoHz => 16140,
            Period::OneHz => 32280,
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MonitorTxDataPages {
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

pub enum DisplayTxDataPages {
    HRFeatureCommand(HRFeatureCommand),
    RequestDataPage(RequestDataPage),
    ModeSettings(ModeSettings),
    ManufacturerSpecific(ManufacturerSpecific),
}

#[derive(Debug, Clone)]
pub enum Error {
    BytePatternError(packed_struct::PackingError),
    UnsupportedDataPage(u8),
    PageAlreadyPending(),
    NotAssociated(),
    ConfigurationError(StateError),
}

impl From<packed_struct::PackingError> for Error {
    fn from(err: packed_struct::PackingError) -> Self {
        Self::BytePatternError(err)
    }
}

impl From<StateError> for Error {
    fn from(err: StateError) -> Self {
        Self::ConfigurationError(err)
    }
}
