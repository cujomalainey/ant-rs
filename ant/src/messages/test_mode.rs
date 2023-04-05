use crate::messages::{TransmitableMessage, TxMessage, TxMessageId};
use ant_derive::AntTx;
use derive_new::new;
use packed_struct::prelude::*;

#[derive(PackedStruct, AntTx, new, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct CwInit {
    #[new(default)]
    #[packed_field(bytes = "0")]
    filler: ReservedZeroes<packed_bits::Bits8>,
}

#[derive(PackedStruct, AntTx, new, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct CwTest {
    #[new(default)]
    #[packed_field(bytes = "0")]
    filler: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bytes = "1")]
    pub transmit_power: u8,
    #[packed_field(bytes = "2")]
    pub channel_rf_frequency: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cw_init() -> Result<(), PackingError> {
        let packed = CwInit::new();
        assert_eq!(packed.pack()?, [0]);
        Ok(())
    }

    #[test]
    fn cw_test() {
        let packed = CwTest::new(1, 2);
        assert_eq!(packed.pack().unwrap(), [0, 1, 2]);
    }
}
