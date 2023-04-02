use crate::messages::{TransmitableMessage, TxMessage, TxMessageId};
use ant_derive::AntTx;
use packed_struct::prelude::*;

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct CwInit {
    #[packed_field(bytes = "0")]
    filler: ReservedZeroes<packed_bits::Bits8>,
}

impl CwInit {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct CwTest {
    #[packed_field(bytes = "0")]
    filler: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bytes = "1")]
    pub transmit_power: u8,
    #[packed_field(bytes = "2")]
    pub channel_rf_frequency: u8,
}

impl CwTest {
    pub fn new(transmit_power: u8, channel_rf_frequency: u8) -> Self {
        Self {
            transmit_power,
            channel_rf_frequency,
            ..Self::default()
        }
    }
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
