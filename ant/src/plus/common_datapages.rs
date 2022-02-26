// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::fields::{DeviceType, TransmissionType};
use ant_derive::DataPage;
use packed_struct::prelude::*;
use std::ops::RangeInclusive;

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
#[derive(PackedStruct, DataPage, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct AntFsClientBeacon {
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

impl AntFsClientBeacon {
    pub fn new(
        status_byte_1: u8,
        status_byte_2: u8,
        authentication_type: u8,
        device_descriptor_host_serial_number: [u8; 4],
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::AntFsClientBeacon.to_primitive(),
            status_byte_1,
            status_byte_2,
            authentication_type,
            device_descriptor_host_serial_number,
        }
    }
}

// TODO get field information from ANTFS spec
#[derive(PackedStruct, DataPage, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct AntFsHostCommandResponse {
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1")]
    pub command: u8,
    #[packed_field(bytes = "2:7")]
    pub parameters: [u8; 6],
}

impl AntFsHostCommandResponse {
    pub fn new(command: u8, parameters: [u8; 6]) -> Self {
        Self {
            data_page_number: DataPageNumbers::AntFsHostCommandResponse.to_primitive(),
            command,
            parameters,
        }
    }
}

// TODO add custom functions to set transmit until acked
#[derive(PackedStruct, Copy, Clone, Debug, Default, PartialEq)]
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

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct RequestDataPage {
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

impl RequestDataPage {
    pub fn new(
        slave_serial_number: u16,
        descriptor_byte_1: u8,
        descriptor_byte_2: u8,
        requested_transmission_response: RequestedTransmissionResponse,
        requested_page_number: u8,
        command_type: CommandType,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::RequestDataPage.to_primitive(),
            slave_serial_number,
            descriptor_byte_1,
            descriptor_byte_2,
            requested_transmission_response,
            requested_page_number,
            command_type,
        }
    }
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum CommandStatusValue {
    Pass = 0,
    Fail = 1,
    NotSupported = 2,
    Rejected = 3,
    Pending = 4,
    Uninitialized = 255,
}

impl Default for CommandStatusValue {
    fn default() -> Self {
        CommandStatusValue::Uninitialized
    }
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct CommandStatus {
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

impl CommandStatus {
    pub fn new(
        last_received_command_id: u8,
        sequence_number: u8,
        command_status: CommandStatusValue,
        data: [u8; 4],
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::CommandStatus.to_primitive(),
            last_received_command_id,
            sequence_number,
            command_status,
            data,
        }
    }
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct GenericCommandPage {
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
    // TODO add no command check
}

impl GenericCommandPage {
    pub fn new(
        slave_serial_number: u16,
        slave_manufacturer_id: u16,
        sequence_number: u8,
        command_number: u16,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::GenericCommandPage.to_primitive(),
            slave_serial_number,
            slave_manufacturer_id,
            sequence_number,
            command_number,
        }
    }
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct OpenChannelCommand {
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

impl OpenChannelCommand {
    pub fn new(
        serial_number: Integer<u32, packed_bits::Bits24>,
        device_type: DeviceType,
        rf_frequency: u8,
        channel_period: u16,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::OpenChannelCommand.to_primitive(),
            serial_number,
            device_type,
            rf_frequency,
            channel_period,
        }
    }
}

// TODO fill in this enum from FIT SDK
#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum SportMode {
    Generic = 0,
    Running = 1,
    Cycling = 2,
    Swimming = 5,
}

// TODO fill in this enum from FIT SDK
#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum SubSportMode {
    Generic = 0,
    Treadmill = 1,
    Spin = 5,
    LapSwimming = 11,
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct ModeSettings {
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1:5")]
    pub _reserved: ReservedOnes<packed_bits::Bits40>,
    #[packed_field(bytes = "6", ty = "enum")]
    pub sub_sport_mode: SubSportMode,
    #[packed_field(bytes = "7", ty = "enum")]
    pub sport_mode: SportMode,
}

impl ModeSettings {
    pub fn new(sport_mode: SportMode, sub_sport_mode: SubSportMode) -> Self {
        Self {
            data_page_number: DataPageNumbers::ModeSettings.to_primitive(),
            sub_sport_mode,
            sport_mode,
            _reserved: Default::default(),
        }
    }
}

#[derive(PackedStruct, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct ComponentIdentifier {
    #[packed_field(bits = "0:3")]
    pub number_of_components: Integer<u8, packed_bits::Bits4>,
    #[packed_field(bits = "4:7")]
    pub component_identifier: Integer<u8, packed_bits::Bits4>,
}

#[derive(PackedStruct, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "5")]
pub struct CommonManufacturersInformation {
    #[packed_field(bytes = "0")]
    pub hw_revision: u8,
    #[packed_field(bytes = "1:2")]
    pub manufacturer_id: u16,
    #[packed_field(bytes = "3:4")]
    pub model_number: u16,
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct MultiComponentSystemManufacturersInformation {
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1")]
    _reserved: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bytes = "2")]
    pub component_identifier: ComponentIdentifier,
    #[packed_field(bytes = "3:7")]
    pub commmon_manufacturers_information: CommonManufacturersInformation,
}

impl MultiComponentSystemManufacturersInformation {
    pub fn new(
        component_identifier: ComponentIdentifier,
        commmon_manufacturers_information: CommonManufacturersInformation,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::MultiComponentSystemManufacturersInformation
                .to_primitive(),
            _reserved: Default::default(),
            component_identifier,
            commmon_manufacturers_information,
        }
    }
}

#[derive(PackedStruct, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "6")]
pub struct CommonProductInformation {
    #[packed_field(bytes = "0")]
    pub sw_revision_supplemental: u8,
    #[packed_field(bytes = "1")]
    pub sw_revision_main: u8,
    #[packed_field(bytes = "2:5")]
    pub serial_number: u32,
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct MultiComponentSystemProductInformation {
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1")]
    pub component_identifier: ComponentIdentifier,
    #[packed_field(bytes = "2:7")]
    pub common_product_information: CommonProductInformation,
}

impl MultiComponentSystemProductInformation {
    pub fn new(
        component_identifier: ComponentIdentifier,
        common_product_information: CommonProductInformation,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::MultiComponentSystemProductInformation
                .to_primitive(),
            component_identifier,
            common_product_information,
        }
    }
}

// TODO extract product and manufacter data info into separate struct for multi and regular

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct ManufacturersInformation {
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1:2")]
    _reserved: ReservedOnes<packed_bits::Bits16>,
    #[packed_field(bytes = "3:7")]
    pub commmon_manufacturers_information: CommonManufacturersInformation,
}

impl ManufacturersInformation {
    pub fn new(commmon_manufacturers_information: CommonManufacturersInformation) -> Self {
        Self {
            data_page_number: DataPageNumbers::ManufacturersInformation.to_primitive(),
            _reserved: Default::default(),
            commmon_manufacturers_information,
        }
    }
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct ProductInformation {
    #[packed_field(bytes = "0")]
    pub data_page_number: u8,
    #[packed_field(bytes = "1")]
    _reserved: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bytes = "2:7")]
    pub common_product_information: CommonProductInformation,
}

impl ProductInformation {
    pub fn new(common_product_information: CommonProductInformation) -> Self {
        Self {
            data_page_number: DataPageNumbers::ProductInformation.to_primitive(),
            _reserved: Default::default(),
            common_product_information,
        }
    }
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum BatteryStatusField {
    Reserved0 = 0,
    New = 1,
    Good = 2,
    OK = 3,
    Low = 4,
    Critical = 5,
    Reserved1 = 6,
    Invalid = 7,
}

impl Default for BatteryStatusField {
    fn default() -> Self {
        BatteryStatusField::Invalid
    }
}

// This is a copy o ComponentIdentifier but with its fields renamed to match the datasheet
#[derive(PackedStruct, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct BatteryIdentifier {
    #[packed_field(bits = "0:3")]
    pub number_of_batteries: Integer<u8, packed_bits::Bits4>,
    #[packed_field(bits = "4:7")]
    pub identifier: Integer<u8, packed_bits::Bits4>,
}

impl BatteryIdentifier {
    pub fn new(
        number_of_batteries: Integer<u8, packed_bits::Bits4>,
        identifier: Integer<u8, packed_bits::Bits4>,
    ) -> Self {
        Self {
            number_of_batteries,
            identifier,
        }
    }
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum OperatingTimeResolution {
    SixteenSecondResolution = 0,
    TwoSecondResolution = 1,
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct DescriptiveBitField {
    #[packed_field(bits = "0:3")]
    pub coarse_battery_voltage: Integer<u8, packed_bits::Bits4>,
    #[packed_field(bits = "4:6", ty = "enum")]
    pub battery_status: BatteryStatusField,
    #[packed_field(bits = "7", ty = "enum")]
    pub operating_time_resolution: OperatingTimeResolution,
}

impl DescriptiveBitField {
    pub fn new(
        coarse_battery_voltage: Integer<u8, packed_bits::Bits4>,
        battery_status: BatteryStatusField,
        operating_time_resolution: OperatingTimeResolution,
    ) -> Self {
        Self {
            coarse_battery_voltage,
            battery_status,
            operating_time_resolution,
        }
    }
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct BatteryStatus {
    #[packed_field(bytes = "0")]
    data_page_number: u8,
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

impl BatteryStatus {
    pub fn new(
        battery_identifier: BatteryIdentifier,
        cumulative_operating_time: Integer<u32, packed_bits::Bits24>,
        fractional_battery_voltage: u8,
        descriptive_bit_field: DescriptiveBitField,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::BatteryStatus.to_primitive(),
            _reserved: Default::default(),
            battery_identifier,
            cumulative_operating_time,
            fractional_battery_voltage,
            descriptive_bit_field,
        }
    }
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

// TODO try and move this into the struct directly
#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct Day {
    #[packed_field(bits = "0:4")]
    pub day: Integer<u8, packed_bits::Bits5>,
    #[packed_field(bits = "5:7", ty = "enum")]
    pub day_of_week: DayOfWeek,
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct TimeAndDate {
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1")]
    _reserved: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bytes = "2")]
    pub seconds: u8,
    #[packed_field(bytes = "3")]
    pub minutes: u8,
    #[packed_field(bytes = "4")]
    pub hours: u8,
    #[packed_field(bytes = "5")]
    pub day: Day,
    #[packed_field(bytes = "6")]
    pub month: u8,
    #[packed_field(bytes = "7")]
    pub year: u8,
}

impl TimeAndDate {
    pub fn new(seconds: u8, minutes: u8, hours: u8, day: Day, month: u8, year: u8) -> Self {
        Self {
            data_page_number: DataPageNumbers::TimeAndDate.to_primitive(),
            _reserved: Default::default(),
            seconds,
            minutes,
            hours,
            day,
            month,
            year,
        }
    }
}

// TODO decide if subpage should be a enum
#[derive(PackedStruct, DataPage, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct SubfieldData {
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1")]
    _reserved: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bytes = "2")]
    pub subpage_1: u8,
    #[packed_field(bytes = "3")]
    pub subpage_2: u8,
    #[packed_field(bytes = "4:5")]
    pub data_field_1: u16,
    #[packed_field(bytes = "6:7")]
    pub data_field_2: u16,
}

impl SubfieldData {
    pub fn new(subpage_1: u8, subpage_2: u8, data_field_1: u16, data_field_2: u16) -> Self {
        Self {
            data_page_number: DataPageNumbers::SubfieldData.to_primitive(),
            _reserved: Default::default(),
            subpage_1,
            subpage_2,
            data_field_1,
            data_field_2,
        }
    }
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

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct TotalSizeUnit {
    #[packed_field(bits = "0:6", ty = "enum")]
    pub units: Units,
    #[packed_field(bits = "7", ty = "enum")]
    pub base_units: BaseUnits,
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct MemoryLevel {
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1:3")]
    _reserved: ReservedOnes<packed_bits::Bits24>,
    #[packed_field(bytes = "4")]
    pub percent_used: u8,
    #[packed_field(bytes = "5:6")]
    pub total_size: u16,
    #[packed_field(bytes = "7")]
    pub total_size_unit: TotalSizeUnit,
}

impl MemoryLevel {
    pub fn new(percent_used: u8, total_size: u16, total_size_unit: TotalSizeUnit) -> Self {
        Self {
            data_page_number: DataPageNumbers::MemoryLevel.to_primitive(),
            _reserved: Default::default(),
            percent_used,
            total_size,
            total_size_unit,
        }
    }
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

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct ChannelState {
    #[packed_field(bits = "7", ty = "enum")]
    pub paired: Paired,
    #[packed_field(bits = "3:6", ty = "enum")]
    pub connection_state: ConnectionState,
    #[packed_field(bits = "0:2", ty = "enum")]
    pub network_key: NetworkKey,
}

impl ChannelState {
    pub fn new(paired: Paired, connection_state: ConnectionState, network_key: NetworkKey) -> Self {
        Self {
            paired,
            connection_state,
            network_key,
        }
    }
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct PairedDevices {
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

impl PairedDevices {
    pub fn new(
        peripheral_device_index: u8,
        total_number_of_connected_devices: u8,
        channel_state: ChannelState,
        peripheral_device_id_device_number: u16,
        peripheral_device_id_transmission_type: TransmissionType,
        peripheral_device_id_device_type: DeviceType,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::PairedDevices.to_primitive(),
            peripheral_device_index,
            total_number_of_connected_devices,
            channel_state,
            peripheral_device_id_device_number,
            peripheral_device_id_transmission_type,
            peripheral_device_id_device_type,
        }
    }
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum ErrorLevel {
    Warning = 1,
    Critical = 2,
}

#[derive(PackedStruct, DataPage, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct ErrorDescription {
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1")]
    _reserved0: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bits = "16:19")]
    pub system_component_identifier: Integer<u8, packed_bits::Bits4>,
    #[packed_field(bits = "20:21")]
    _reserved1: ReservedZeroes<packed_bits::Bits2>,
    #[packed_field(bits = "22:23", ty = "enum")]
    pub error_level: ErrorLevel,
    #[packed_field(bytes = "3")]
    pub profile_specific_error_codes: u8,
    #[packed_field(bytes = "4:7")]
    pub manufacturer_specific_error_codes: u32,
}

impl ErrorDescription {
    pub fn new(
        system_component_identifier: Integer<u8, packed_bits::Bits4>,
        error_level: ErrorLevel,
        profile_specific_error_codes: u8,
        manufacturer_specific_error_codes: u32,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::ErrorDescription.to_primitive(),
            _reserved0: Default::default(),
            system_component_identifier,
            _reserved1: Default::default(),
            error_level,
            profile_specific_error_codes,
            manufacturer_specific_error_codes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ant_fs_client_beacon() {
        // TODO
    }

    #[test]
    fn ant_fs_host_command_response() {
        // TODO
    }

    #[test]
    fn request_data_page() {
        // TODO
    }

    #[test]
    fn command_status() {
        // TODO
    }

    #[test]
    fn generic_command_page() {
        // TODO
    }

    #[test]
    fn open_channel_command() {
        // TODO
    }

    #[test]
    fn mode_settings_page() {
        // TODO
    }

    #[test]
    fn multi_component_system_manufacturers_information() {
        // TODO
    }

    #[test]
    fn multi_component_system_product_information() {
        // TODO
    }

    #[test]
    fn manufacturers_information() {
        let unpacked =
            ManufacturersInformation::unpack(&[0x50, 0xFF, 0xFF, 0x0A, 0x02, 0x00, 0x24, 0x01])
                .unwrap();
        assert_eq!(unpacked.commmon_manufacturers_information.hw_revision, 10);
        assert_eq!(
            unpacked.commmon_manufacturers_information.manufacturer_id,
            2
        );
        assert_eq!(unpacked.commmon_manufacturers_information.model_number, 292);
    }

    #[test]
    fn product_information() {
        let unpacked =
            ProductInformation::unpack(&[0x51, 0xFF, 0x50, 0x0D, 0x02, 0x00, 0x24, 0x01]).unwrap();

        assert_eq!(
            unpacked.common_product_information.sw_revision_supplemental,
            80
        );
        assert_eq!(unpacked.common_product_information.sw_revision_main, 13);
        assert_eq!(unpacked.common_product_information.serial_number, 19136514);
    }

    #[test]
    fn battery_status() {
        let unpacked =
            BatteryStatus::unpack(&[0x52, 0xFF, 0xA1, 0x1A, 0x2C, 0x03, 0x8B, 0x32]).unwrap();

        assert_eq!(
            unpacked.descriptive_bit_field,
            DescriptiveBitField {
                coarse_battery_voltage: 2.into(),
                battery_status: BatteryStatusField::OK,
                operating_time_resolution: OperatingTimeResolution::SixteenSecondResolution
            }
        );
        assert_eq!(unpacked.cumulative_operating_time, 0x32C1A.into());
        assert_eq!(unpacked.fractional_battery_voltage, 0x8B);
        // TODO check below against SimulANT
        assert_eq!(unpacked.battery_identifier.identifier, 0xA.into());
        assert_eq!(unpacked.battery_identifier.number_of_batteries, 0x1.into());
    }

    #[test]
    fn time_and_date() {
        let unpacked =
            TimeAndDate::unpack(&[0x53, 0xFF, 0x0D, 0x1B, 0x11, 0x92, 0x06, 0x09]).unwrap();

        assert_eq!(unpacked.seconds, 13);
        assert_eq!(unpacked.minutes, 27);
        assert_eq!(unpacked.hours, 17);
        assert_eq!(unpacked.day.day_of_week, DayOfWeek::Thursday);
        assert_eq!(unpacked.day.day, 18.into());
        assert_eq!(unpacked.month, 6);
        assert_eq!(unpacked.year, 09);
    }

    #[test]
    fn subfield_data() {
        let unpacked =
            SubfieldData::unpack(&[0x54, 0xFF, 0x01, 0x03, 0x6B, 0x0A, 0xEA, 0x19]).unwrap();

        assert_eq!(unpacked.subpage_1, 1);
        assert_eq!(unpacked.subpage_2, 3);
        assert_eq!(unpacked.data_field_1 as i16, 2667);
        assert_eq!(unpacked.data_field_2, 6634);
    }

    #[test]
    fn memory_level() {
        // TODO
    }

    #[test]
    fn paired_devices() {
        // TODO
    }

    #[test]
    fn error_description() {
        // TODO
    }
}
