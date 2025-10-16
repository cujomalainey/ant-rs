pub use crate::plus::common::datapages::BatteryStatusField;
use derive_new::new;
use packed_struct::prelude::*;

pub const DATA_PAGE_NUMBER_MASK: u8 = 0x7F;

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum DataPageNumbers {
    MainDataPage = 16,
}

impl From<DataPageNumbers> for Integer<u8, packed_bits::Bits<7>> {
    fn from(dp: DataPageNumbers) -> Self {
        dp.to_primitive().into()
    }
}

// TODO: Implement MainDataPage
#[derive(PackedStruct, new, PartialEq, Copy, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "8")]
pub struct MainDataPage {
    #[packed_field(bytes = "0:7")]
    _reserved: [u8; 8],
}