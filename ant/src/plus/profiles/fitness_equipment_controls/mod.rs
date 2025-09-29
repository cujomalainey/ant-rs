//! Based off V5 of the Fitness Equipment specification

mod datapages;
mod display;

pub use datapages::*;
pub use display::*;

// use crate::plus::common::datapages::{ModeSettings, RequestDataPage};
use crate::plus::common::msg_handler::StateError;

const DEVICE_TYPE: u8 = 17;

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
            Period::FourHz => 8192,
            Period::TwoHz => 16140,
            Period::OneHz => 32280,
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MonitorTxDataPage {
    MainDataPage(MainDataPage),
    PowerDataPage(PowerDataPage),
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum DisplayTxDataPage {
    // ManufacturerSpecific(ManufacturerSpecific),
    TargetPowerDataPage(TargetPowerDataPage),
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

#[derive(Debug, Clone, Copy)]
pub enum EquipmentType {
    Treadmill,
    Elliptical,
    Reserved,
    Rower,
    Climber,
    NordicSkier,
    StationaryBike,
    General,
}

impl From<u8> for EquipmentType {
    fn from(p: u8) -> EquipmentType {
        match p {
            19 => EquipmentType::Treadmill,
            20 => EquipmentType::Elliptical,
            21 => EquipmentType::Reserved,
            22 => EquipmentType::Rower,
            23 => EquipmentType::Climber,
            24 => EquipmentType::NordicSkier,
            25 => EquipmentType::StationaryBike,
            _ => EquipmentType::General,
        }
    }
}