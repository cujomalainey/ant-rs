// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use packed_struct::prelude::*;

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum SerialErrorType {
    IncorrectSyncByte = 0x00,
    IncorrectChecksumByte = 0x02,
    IncorrectMessageLength = 0x03,
}

#[derive(PackedStruct, Debug, Clone, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", endian = "lsb", size_bytes = "1")]
pub struct StartUpMessage {
    #[packed_field(bits = "0")]
    pub hardware_reset_line: bool,
    #[packed_field(bits = "1")]
    pub watch_dog_reset: bool,
    #[packed_field(bits = "5")]
    pub command_reset: bool,
    #[packed_field(bits = "6")]
    pub synchronous_reset: bool,
    #[packed_field(bits = "7")]
    pub suspend_reset: bool,
}

impl StartUpMessage {
    /// Helper function to detect special bitfield case of power on reset cause
    // TODO test
    pub fn is_power_on_reset(&self) -> bool {
        !(self.hardware_reset_line
            || self.watch_dog_reset
            || self.command_reset
            || self.synchronous_reset
            || self.suspend_reset)
    }
}

// TODO spec says rest of data contains a copy of the error message, need to validate how this
// works on the usb in the field
// Note this message has a range up to 255
// TODO make a config so users can set TX and RX buffer sizes for embeded devices since only
// users of the USB devices need the full 256 bytes for NVMe
#[derive(PackedStruct, Debug, Clone, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct SerialErrorMessage {
    #[packed_field(bytes = "0", ty = "enum")]
    pub error_number: SerialErrorType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_message() {
        let unpacked = StartUpMessage::unpack(&[0x02]).unwrap();
        assert_eq!(unpacked.watch_dog_reset, true);
    }

    #[test]
    fn serial_error_message() {
        let unpacked = SerialErrorMessage::unpack(&[0x02]).unwrap();
        assert_eq!(
            unpacked.error_number,
            SerialErrorType::IncorrectChecksumByte
        );
    }
}
