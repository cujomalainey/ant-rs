// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::plus::common_datapages::BatteryStatusField;
use ant_derive::DataPage;
use packed_struct::prelude::*;

// TODO TEST THIS FILE
// TODO add is_valid checks to fields
// TODO hard code datapage values
// TODO add invalid defaults

pub const DATA_PAGE_NUMBER_MASK: u8 = 0x7F;

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum DataPageNumbers {
    DefaultDataPage = 0,
    CumulativeOperatingTime = 1,
    ManufacturerInformation = 2,
    ProductInformation = 3,
    PreviousHeartBeat = 4,
    SwimIntervalSummary = 5,
    Capabilities = 6,
    BatteryStatus = 7,
    DeviceInformation = 9,
    HRFeatureCommand = 32,
}

// TODO is this doing anything?
impl From<DataPageNumbers> for Integer<u8, packed_bits::Bits7> {
    fn from(dp: DataPageNumbers) -> Self {
        (dp as u8).into()
    }
}

/// The last 4 bytes in every message in the heart rate profile are the same, this maps out those
/// fields
#[derive(PackedStruct, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "4")]
pub struct CommonData {
    #[packed_field(bytes = "0:1")]
    pub heart_beat_event_time: u16,
    #[packed_field(bytes = "2")]
    pub heart_beat_count: u8,
    #[packed_field(bytes = "3")]
    pub computed_heart_rate: u8,
}

impl CommonData {
    pub fn new(heart_beat_event_time: u16, heart_beat_count: u8, computed_heart_rate: u8) -> Self {
        Self {
            heart_beat_event_time,
            heart_beat_count,
            computed_heart_rate,
        }
    }
}

/// This struct represents datapage 0 in the heart rate profile.
#[derive(PackedStruct, DataPage, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct DefaultDataPage {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1:3")]
    _reserved: ReservedOnes<packed_bits::Bits24>,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

impl DefaultDataPage {
    pub fn new(page_change_toggle: bool, common: CommonData) -> Self {
        Self {
            data_page_number: DataPageNumbers::DefaultDataPage.into(),
            page_change_toggle,
            common,
            _reserved: Default::default(),
        }
    }
}

/// This struct represents datapage 1 in the heart rate profile.
#[derive(PackedStruct, DataPage, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct CumulativeOperatingTime {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1:3")]
    pub cumulative_operating_time: Integer<u32, packed_bits::Bits24>,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

impl CumulativeOperatingTime {
    pub fn new(
        page_change_toggle: bool,
        cumulative_operating_time: Integer<u32, packed_bits::Bits24>,
        common: CommonData,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::CumulativeOperatingTime.into(),
            page_change_toggle,
            cumulative_operating_time,
            common,
        }
    }
}

/// This struct represents datapage 2 in the heart rate profile.
#[derive(PackedStruct, DataPage, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct ManufacturerInformation {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1")]
    pub manufacturer_id: u8,
    #[packed_field(bytes = "2:3")]
    pub serial_number: u16,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

impl ManufacturerInformation {
    pub fn new(
        page_change_toggle: bool,
        manufacturer_id: u8,
        serial_number: u16,
        common: CommonData,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::ManufacturerInformation.into(),
            page_change_toggle,
            manufacturer_id,
            serial_number,
            common,
        }
    }
}

/// This struct represents datapage 3 in the heart rate profile.
#[derive(PackedStruct, DataPage, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct ProductInformation {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1")]
    pub hardware_version: u8,
    #[packed_field(bytes = "2")]
    pub software_version: u8,
    #[packed_field(bytes = "3")]
    pub model_number: u8,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

impl ProductInformation {
    pub fn new(
        page_change_toggle: bool,
        hardware_version: u8,
        software_version: u8,
        model_number: u8,
        common: CommonData,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::ProductInformation.into(),
            page_change_toggle,
            hardware_version,
            software_version,
            model_number,
            common,
        }
    }
}

/// This struct represents datapage 4 in the heart rate profile.
#[derive(PackedStruct, DataPage, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct PreviousHeartBeat {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1")]
    pub manufacturer_specific: u8,
    #[packed_field(bytes = "2:3")]
    pub previous_heart_beat_event_time: u16,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

impl PreviousHeartBeat {
    pub fn new(
        page_change_toggle: bool,
        manufacturer_specific: u8,
        previous_heart_beat_event_time: u16,
        common: CommonData,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::PreviousHeartBeat.into(),
            page_change_toggle,
            manufacturer_specific,
            previous_heart_beat_event_time,
            common,
        }
    }
}

/// This struct represents datapage 5 in the heart rate profile.
/// Monitors don't need to implement this unless they support the Swimming feature.
#[derive(PackedStruct, DataPage, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct SwimIntervalSummary {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1")]
    pub interval_average_heart_rate: u8,
    #[packed_field(bytes = "2")]
    pub interval_maximum_heart_rate: u8,
    #[packed_field(bytes = "3")]
    pub session_average_heart_rate: u8,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

impl SwimIntervalSummary {
    pub fn new(
        page_change_toggle: bool,
        interval_average_heart_rate: u8,
        interval_maximum_heart_rate: u8,
        session_average_heart_rate: u8,
        common: CommonData,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::SwimIntervalSummary.into(),
            page_change_toggle,
            interval_average_heart_rate,
            interval_maximum_heart_rate,
            session_average_heart_rate,
            common,
        }
    }
}

#[derive(PackedStruct, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct Features {
    #[packed_field(bits = "0")]
    pub extended_running_features: bool,
    #[packed_field(bits = "1")]
    pub extended_cycling_features: bool,
    #[packed_field(bits = "2")]
    pub extended_swimming_features: bool,
    #[packed_field(bits = "3")]
    pub gym_mode: bool,
    #[packed_field(bits = "4:5")]
    _reserved: ReservedZeroes<packed_bits::Bits3>,
    #[packed_field(bits = "6")]
    pub manufacturer_specific_feature_0: bool,
    #[packed_field(bits = "7")]
    pub manufacturer_specific_feature_1: bool,
}

impl Features {
    pub fn new(
        extended_running_features: bool,
        extended_cycling_features: bool,
        extended_swimming_features: bool,
        gym_mode: bool,
        manufacturer_specific_feature_0: bool,
        manufacturer_specific_feature_1: bool,
    ) -> Self {
        Self {
            extended_running_features,
            extended_cycling_features,
            extended_swimming_features,
            gym_mode,
            _reserved: Default::default(),
            manufacturer_specific_feature_0,
            manufacturer_specific_feature_1,
        }
    }
}

/// This struct represents datapage 6 in the heart rate profile.
#[derive(PackedStruct, DataPage, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct Capabilities {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1")]
    _reserved: ReservedOnes<packed_bits::Bits8>,
    #[packed_field(bytes = "2")]
    pub features_supported: Features,
    #[packed_field(bytes = "3")]
    pub features_enabled: Features,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

impl Capabilities {
    pub fn new(
        page_change_toggle: bool,
        features_supported: Features,
        features_enabled: Features,
        common: CommonData,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::Capabilities.into(),
            page_change_toggle,
            _reserved: Default::default(),
            features_supported,
            features_enabled,
            common,
        }
    }
}

// Note we cannot reuse the common datapage battery fields because HR does not define bit 7
#[derive(PackedStruct, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct DescriptiveBitField {
    #[packed_field(bits = "0:3")]
    pub coarse_battery_voltage: Integer<u8, packed_bits::Bits4>,
    #[packed_field(bits = "4:6", ty = "enum")]
    pub battery_status: BatteryStatusField,
    #[packed_field(bits = "7")]
    _reserved: ReservedZeroes<packed_bits::Bits1>,
}

impl DescriptiveBitField {
    pub fn new(
        coarse_battery_voltage: Integer<u8, packed_bits::Bits4>,
        battery_status: BatteryStatusField,
    ) -> Self {
        Self {
            coarse_battery_voltage,
            battery_status,
            _reserved: Default::default(),
        }
    }
}

/// This struct represents datapage 7 in the heart rate profile.
#[derive(PackedStruct, DataPage, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct BatteryStatus {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1")]
    pub battery_level: u8,
    #[packed_field(bytes = "2")]
    pub fractional_battery_voltage: u8,
    #[packed_field(bytes = "3")]
    pub descriptive_bit_field: DescriptiveBitField,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

impl BatteryStatus {
    pub fn new(
        page_change_toggle: bool,
        battery_level: u8,
        fractional_battery_voltage: u8,
        coarse_battery_voltage: Integer<u8, packed_bits::Bits4>,
        battery_status: BatteryStatusField,
        common: CommonData,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::BatteryStatus.into(),
            page_change_toggle,
            battery_level,
            fractional_battery_voltage,
            descriptive_bit_field: DescriptiveBitField::new(coarse_battery_voltage, battery_status),
            common,
        }
    }
}

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum HeartbeatEventType {
    MeasuredTimestamp = 0,
    ComputedTimestamp = 1,
}

/// This struct represents datapage 9 in the heart rate profile.
#[derive(PackedStruct, DataPage, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct DeviceInformation {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bits = "8:13")]
    _reserved0: ReservedOnes<packed_bits::Bits6>,
    #[packed_field(bits = "14:15", ty = "enum")]
    pub heartbeat_event_type: HeartbeatEventType,
    #[packed_field(bytes = "2:3")]
    _reserved1: ReservedOnes<packed_bits::Bits16>,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

impl DeviceInformation {
    pub fn new(
        page_change_toggle: bool,
        heartbeat_event_type: HeartbeatEventType,
        common: CommonData,
    ) -> Self {
        Self {
            data_page_number: DataPageNumbers::DeviceInformation.into(),
            page_change_toggle,
            _reserved0: Default::default(),
            heartbeat_event_type,
            _reserved1: Default::default(),
            common,
        }
    }
}

#[derive(PackedStruct, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct ApplyField {
    #[packed_field(bits = "0:6")]
    _reserved: ReservedOnes<packed_bits::Bits7>,
    #[packed_field(bits = "7")]
    pub gym_mode: bool,
}

#[derive(PackedStruct, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct FeatureField {
    #[packed_field(bits = "0:6")]
    _reserved: ReservedZeroes<packed_bits::Bits7>,
    #[packed_field(bits = "7")]
    pub gym_mode: bool,
}

/// This struct represents datapage 32 in the heart rate profile.
#[derive(PackedStruct, DataPage, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "8")]
pub struct HRFeatureCommand {
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1:5")]
    _reserved: ReservedOnes<packed_bits::Bits40>,
    #[packed_field(bytes = "6")]
    pub apply: ApplyField,
    #[packed_field(bytes = "7")]
    pub features: FeatureField,
}

impl HRFeatureCommand {
    pub fn new(apply: ApplyField, features: FeatureField) -> Self {
        Self {
            data_page_number: DataPageNumbers::HRFeatureCommand.to_primitive(),
            _reserved: Default::default(),
            apply,
            features,
        }
    }
}

/// This struct represents datapage 112-127 in the heart rate profile.
/// The data section is open to interpretation by the implementer
#[derive(PackedStruct, DataPage, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "8")]
pub struct ManufacturerSpecific {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1:3")]
    pub data: [u8; 3],
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

impl ManufacturerSpecific {
    pub fn new(
        data_page_number: Integer<u8, packed_bits::Bits7>,
        page_change_toggle: bool,
        data: [u8; 3],
        common: CommonData,
    ) -> Self {
        Self {
            data_page_number,
            page_change_toggle,
            data,
            common,
        }
    }
}

// TODO invert tests to check bytes so reserved fields are checked
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_data() {
        let packed = CommonData::new(0xFFAA, 2, 3).pack().unwrap();
        assert_eq!(packed, [0xAA, 0xFF, 2, 3]);
    }

    #[test]
    fn default_datapage() {
        let packed = DefaultDataPage::new(true, CommonData::new(0x1122, 3, 4))
            .pack()
            .unwrap();
        assert_eq!(packed, [128, 0xff, 0xff, 0xff, 0x22, 0x11, 3, 4]);
    }

    #[test]
    fn cumulative_operating_time() {
        // TODO
    }

    #[test]
    fn manufacturers_information() {
        let packed = ManufacturerInformation::new(true, 56, 12345, CommonData::new(0, 0, 0x7B))
            .pack()
            .unwrap();

        assert_eq!(packed, [0x82, 0x38, 0x39, 0x30, 0x00, 0x00, 0x00, 0x7B]);
    }

    #[test]
    fn product_information() {
        let packed = ProductInformation::new(true, 127, 23, 51, CommonData::new(0xE000, 0x0D, 0))
            .pack()
            .unwrap();

        assert_eq!(packed, [0x83, 0x7F, 0x17, 0x33, 0x00, 0xE0, 0x0D, 0x00]);
    }

    #[test]
    fn previous_heart_beat() {
        // TODO
    }

    #[test]
    fn swim_interval_summary() {
        // TODO
    }

    #[test]
    fn capabilities() {
        let unpacked =
            Capabilities::unpack(&[0x06, 0xFF, 0xC6, 0x82, 0x00, 0x00, 0x20, 0x00]).unwrap();

        assert_eq!(unpacked.features_supported.extended_running_features, false);
        assert_eq!(unpacked.features_supported.extended_cycling_features, true);
        assert_eq!(unpacked.features_supported.extended_swimming_features, true);
        assert_eq!(unpacked.features_supported.gym_mode, false);
        assert_eq!(
            unpacked.features_supported.manufacturer_specific_feature_0,
            true
        );
        assert_eq!(
            unpacked.features_supported.manufacturer_specific_feature_1,
            true
        );
        assert_eq!(unpacked.features_enabled.extended_running_features, false);
        assert_eq!(unpacked.features_enabled.extended_cycling_features, true);
        assert_eq!(unpacked.features_enabled.extended_swimming_features, false);
        assert_eq!(unpacked.features_enabled.gym_mode, false);
        assert_eq!(
            unpacked.features_enabled.manufacturer_specific_feature_0,
            false
        );
        assert_eq!(
            unpacked.features_enabled.manufacturer_specific_feature_1,
            true
        );
    }

    #[test]
    fn battery_status() {
        // TODO
    }

    #[test]
    fn device_information() {
        let pack = DeviceInformation::new(
            true,
            HeartbeatEventType::ComputedTimestamp,
            CommonData::new(111 << 8 | 183, 242, 93),
        )
        .pack()
        .unwrap();
        assert_eq!([137, 253, 255, 255, 183, 111, 242, 93], pack);
    }

    #[test]
    fn hr_feature_command() {
        // TODO
    }

    #[test]
    fn manufacter_specific() {
        // TODO
    }
}
