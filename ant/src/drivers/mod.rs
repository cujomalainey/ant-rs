// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod serial;
#[cfg(feature = "usb")]
mod usb;

pub use serial::*;
#[cfg(feature = "usb")]
pub use usb::*;

use crate::messages::channel::{ChannelEvent, ChannelResponse};
use crate::messages::data::{
    AcknowledgedData, AdvancedBurstData, BroadcastData, BurstTransferData,
};
use crate::messages::notifications::{SerialErrorMessage, StartUpMessage};
use crate::messages::requested_response::{
    AdvancedBurstCapabilities, AdvancedBurstCurrentConfiguration, AntVersion, Capabilities,
    ChannelId, ChannelStatus, EncryptionModeParameters, EventBufferConfiguration, EventFilter,
    SelectiveDataUpdateMaskSetting, SerialNumber, UserNvm,
};
use crate::messages::{
    AntMessage, RxMessage, RxMessageHeader, RxMessageId, RxSyncByte, TransmitableMessage, TxMessageHeader,
    TxSyncByte, MAX_MESSAGE_DATA_SIZE,
};

use arrayvec::{ArrayVec, CapacityError};
use embedded_hal::digital::v2::PinState;
use packed_struct::prelude::{PackedStructSlice, PackingError};
use std::array::TryFromSliceError;
use std::cmp;

pub trait Driver<R, W> {
    fn get_message(&mut self) -> Result<Option<AntMessage>, DriverError<R, W>>;
    fn send_message(&mut self, msg: &dyn TransmitableMessage) -> Result<(), DriverError<R, W>>;
}

// TODO finalize
const ANT_MESSAGE_SIZE: usize = MAX_MESSAGE_DATA_SIZE;
const CHECKSUM_SIZE: usize = 1;

#[derive(Debug)]
pub enum DriverError<R, W> {
    ReadError(nb::Error<R>),
    WriteError(nb::Error<W>),
    BadChecksum(u8, u8),
    BadLength(usize, usize),
    PackingError(PackingError),
    ReferenceError(),
    InvalidData(),
    BufferTooSmall(usize, usize),
    SliceError(TryFromSliceError),
    CapacityError(CapacityError),
    PinChangeBug(PinState), // TODO update this to use the type provided by the pin trait
}

impl<R, W> std::cmp::PartialEq for DriverError<R, W> {
    fn eq(&self, other: &Self) -> bool {
        use std::mem::discriminant;
        discriminant(self) == discriminant(other)
    }
}

impl<R, W> From<packed_struct::PackingError> for DriverError<R, W> {
    fn from(err: packed_struct::PackingError) -> Self {
        DriverError::PackingError(err)
    }
}

impl<R, W> From<TryFromSliceError> for DriverError<R, W> {
    fn from(err: TryFromSliceError) -> Self {
        DriverError::SliceError(err)
    }
}

impl<R, W> From<arrayvec::CapacityError> for DriverError<R, W> {
    fn from(err: arrayvec::CapacityError) -> Self {
        DriverError::CapacityError(err)
    }
}

fn calculate_checksum(buf: &[u8]) -> u8 {
    buf.iter().fold(0, |acc, x| acc ^ x)
}

fn align_buffer(buf: &[u8]) -> usize {
    if !buf.is_empty() {
        // TODO analyze this, shouldnt we toss the buffer in this case?
        let msg_start = buf
            .iter()
            .position(|&x| x == (RxSyncByte::Write as u8))
            .unwrap_or(0);
        return msg_start;
    }
    0
}

fn update_buffer<R, W>(msg: &Result<Option<AntMessage>, DriverError<R, W>>, buf: &[u8]) -> usize {
    if msg.is_err() {
        // It was a corrupted message, skip first byte to resposition buf and move on
        return 1;
    } else if let Ok(Some(data)) = msg {
        // This check is simply to make sure we don't panic in the event a message somehow
        // mis-represented its size and we were able to parse it still correctly. Specificly
        // the case where len > buf len
        let amount = cmp::min(
            (data.header.msg_length as usize) + HEADER_SIZE + CHECKSUM_SIZE,
            buf.len(),
        );
        return amount;
    }
    0
}


fn create_packed_message<'a>(
    buf: &'a mut [u8],
    msg: &dyn TransmitableMessage,
) -> Result<&'a [u8], PackingError> {
    let msg_len = msg.serialize_message(&mut buf[HEADER_SIZE..])?;
    let header = TxMessageHeader {
        sync: TxSyncByte::Value,
        msg_length: msg_len as u8,
        msg_id: msg.get_tx_msg_id(),
    };

    let padded_len = msg_len + HEADER_SIZE;
    header.pack_to_slice(&mut buf[..HEADER_SIZE])?;
    buf[padded_len] = calculate_checksum(&buf[..padded_len]);

    Ok(&buf[..padded_len + 1])
}

const HEADER_SIZE: usize = 3;

type Buffer = ArrayVec<u8, ANT_MESSAGE_SIZE>;

fn parse_buffer<R, W>(buf: &[u8]) -> Result<Option<AntMessage>, DriverError<R, W>> {
    // Not enough bytes
    if buf.len() < HEADER_SIZE {
        return Ok(None);
    }

    // no need to check sync byte as we already used that to position ourselves
    let header = RxMessageHeader::unpack_from_slice(&buf[..HEADER_SIZE])?;
    let msg_size = (header.msg_length as usize) + HEADER_SIZE + CHECKSUM_SIZE;

    // TODO
    // if buf.capacity() < msg_size {
    //     return Err(DriverError::BufferTooSmall(msg_size, buf.capacity()));
    // }

    if buf.len() < msg_size {
        return Ok(None);
    }

    let expected_checksum = calculate_checksum(&buf[..(header.msg_length as usize) + HEADER_SIZE]);
    let checksum = buf[(header.msg_length as usize) + HEADER_SIZE];
    if expected_checksum != checksum {
        return Err(DriverError::BadChecksum(checksum, expected_checksum));
    }

    let msg_slice = &buf[HEADER_SIZE..(header.msg_length as usize) + HEADER_SIZE];

    let body = match header.msg_id {
        RxMessageId::StartUpMessage => {
            RxMessage::StartUpMessage(StartUpMessage::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::SerialErrorMessage => {
            RxMessage::SerialErrorMessage(SerialErrorMessage::unpack_from_slice(msg_slice)?)
        }

        RxMessageId::BroadcastData => {
            RxMessage::BroadcastData(BroadcastData::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::AcknowledgedData => {
            RxMessage::AcknowledgedData(AcknowledgedData::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::BurstTransferData => {
            RxMessage::BurstTransferData(BurstTransferData::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::AdvancedBurstData => {
            RxMessage::AdvancedBurstData(AdvancedBurstData::unpack_from_slice(msg_slice)?)
        }

        RxMessageId::ChannelEvent => {
            if msg_slice[1] == 1 {
                RxMessage::ChannelEvent(ChannelEvent::unpack_from_slice(msg_slice)?)
            } else {
                RxMessage::ChannelResponse(ChannelResponse::unpack_from_slice(msg_slice)?)
            }
        }
        RxMessageId::ChannelStatus => {
            RxMessage::ChannelStatus(ChannelStatus::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::ChannelId => RxMessage::ChannelId(ChannelId::unpack_from_slice(msg_slice)?),

        RxMessageId::AntVersion => RxMessage::AntVersion(AntVersion::unpack_from_slice(msg_slice)?),
        RxMessageId::Capabilities => {
            RxMessage::Capabilities(Capabilities::unpack_from_slice(msg_slice)?)
        }

        RxMessageId::SerialNumber => {
            RxMessage::SerialNumber(SerialNumber::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::EventBufferConfiguration => RxMessage::EventBufferConfiguration(
            EventBufferConfiguration::unpack_from_slice(msg_slice)?,
        ),

        RxMessageId::AdvancedBurstCapabilities => match buf.len() {
            5 => RxMessage::AdvancedBurstCapabilities(
                AdvancedBurstCapabilities::unpack_from_slice(msg_slice)?,
            ),
            12 => RxMessage::AdvancedBurstCurrentConfiguration(
                AdvancedBurstCurrentConfiguration::unpack_from_slice(msg_slice)?,
            ),
            _ => return Err(DriverError::BadLength(0, buf.len())),
        },

        RxMessageId::EventFilter => {
            RxMessage::EventFilter(EventFilter::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::SelectiveDataUpdateMaskSetting => RxMessage::SelectiveDataUpdateMaskSetting(
            SelectiveDataUpdateMaskSetting::unpack_from_slice(msg_slice)?,
        ),

        // TODO handle data payload
        RxMessageId::UserNvm => RxMessage::UserNvm(UserNvm::unpack_from_slice(msg_slice)?),

        RxMessageId::EncryptionModeParameters => RxMessage::EncryptionModeParameters(
            EncryptionModeParameters::unpack_from_slice(msg_slice)?,
        ),
    };

    Ok(Some(AntMessage {
        header,
        message: body,
        checksum,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::config::{
        AddChannelIdToList, DeviceType, TransmissionChannelType, TransmissionType,
    };

    #[test]
    fn checksum() {
        let data = [0xA4, 6, 0x59, 2, 0x44, 0x33, 120, 34, 2];
        assert_eq!(calculate_checksum(&data), 214);
    }

    #[test]
    fn message_packing() {
        let mut buf: [u8; 12] = [0; 12];
        let mut transmission_type = TransmissionType::default();
        transmission_type.device_number_extension = 2.into();
        transmission_type.transmission_channel_type =
            TransmissionChannelType::SharedChannel1ByteAddress;
        create_packed_message(
            &mut buf,
            &AddChannelIdToList {
                channel_number: 2,
                device_number: 0x3344,
                device_type: DeviceType {
                    device_type_id: 120.into(),
                    ..DeviceType::default()
                },
                transmission_type,
                list_index: 2,
            },
        )
        .unwrap();

        assert_eq!(buf, [0xA4, 6, 0x59, 2, 0x44, 0x33, 120, 34, 2, 214, 0, 0]);
    }
}
