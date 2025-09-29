pub use crate::plus::common::datapages::BatteryStatusField;
use ant_derive::DataPage;
use derive_new::new;
use packed_struct::prelude::*;

pub const DATA_PAGE_NUMBER_MASK: u8 = 0x7F;

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum DataPageNumbers {
    MainDataPage = 16,
    PowerDataPage = 25,
}

impl From<DataPageNumbers> for Integer<u8, packed_bits::Bits<7>> {
    fn from(dp: DataPageNumbers) -> Self {
        dp.to_primitive().into()
    }
}

#[derive(PackedStruct, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct MainDataPage {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1")]
    pub equiment_type: u8,
    #[packed_field(bytes = "2")]
    pub elapsed_time: u8,
    #[packed_field(bytes = "3")]
    pub distance: u8,
    #[packed_field(bytes = "4:5")]
    pub speed: u16,
    #[packed_field(bytes = "6")]
    pub heart_rate: u8,
    #[packed_field(bytes = "7")]
    pub cap_state_bf: u8,
}

#[derive(PackedStruct, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct PowerDataPage {
    #[packed_field(bits = "1:7")]
    data_page_number: Integer<u8, packed_bits::Bits<7>>,
    #[packed_field(bits = "0")]
    pub page_change_toggle: bool,
    #[packed_field(bytes = "1")]
    pub event_count: u8,
    #[packed_field(bytes = "2")]
    pub cadance: u8,
    #[packed_field(bytes = "3:4")]
    pub accumulated_power: u16,
    #[packed_field(bits = "40:51")]
    pub instantaneous_power: Integer<u16, packed_bits::Bits<12>>,
    #[packed_field(bits = "52:55")]
    pub trainer_status: u8,
    #[packed_field(bytes = "7")]
    pub flag_state_bf: u8,
}

#[derive(PackedStruct, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct BasicResistanceDataPage {
    #[packed_field(byte = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1:6")]
    pub reserved: [u8; 6],
    #[packed_field(bytes = "7")]
    pub total_resistance: u8,
}

#[derive(PackedStruct, DataPage, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct TargetPowerDataPage {
    #[packed_field(byte = "0")]
    data_page_number: u8,
    #[packed_field(bytes = "1:5")]
    pub reserved: [u8; 5],
    #[packed_field(bytes = "6")]
    pub total_power_lsb: u8,
    #[packed_field(bytes = "7")]
    pub total_power_rsb: u8,
}