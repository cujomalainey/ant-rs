// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::messages::{AntAutoPackWithExtention, TransmitableMessage, TxMessage, TxMessageId};
use ant_derive::AntTx;
use packed_struct::prelude::*;

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct ResetSystem {
    #[packed_field(bytes = "0")]
    filler: ReservedZeroes<packed_bits::Bits8>,
}

impl ResetSystem {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct OpenChannel {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
}

impl OpenChannel {
    pub fn new(channel_number: u8) -> Self {
        Self { channel_number }
    }
}

#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct CloseChannel {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
}

impl CloseChannel {
    pub fn new(channel_number: u8) -> Self {
        Self { channel_number }
    }
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum RequestableMessageId {
    ChannelStatus = 0x52,
    ChannelId = 0x51,
    AntVersion = 0x3E,
    Capabilities = 0x54,
    SerialNumber = 0x61,
    EventBufferConfiguration = 0x74,
    AdvancedBurstCapabilities = 0x78,
}

#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct RequestMessageData {
    #[packed_field(bytes = "0")]
    pub channel: u8,
    #[packed_field(bytes = "1", ty = "enum")]
    pub message_id: RequestableMessageId,
}

// TODO test
#[derive(PackedStruct, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct NvmeRequest {
    #[packed_field(bytes = "0:1")]
    pub addr: u16,
    #[packed_field(bytes = "2")]
    pub size: u8,
}

impl NvmeRequest {
    pub fn new(addr: u16, size: u8) -> Self {
        Self { addr, size }
    }
}

// TODO test
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestMessage {
    pub data: RequestMessageData,
    pub nvme_region: Option<NvmeRequest>,
}
AntAutoPackWithExtention!(
    RequestMessage,
    TxMessageId::RequestMessage,
    data,
    nvme_region
);

impl RequestMessage {
    pub fn new(
        channel: u8,
        message_id: RequestableMessageId,
        nvme_region: Option<NvmeRequest>,
    ) -> Self {
        Self {
            data: RequestMessageData {
                channel,
                message_id,
            },
            nvme_region,
        }
    }
}

// TODO implement serialize and test
// TODO implement new handler
pub struct OpenRxScanMode {
    pub synchronous_channel_packets_only: Option<bool>,
}

#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct SleepMessage {
    #[packed_field(bytes = "0")]
    filler: ReservedZeroes<packed_bits::Bits8>,
}

impl SleepMessage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reset_system() {
        let packed = ResetSystem::new();
        assert_eq!(packed.pack().unwrap(), [0]);
    }

    #[test]
    fn open_channel() {
        let packed = OpenChannel::new(0);
        assert_eq!(packed.pack().unwrap(), [0]);
    }

    #[test]
    fn close_channel() {
        let packed = CloseChannel::new(0);
        assert_eq!(packed.pack().unwrap(), [0]);
    }

    #[test]
    fn sleep_message() {
        let packed = SleepMessage::new();
        assert_eq!(packed.pack().unwrap(), [0]);
    }
}
