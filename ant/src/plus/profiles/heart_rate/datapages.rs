// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use crate::plus::common::datapages::BatteryStatusField;
use ant_derive::DataPage;
use derive_new::new;
use packed_struct::prelude::*;

// TODO add is_valid checks to fields
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

impl From<DataPageNumbers> for Integer<u8, packed_bits::Bits<7>> {
    fn from(dp: DataPageNumbers) -> Self {
        dp.to_primitive().into()
    }
}

/// The last 4 bytes in every message in the heart rate profile are the same, this maps out those
/// fields
#[derive(PackedStruct, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "4")]
pub struct CommonData {
    #[packed_field(bytes = "0:1")]
    pub heart_beat_event_time: u16,
    #[packed_field(bytes = "2")]
    pub heart_beat_count: u8,
    #[packed_field(bytes = "3")]
    pub computed_heart_rate: u8,
}

/// This struct represents datapage 0 in the heart rate profile.
#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct DefaultDataPage {
    #[new(value = "DataPageNumbers::DefaultDataPage.into()")]
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[new(default)]
    #[packed_field(bytes = "1:3")]
    _reserved: ReservedOnes<packed_bits::Bits<24>>,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

/// This struct represents datapage 1 in the heart rate profile.
#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct CumulativeOperatingTime {
    #[new(value = "DataPageNumbers::CumulativeOperatingTime.into()")]
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1:3")]
    pub cumulative_operating_time: Integer<u32, packed_bits::Bits<24>>,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

/// This struct represents datapage 2 in the heart rate profile.
#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct ManufacturerInformation {
    #[new(value = "DataPageNumbers::ManufacturerInformation.into()")]
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1")]
    pub manufacturer_id: u8,
    #[packed_field(bytes = "2:3")]
    pub serial_number: u16,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

/// This struct represents datapage 3 in the heart rate profile.
#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct ProductInformation {
    #[new(value = "DataPageNumbers::ProductInformation.into()")]
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
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

/// This struct represents datapage 4 in the heart rate profile.
#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct PreviousHeartBeat {
    #[new(value = "DataPageNumbers::PreviousHeartBeat.into()")]
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    /// Set to [PreviousHeartBeat::MANUFACTURER_SPECIFIC_UNUSED] if you do not use this field
    #[packed_field(bytes = "1")]
    pub manufacturer_specific: u8,
    #[packed_field(bytes = "2:3")]
    pub previous_heart_beat_event_time: u16,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

impl PreviousHeartBeat {
    /// Value for unused [PreviousHeartBeat::manufacturer_specific]
    pub const MANUFACTURER_SPECIFIC_UNUSED: u8 = 0xFF;
}

/// This struct represents datapage 5 in the heart rate profile.
/// Monitors don't need to implement this unless they support the Swimming feature.
#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct SwimIntervalSummary {
    #[new(value = "DataPageNumbers::SwimIntervalSummary.into()")]
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
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

#[derive(PackedStruct, new, PartialEq, Copy, Clone, Debug)]
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
    #[new(default)]
    #[packed_field(bits = "4:5")]
    _reserved: ReservedZeroes<packed_bits::Bits<3>>,
    #[packed_field(bits = "6")]
    pub manufacturer_specific_feature_0: bool,
    #[packed_field(bits = "7")]
    pub manufacturer_specific_feature_1: bool,
}

/// This struct represents datapage 6 in the heart rate profile.
#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct Capabilities {
    #[new(value = "DataPageNumbers::Capabilities.into()")]
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[new(default)]
    #[packed_field(bytes = "1")]
    _reserved: ReservedOnes<packed_bits::Bits<8>>,
    #[packed_field(bytes = "2")]
    pub features_supported: Features,
    #[packed_field(bytes = "3")]
    pub features_enabled: Features,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

// Note we cannot reuse the common datapage battery fields because HR does not define bit 7
#[derive(PackedStruct, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct DescriptiveBitField {
    #[packed_field(bits = "0:3")]
    pub coarse_battery_voltage: Integer<u8, packed_bits::Bits<4>>,
    #[packed_field(bits = "4:6", ty = "enum")]
    pub battery_status: BatteryStatusField,
    #[new(default)]
    #[packed_field(bits = "7")]
    _reserved: ReservedZeroes<packed_bits::Bits<1>>,
}

/// This struct represents datapage 7 in the heart rate profile.
#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct BatteryStatus {
    #[new(value = "DataPageNumbers::BatteryStatus.into()")]
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
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

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum HeartbeatEventType {
    MeasuredTimestamp = 0,
    ComputedTimestamp = 1,
}

/// This struct represents datapage 9 in the heart rate profile.
#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct DeviceInformation {
    #[new(value = "DataPageNumbers::DeviceInformation.into()")]
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[new(default)]
    #[packed_field(bits = "8:13")]
    _reserved0: ReservedOnes<packed_bits::Bits<6>>,
    #[packed_field(bits = "14:15", ty = "enum")]
    pub heartbeat_event_type: HeartbeatEventType,
    #[new(default)]
    #[packed_field(bytes = "2:3")]
    _reserved1: ReservedOnes<packed_bits::Bits<16>>,
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

#[derive(PackedStruct, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct ApplyField {
    #[new(default)]
    #[packed_field(bits = "0:6")]
    _reserved: ReservedOnes<packed_bits::Bits<7>>,
    #[packed_field(bits = "7")]
    pub gym_mode: bool,
}

#[derive(PackedStruct, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct FeatureField {
    #[new(default)]
    #[packed_field(bits = "0:6")]
    _reserved: ReservedZeroes<packed_bits::Bits<7>>,
    #[packed_field(bits = "7")]
    pub gym_mode: bool,
}

/// This struct represents datapage 32 in the heart rate profile.
#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct HRFeatureCommand {
    #[new(value = "DataPageNumbers::HRFeatureCommand.to_primitive()")]
    #[packed_field(bytes = "0")]
    data_page_number: u8,
    #[new(default)]
    #[packed_field(bytes = "1:5")]
    _reserved: ReservedOnes<packed_bits::Bits<40>>,
    #[packed_field(bytes = "6")]
    pub apply: ApplyField,
    #[packed_field(bytes = "7")]
    pub features: FeatureField,
}

/// This struct represents datapage 112-127 in the heart rate profile.
/// The data section is open to interpretation by the implementer
#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "8")]
pub struct ManufacturerSpecific {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1:3")]
    pub data: [u8; 3],
    #[packed_field(bytes = "4:7")]
    pub common: CommonData,
}

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
        let packed =
            CumulativeOperatingTime::new(false, 0xAABBCC.into(), CommonData::new(0x1122, 3, 4))
                .pack()
                .unwrap();
        assert_eq!(packed, [1, 0xCC, 0xBB, 0xAA, 0x22, 0x11, 3, 4]);
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
        let packed = PreviousHeartBeat::new(
            false,
            PreviousHeartBeat::MANUFACTURER_SPECIFIC_UNUSED,
            0xAABB,
            CommonData::new(0xE000, 0x0D, 0),
        )
        .pack()
        .unwrap();

        assert_eq!(packed, [4, 0xFF, 0xBB, 0xAA, 0x00, 0xE0, 0x0D, 0x00]);
    }

    #[test]
    fn swim_interval_summary() {
        let packed =
            SwimIntervalSummary::new(false, 0xCC, 0xAA, 0xBB, CommonData::new(0xE000, 0x0D, 0))
                .pack()
                .unwrap();

        assert_eq!(packed, [5, 0xCC, 0xAA, 0xBB, 0x00, 0xE0, 0x0D, 0x00]);
    }

    #[test]
    fn capabilities() {
        let packed = Capabilities::new(
            false,
            Features::new(false, true, true, false, true, true),
            Features::new(false, true, false, false, false, true),
            CommonData::new(0xE000, 0x0D, 0),
        )
        .pack()
        .unwrap();

        assert_eq!(packed, [0x06, 0xFF, 0xC6, 0x82, 0x00, 0xE0, 0x0D, 0x00]);
    }

    #[test]
    fn battery_status() {
        let packed = BatteryStatus::new(
            false,
            0x0F,
            0xAA,
            DescriptiveBitField::new(0xF.into(), BatteryStatusField::Invalid),
            CommonData::new(0xE000, 0x0D, 0),
        )
        .pack()
        .unwrap();

        assert_eq!(packed, [7, 0x0F, 0xAA, 0x7F, 0x00, 0xE0, 0x0D, 0x00]);
    }

    #[test]
    fn device_information() {
        let pack = DeviceInformation::new(
            true,
            HeartbeatEventType::ComputedTimestamp,
            CommonData::new(0xFFAA, 242, 93),
        )
        .pack()
        .unwrap();
        assert_eq!([137, 253, 255, 255, 0xAA, 0xFF, 242, 93], pack);
    }

    #[test]
    fn hr_feature_command() {
        let packed = HRFeatureCommand::new(ApplyField::new(true), FeatureField::new(false))
            .pack()
            .unwrap();

        assert_eq!(packed, [32, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
    }

    #[test]
    fn manufacter_specific() {
        let pack = ManufacturerSpecific::new(
            114.into(),
            false,
            [0xAA, 0xFF, 0xCC],
            CommonData::new(0xFFAA, 242, 93),
        )
        .pack()
        .unwrap();
        assert_eq!([114, 0xAA, 0xFF, 0xCC, 0xAA, 0xFF, 242, 93], pack);
    }
}
