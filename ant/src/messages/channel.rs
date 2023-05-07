// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use packed_struct::prelude::*;

// Re-export types used in multiple scopes based on the datasheet
pub use crate::messages::requested_response::{EncryptionId, UserInformationString};
pub use crate::messages::TxMessageId;

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum MessageCode {
    ResponseNoError = 0x00,
    EventRxSearchTimeout = 0x01,
    EventRxFail = 0x02,
    EventTx = 0x03,
    EventTransferRxFailed = 0x04,
    EventTransferTxCompleted = 0x05,
    EventTransferTxFailed = 0x06,
    EventChannelClosed = 0x07,
    EventRxFailGoToSearch = 0x08,
    EventChannelCollision = 0x09,
    EventTransferTxStart = 0x0A,
    EventTransferNextDataBlock = 0x11,
    ChannelInWrongState = 0x15,
    ChannelNotOpened = 0x16,
    ChannelIdNotSet = 0x18,
    CloseAllChannels = 0x19,
    TransferInProgress = 0x1F,
    TransferSequenceNumberError = 0x20,
    TransferInError = 0x21,
    MessageSizeExceedsLimit = 0x27,
    InvalidMessage = 0x28,
    InvalidNetworkNumber = 0x29,
    InvalidListId = 0x30,
    InvalidScanTxChannel = 0x31,
    InvalidParameterProvided = 0x32,
    EventSerialQueOverflow = 0x34,
    EventQueOverflow = 0x35,
    EncryptNegotiationSuccess = 0x38,
    EncryptNegotiationFail = 0x39,
    NvmFullError = 0x40,
    NvmWriteError = 0x41,
    UsbStringWriteFail = 0x70,
    MesgSerialErrorId = 0xAE, // TODO verify how this behaves with "data portion"
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "3")]
pub struct ChannelEventPayload {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    #[packed_field(bits = "8:14")]
    _reserved0: ReservedZeroes<packed_bits::Bits<7>>,
    #[packed_field(bits = "15")]
    _reserved1: ReservedOnes<packed_bits::Bits<1>>,
    #[packed_field(bytes = "2", ty = "enum")]
    pub message_code: MessageCode,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ChannelEventExtension {
    EncryptNegotiationSuccess(EncryptionId, Option<UserInformationString>),
    EncryptNegotiationFail(EncryptionId),
}

// TODO On PC applications ADV burst comes in through this event type, need to add another layer of
// abstraction
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChannelEvent {
    pub payload: ChannelEventPayload,
    pub extended_info: Option<ChannelEventExtension>,
}

impl ChannelEvent {
    pub(crate) const MSG_ID: u8 = 1;
    pub(crate) const MSG_ID_INDEX: usize = 1;

    pub(crate) fn unpack_from_slice(data: &[u8]) -> Result<Self, PackingError> {
        let payload = ChannelEventPayload::unpack_from_slice(data)?;

        Ok(ChannelEvent {
            payload,
            // TODO extended_info,
            extended_info: None,
        })
    }
}

#[derive(PackedStruct, Debug, Clone, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct ChannelResponse {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    #[packed_field(bytes = "1", ty = "enum")]
    pub message_id: TxMessageId,
    #[packed_field(bytes = "2", ty = "enum")]
    pub message_code: MessageCode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_response() -> Result<(), PackingError> {
        let unpacked = ChannelResponse::unpack(&[1, 0x6E, 0x00])?;
        assert_eq!(unpacked.channel_number, 1);
        assert_eq!(unpacked.message_id, TxMessageId::LibConfig);
        assert_eq!(unpacked.message_code, MessageCode::ResponseNoError);
        Ok(())
    }

    #[test]
    fn channel_event() -> Result<(), PackingError> {
        // TODO test
        Ok(())
    }
}
