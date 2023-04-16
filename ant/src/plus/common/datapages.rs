// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use crate::messages::config::{
    DeviceType, TransmissionChannelType, TransmissionGlobalDataPages, TransmissionType,
};
use ant_derive::DataPage;
use derive_new::new;
use packed_struct::prelude::*;

use core::ops::RangeInclusive;

pub const MANUFACTURER_SPECIFIC_RANGE: RangeInclusive<u8> = 112..=127;

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum DataPageNumbers {
    AntFsClientBeacon = 0x43,
    AntFsHostCommandResponse = 0x44,
    RequestDataPage = 0x46,
    CommandStatus = 0x47,
    GenericCommandPage = 0x49,
    OpenChannelCommand = 0x4A,
    ModeSettings = 0x4C,
    MultiComponentSystemManufacturersInformation = 0x4E,
    MultiComponentSystemProductInformation = 0x4F,
    ManufacturersInformation = 0x50,
    ProductInformation = 0x51,
    BatteryStatus = 0x52,
    TimeAndDate = 0x53,
    SubfieldData = 0x54,
    MemoryLevel = 0x55,
    PairedDevices = 0x56,
    ErrorDescription = 0x57,
}

// TODO get field information from ANTFS spec
#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct AntFsClientBeacon {
    #[new(value = "DataPageNumbers::AntFsClientBeacon.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1")]
    pub status_byte_1: u8,
    #[packed_field(bytes = "2")]
    pub status_byte_2: u8,
    #[packed_field(bytes = "3")]
    pub authentication_type: u8,
    #[packed_field(bytes = "4:7")]
    pub device_descriptor_host_serial_number: [u8; 4],
}

// TODO get field information from ANTFS spec
#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct AntFsHostCommandResponse {
    #[new(value = "DataPageNumbers::AntFsHostCommandResponse.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1")]
    pub command: u8,
    #[packed_field(bytes = "2:7")]
    pub parameters: [u8; 6],
}

// TODO add custom functions to set transmit until acked
#[derive(PackedStruct, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct RequestedTransmissionResponse {
    #[packed_field(bits = "0:6")]
    pub number_of_transmissions: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "7")]
    pub use_acknowleged_messages: bool,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum CommandType {
    RequestDataPage = 1,
    RequestAntFsSession = 2,
    RequestDataPageFromSlave = 3,
    RequestDataPageSet = 4,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct RequestDataPage {
    #[new(value = "DataPageNumbers::RequestDataPage.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1:2")]
    pub slave_serial_number: u16,
    #[packed_field(bytes = "3")]
    pub descriptor_byte_1: u8,
    #[packed_field(bytes = "4")]
    pub descriptor_byte_2: u8,
    #[packed_field(bytes = "5")]
    pub requested_transmission_response: RequestedTransmissionResponse,
    #[packed_field(bytes = "6")]
    pub requested_page_number: u8,
    #[packed_field(bytes = "7", ty = "enum")]
    pub command_type: CommandType,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug, Default)]
pub enum CommandStatusValue {
    Pass = 0,
    Fail = 1,
    NotSupported = 2,
    Rejected = 3,
    Pending = 4,
    #[default]
    Uninitialized = 255,
}

// TODO add consts for invalid validation
#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct CommandStatus {
    #[new(value = "DataPageNumbers::CommandStatus.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1")]
    pub last_received_command_id: u8,
    #[packed_field(bytes = "2")]
    pub sequence_number: u8,
    #[packed_field(bytes = "3", ty = "enum")]
    pub command_status: CommandStatusValue,
    #[packed_field(bytes = "4:7")]
    pub data: [u8; 4],
}

pub enum GenericCommandType {
    AntPlusProfileSpecific(u16),
    CustomCommand(u16),
    NoCommand(u16),
}

impl From<u16> for GenericCommandType {
    fn from(field: u16) -> Self {
        match field {
            // NOTE: this value deviates from the datasheets defined 32787 as that overlaps with the above
            // range and is an assumed error in the document
            0..=32767 => GenericCommandType::AntPlusProfileSpecific(field),
            32768..=65534 => GenericCommandType::CustomCommand(field),
            65535 => GenericCommandType::NoCommand(field),
        }
    }
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct GenericCommandPage {
    #[new(value = "DataPageNumbers::GenericCommandPage.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1:2")]
    pub slave_serial_number: u16,
    #[packed_field(bytes = "3:4")]
    pub slave_manufacturer_id: u16,
    #[packed_field(bytes = "5")]
    pub sequence_number: u8,
    #[packed_field(bytes = "6:7")]
    pub command_number: u16,
}

impl GenericCommandPage {
    pub fn get_generic_command(&self) -> GenericCommandType {
        self.command_number.into()
    }
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct OpenChannelCommand {
    #[new(value = "DataPageNumbers::OpenChannelCommand.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1:3")]
    pub serial_number: Integer<u32, packed_bits::Bits24>,
    #[packed_field(bytes = "4")]
    pub device_type: DeviceType,
    #[packed_field(bytes = "5")]
    pub rf_frequency: u8,
    #[packed_field(bytes = "6:7")]
    pub channel_period: u16,
}

// Filed in from FIT SDK 21.015.00
#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum SportMode {
    Generic = 0,
    Running = 1,
    Cycling = 2,
    Transition = 3,
    FitnessEquipment = 4,
    Swimming = 5,
    BasketBall = 6,
    Soccer = 7,
    Tennis = 8,
    AmericanFootball = 9,
    Training = 10,
    Walking = 11,
    CrossCountrySkiing = 12,
    AlpingSking = 13,
    Snowboarding = 14,
    Rowing = 15,
    Mountaineering = 16,
    Hiking = 17,
    Multisport = 18,
    Paddling = 19,
    Flying = 20,
    EBiking = 21,
    Motorcycling = 22,
    Boating = 23,
    Driving = 24,
    Golf = 25,
    Hanggliding = 26,
    HoresebackRiding = 27,
    Hunting = 28,
    Fishing = 29,
    InlineSkating = 30,
    RockClimbing = 31,
    Sailing = 32,
    IceSkating = 33,
    SkyDiving = 34,
    SnowShoeing = 35,
    Snowmobiling = 36,
    StandUpPaddleboarding = 37,
    Surfing = 38,
    Wakeboarding = 39,
    WaterSkiing = 40,
    Kayaking = 41,
    Rafting = 42,
    Windsurfing = 43,
    Kitesurfing = 44,
    Tactical = 45,
    Jumpmaster = 46,
    Boxing = 47,
    FloorClimbing = 48,
    Diving = 53,
    HIIT = 62,
    Racket = 64,
    WaterTubing = 76,
    Wakesurfing = 77,
    All = 254,
    Invalid = 255,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum SubSportMode {
    Generic = 0,
    Treadmill = 1,
    Street = 2,
    Trail = 3,
    Track = 4,
    Spin = 5,
    IndoorCucling = 6,
    Road = 7,
    Mountain = 8,
    Downhill = 9,
    Recumbent = 10,
    Cyclocross = 11,
    Handycling = 12,
    TrackCycling = 13,
    IndoorRowing = 14,
    Elliptical = 15,
    StairClimbing = 16,
    LapSwimming = 17,
    OpenWater = 18,
    FlexibilityTraining = 19,
    StrengthTraining = 20,
    WarmUp = 21,
    Match = 22,
    Exercise = 23,
    Challenge = 24,
    IndoorSkiing = 25,
    CardioTraining = 26,
    IndoorWalking = 27,
    EBikeFitness = 28,
    BMX = 29,
    CasualWalking = 30,
    BikeToRunTransition = 32,
    RunToBikeTransition = 33,
    SwimToBikeTransition = 34,
    ATV = 35,
    Motocross = 36,
    Backcountry = 37,
    Resort = 38,
    RcDrone = 39,
    Wingsuit = 40,
    Whitewater = 41,
    SkateSkiing = 42,
    Yoga = 43,
    Pilates = 44,
    IndoorRunning = 45,
    GravelCycling = 46,
    EBikeMountain = 47,
    Commuting = 48,
    MixedSurface = 49,
    Navigate = 50,
    TrackMe = 51,
    Map = 52,
    SingleGasDiving = 53,
    MultiGasDiving = 54,
    GaugeDiving = 55,
    ApneaDiving = 56,
    ApneaHunting = 57,
    VirtualActivity = 58,
    Obstacle = 59,
    Breathing = 62,
    SailRace = 65,
    Ultra = 67,
    IndoorClimbing = 68,
    Bouldering = 69,
    HIIT = 70,
    Amrap = 73,
    Emom = 74,
    Tabata = 75,
    Pickleball = 84,
    Padel = 85,
    All = 254,
    Invalid = 255,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct ModeSettings {
    #[new(value = "DataPageNumbers::ModeSettings.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[new(default)]
    #[packed_field(bytes = "1:5")]
    _reserved: ReservedOnes<packed_bits::Bits40>,
    #[packed_field(bytes = "6", ty = "enum")]
    pub sub_sport_mode: SubSportMode,
    #[packed_field(bytes = "7", ty = "enum")]
    pub sport_mode: SportMode,
}

#[derive(PackedStruct, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct ComponentIdentifier {
    #[packed_field(bits = "0:3")]
    pub number_of_components: Integer<u8, packed_bits::Bits4>,
    #[packed_field(bits = "4:7")]
    pub component_identifier: Integer<u8, packed_bits::Bits4>,
}

#[derive(PackedStruct, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "5")]
pub struct CommonManufacturersInformation {
    #[packed_field(bytes = "0")]
    pub hw_revision: u8,
    #[packed_field(bytes = "1:2")]
    pub manufacturer_id: u16,
    #[packed_field(bytes = "3:4")]
    pub model_number: u16,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct MultiComponentSystemManufacturersInformation {
    #[new(value = "DataPageNumbers::MultiComponentSystemManufacturersInformation.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[new(default)]
    #[packed_field(bytes = "1")]
    _reserved: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bytes = "2")]
    pub component_identifier: ComponentIdentifier,
    #[packed_field(bytes = "3:7")]
    pub commmon_manufacturers_information: CommonManufacturersInformation,
}

#[derive(PackedStruct, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "6")]
pub struct CommonProductInformation {
    #[packed_field(bytes = "0")]
    pub sw_revision_supplemental: u8,
    #[packed_field(bytes = "1")]
    pub sw_revision_main: u8,
    #[packed_field(bytes = "2:5")]
    pub serial_number: u32,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct MultiComponentSystemProductInformation {
    #[new(value = "DataPageNumbers::MultiComponentSystemProductInformation.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1")]
    pub component_identifier: ComponentIdentifier,
    #[packed_field(bytes = "2:7")]
    pub common_product_information: CommonProductInformation,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct ManufacturersInformation {
    #[new(value = "DataPageNumbers::ManufacturersInformation.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[new(default)]
    #[packed_field(bytes = "1:2")]
    _reserved: ReservedOnes<packed_bits::Bits16>,
    #[packed_field(bytes = "3:7")]
    pub commmon_manufacturers_information: CommonManufacturersInformation,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct ProductInformation {
    #[new(value = "DataPageNumbers::ProductInformation.to_primitive()")]
    #[packed_field(bytes = "0")]
    pub data_page_number: u8,
    #[new(default)]
    #[packed_field(bytes = "1")]
    _reserved: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bytes = "2:7")]
    pub common_product_information: CommonProductInformation,
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug, Default)]
pub enum BatteryStatusField {
    Reserved0 = 0,
    New = 1,
    Good = 2,
    OK = 3,
    Low = 4,
    Critical = 5,
    Reserved1 = 6,
    #[default]
    Invalid = 7,
}

// This is a copy o ComponentIdentifier but with its fields renamed to match the datasheet
#[derive(PackedStruct, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct BatteryIdentifier {
    #[packed_field(bits = "0:3")]
    pub number_of_batteries: Integer<u8, packed_bits::Bits4>,
    #[packed_field(bits = "4:7")]
    pub identifier: Integer<u8, packed_bits::Bits4>,
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum OperatingTimeResolution {
    SixteenSecondResolution = 0,
    TwoSecondResolution = 1,
}

#[derive(PackedStruct, new, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct DescriptiveBitField {
    #[packed_field(bits = "0:3")]
    pub coarse_battery_voltage: Integer<u8, packed_bits::Bits4>,
    #[packed_field(bits = "4:6", ty = "enum")]
    pub battery_status: BatteryStatusField,
    #[packed_field(bits = "7", ty = "enum")]
    pub operating_time_resolution: OperatingTimeResolution,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct BatteryStatus {
    #[new(value = "DataPageNumbers::BatteryStatus.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[new(default)]
    #[packed_field(bytes = "1")]
    _reserved: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bytes = "2")]
    pub battery_identifier: BatteryIdentifier,
    #[packed_field(bytes = "3:5")]
    pub cumulative_operating_time: Integer<u32, packed_bits::Bits24>,
    #[packed_field(bytes = "6")]
    pub fractional_battery_voltage: u8,
    #[packed_field(bytes = "7")]
    pub descriptive_bit_field: DescriptiveBitField,
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum DayOfWeek {
    Sunday = 0,
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Invalid = 7,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct TimeAndDate {
    #[new(value = "DataPageNumbers::TimeAndDate.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[new(default)]
    #[packed_field(bytes = "1")]
    _reserved: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bytes = "2")]
    pub seconds: u8,
    #[packed_field(bytes = "3")]
    pub minutes: u8,
    #[packed_field(bytes = "4")]
    pub hours: u8,
    #[packed_field(bits = "43:47")]
    pub day: Integer<u8, packed_bits::Bits5>,
    #[packed_field(bits = "40:42", ty = "enum")]
    pub day_of_week: DayOfWeek,
    #[packed_field(bytes = "6")]
    pub month: u8,
    #[packed_field(bytes = "7")]
    pub year: u8,
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum Subpage {
    Temperature = 1,
    BarometricPressure = 2,
    Humidity = 3,
    WindSpeed = 4,
    WindDirection = 5,
    ChargingCycles = 6,
    MinimumOperatingTemperature = 7,
    MaximumOperatingTemperature = 8,
    Invalid = 255,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct SubfieldData {
    #[new(value = "DataPageNumbers::SubfieldData.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[new(default)]
    #[packed_field(bytes = "1")]
    _reserved: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bytes = "2", ty = "enum")]
    pub subpage_1: Subpage,
    #[packed_field(bytes = "3", ty = "enum")]
    pub subpage_2: Subpage,
    #[packed_field(bytes = "4:5")]
    pub data_field_1: u16,
    #[packed_field(bytes = "6:7")]
    pub data_field_2: u16,
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum BaseUnits {
    Bit = 0,
    Byte = 1,
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum Units {
    BaseUnit = 0b00,
    Kilo = 0b01,
    Mega = 0b10,
    Tera = 0b11,
}

#[derive(PackedStruct, new, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct TotalSizeUnit {
    #[packed_field(bits = "7", ty = "enum")]
    pub base_units: BaseUnits,
    #[packed_field(bits = "0:6", ty = "enum")]
    pub units: Units,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct MemoryLevel {
    #[new(value = "DataPageNumbers::MemoryLevel.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[new(default)]
    #[packed_field(bytes = "1:3")]
    _reserved: ReservedOnes<packed_bits::Bits24>,
    #[packed_field(bytes = "4")]
    pub percent_used: u8,
    #[packed_field(bytes = "5:6")]
    pub total_size: u16,
    #[packed_field(bytes = "7")]
    pub total_size_unit: TotalSizeUnit,
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum Paired {
    Paired = 1,
    NotPaired = 0,
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum ConnectionState {
    ClosedChannel = 0,
    Searching = 1,
    Synchronised = 2,
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum NetworkKey {
    Public = 0,
    Private = 1,
    AntPlusManaged = 2,
    AntFsKey = 3,
}

#[derive(PackedStruct, new, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct ChannelState {
    #[packed_field(bits = "7", ty = "enum")]
    pub paired: Paired,
    #[packed_field(bits = "3:6", ty = "enum")]
    pub connection_state: ConnectionState,
    #[packed_field(bits = "0:2", ty = "enum")]
    pub network_key: NetworkKey,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct PairedDevices {
    #[new(value = "DataPageNumbers::PairedDevices.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1")]
    pub peripheral_device_index: u8,
    #[packed_field(bytes = "2")]
    pub total_number_of_connected_devices: u8,
    #[packed_field(bytes = "3")]
    pub channel_state: ChannelState,
    #[packed_field(bytes = "4:5")]
    pub peripheral_device_id_device_number: u16,
    #[packed_field(bytes = "6")]
    pub peripheral_device_id_transmission_type: TransmissionType,
    #[packed_field(bytes = "7")]
    pub peripheral_device_id_device_type: DeviceType,
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum ErrorLevel {
    Warning = 1,
    Critical = 2,
}

#[derive(PackedStruct, DataPage, new, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct ErrorDescription {
    #[new(value = "DataPageNumbers::ErrorDescription.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[new(default)]
    #[packed_field(bytes = "1")]
    _reserved0: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bits = "20:23")]
    pub system_component_identifier: Integer<u8, packed_bits::Bits4>,
    #[new(default)]
    #[packed_field(bits = "18:19")]
    _reserved1: ReservedZeroes<packed_bits::Bits2>,
    #[packed_field(bits = "16:17", ty = "enum")]
    pub error_level: ErrorLevel,
    #[packed_field(bytes = "3")]
    pub profile_specific_error_codes: u8,
    #[packed_field(bytes = "4:7")]
    pub manufacturer_specific_error_codes: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ant_fs_client_beacon() {
        let packed = AntFsClientBeacon::new(0, 1, 2, [5, 6, 7, 8])
            .pack()
            .unwrap();

        assert_eq!(packed, [67, 0, 1, 2, 5, 6, 7, 8]);
    }

    #[test]
    fn ant_fs_host_command_response() {
        let packed = AntFsHostCommandResponse::new(12, [0, 1, 2, 3, 4, 5])
            .pack()
            .unwrap();

        assert_eq!(packed, [68, 12, 0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn request_data_page() {
        let packed = RequestDataPage::new(
            0xFFEE,
            0x88,
            0xEE,
            RequestedTransmissionResponse::new(10.into(), true),
            0x55,
            CommandType::RequestDataPage,
        )
        .pack()
        .unwrap();

        assert_eq!(packed, [0x46, 0xEE, 0xFF, 0x88, 0xEE, 0x8A, 0x55, 0x01]);
    }

    #[test]
    fn command_status() {
        let packed = CommandStatus::new(
            0xAB,
            0x12,
            CommandStatusValue::NotSupported,
            [0x55, 0x44, 0x33, 0x22],
        )
        .pack()
        .unwrap();

        assert_eq!(packed, [0x47, 0xAB, 0x12, 0x02, 0x55, 0x44, 0x33, 0x22]);
    }

    #[test]
    fn generic_command_page() {
        let packed = GenericCommandPage::new(0xAABB, 0xCCDD, 5, 0x1122)
            .pack()
            .unwrap();

        assert_eq!(packed, [73, 0xBB, 0xAA, 0xDD, 0xCC, 5, 0x22, 0x11]);
    }

    #[test]
    fn open_channel_command() {
        let packed = OpenChannelCommand::new(
            0xFFDDCC.into(),
            DeviceType::new(80.into(), false),
            12,
            0x4455,
        )
        .pack()
        .unwrap();

        assert_eq!(packed, [74, 0xCC, 0xDD, 0xFF, 80, 12, 0x55, 0x44]);
    }

    #[test]
    fn mode_settings_page() {
        let packed = ModeSettings::new(SubSportMode::HIIT, SportMode::HIIT)
            .pack()
            .unwrap();

        assert_eq!(packed, [76, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 70, 62]);
    }

    #[test]
    fn multi_component_system_manufacturers_information() {
        let packed = MultiComponentSystemManufacturersInformation::new(
            ComponentIdentifier::new(1.into(), 0.into()),
            CommonManufacturersInformation::new(4, 0xBBCC, 0x1122),
        )
        .pack()
        .unwrap();

        assert_eq!(packed, [78, 0xFF, 0x01, 4, 0xCC, 0xBB, 0x22, 0x11]);
    }

    #[test]
    fn multi_component_system_product_information() {
        let packed = MultiComponentSystemProductInformation::new(
            ComponentIdentifier::new(3.into(), 2.into()),
            CommonProductInformation::new(10, 20, 0x77889900),
        )
        .pack()
        .unwrap();

        assert_eq!(packed, [79, 0x23, 10, 20, 0, 0x99, 0x88, 0x77]);
    }

    #[test]
    fn manufacturers_information() {
        let packed = ManufacturersInformation::new(CommonManufacturersInformation::new(10, 2, 292))
            .pack()
            .unwrap();

        assert_eq!(packed, [0x50, 0xFF, 0xFF, 0x0A, 0x02, 0x00, 0x24, 0x01]);
    }

    #[test]
    fn product_information() {
        let packed = ProductInformation::new(CommonProductInformation::new(80, 13, 19136514))
            .pack()
            .unwrap();

        assert_eq!(packed, [0x51, 0xFF, 0x50, 0x0D, 0x02, 0x00, 0x24, 0x01]);
    }

    #[test]
    fn battery_status() {
        let packed = BatteryStatus::new(
            BatteryIdentifier::new(0x1.into(), 0xA.into()),
            0x32C1A.into(),
            0x8B,
            DescriptiveBitField::new(
                2.into(),
                BatteryStatusField::OK,
                OperatingTimeResolution::SixteenSecondResolution,
            ),
        )
        .pack()
        .unwrap();

        assert_eq!(packed, [0x52, 0xFF, 0xA1, 0x1A, 0x2C, 0x03, 0x8B, 0x32]);
    }

    #[test]
    fn time_and_date() {
        let packed = TimeAndDate::new(13, 27, 17.into(), 18.into(), DayOfWeek::Thursday, 6, 09)
            .pack()
            .unwrap();

        assert_eq!(packed, [0x53, 0xFF, 0x0D, 0x1B, 0x11, 0x92, 0x06, 0x09]);
    }

    #[test]
    fn subfield_data() {
        let packed = SubfieldData::new(Subpage::Temperature, Subpage::Humidity, 2667, 6634)
            .pack()
            .unwrap();

        assert_eq!(packed, [0x54, 0xFF, 0x01, 0x03, 0x6B, 0x0A, 0xEA, 0x19]);
    }

    #[test]
    fn memory_level() {
        let packed = MemoryLevel::new(
            150,
            0x3344,
            TotalSizeUnit::new(BaseUnits::Byte, Units::Kilo),
        )
        .pack()
        .unwrap();

        assert_eq!(packed, [85, 0xFF, 0xFF, 0xFF, 150, 0x44, 0x33, 0x81]);
    }

    #[test]
    fn paired_devices() {
        let packed = PairedDevices::new(
            4,
            6,
            ChannelState::new(
                Paired::Paired,
                ConnectionState::Searching,
                NetworkKey::AntPlusManaged,
            ),
            0x3344,
            TransmissionType::new(
                TransmissionChannelType::IndependentChannel,
                TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
                0xF.into(),
            ),
            DeviceType::new(50.into(), false),
        )
        .pack()
        .unwrap();

        assert_eq!(packed, [86, 4, 6, 0x8A, 0x44, 0x33, 0xF1, 50]);
    }

    #[test]
    fn error_description() {
        let packed = ErrorDescription::new(0xA.into(), ErrorLevel::Warning, 0xAC, 0x12345678)
            .pack()
            .unwrap();

        assert_eq!(packed, [87, 0xFF, 0x4A, 0xAC, 0x78, 0x56, 0x34, 0x12]);
    }
}
