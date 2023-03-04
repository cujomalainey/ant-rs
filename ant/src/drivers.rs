// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use crate::fields::{RxMessageHeader, RxMessageId, TxMessageHeader, TxSyncByte};
use crate::messages::*;
use arrayvec::{ArrayVec, CapacityError};
use embedded_hal::digital::v2::{OutputPin, PinState};
use embedded_hal::serial::Read;
use embedded_hal::serial::Write;
use nb;
use packed_struct::{PackedStructSlice, PackingError};
use std::array::TryFromSliceError;
use std::cell::RefCell;
use std::cmp;
use thiserror::Error;

pub trait Driver<R, W> {
    fn get_message(&mut self) -> Result<Option<AntMessage>, DriverError<R, W>>;
    fn send_message(&mut self, msg: &dyn AntTxMessageType) -> Result<(), DriverError<R, W>>;
}

// TODO finalize
const ANT_MESSAGE_SIZE: usize = MAX_MESSAGE_DATA_SIZE;
const CHECKSUM_SIZE: usize = 1;

type Buffer = ArrayVec<u8, ANT_MESSAGE_SIZE>;

pub struct SerialDriver<SERIAL, PIN> {
    serial: SERIAL,
    sleep: Option<PIN>,
    buffer: RefCell<Buffer>, // TODO change this dependency injection so user controls the size
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

const HEADER_SIZE: usize = 3;

impl<SERIAL, SLEEP> SerialDriver<SERIAL, SLEEP>
where
    SERIAL: Read<u8> + Write<u8>,
    SLEEP: OutputPin,
{
    pub fn new(serial: SERIAL, sleep: Option<SLEEP>) -> SerialDriver<SERIAL, SLEEP> {
        SerialDriver {
            serial,
            sleep,
            buffer: RefCell::new(Buffer::new()),
        }
    }

    pub fn release(self) -> (SERIAL, Option<SLEEP>) {
        (self.serial, self.sleep)
    }
}

fn update_buffer<R, W>(msg: &Result<Option<AntMessage>, DriverError<R, W>>, buf: &mut Buffer) {
    if msg.is_err() {
        // It was a corrupted message, skip first byte to resposition buf and move on
        buf.remove(0);
    } else if let Ok(Some(data)) = msg {
        // This check is simply to make sure we don't panic in the event a message somehow
        // mis-represented its size and we were able to parse it still correctly. Specificly
        // the case where len > buf len
        let amount = cmp::min(
            (data.header.msg_length as usize) + HEADER_SIZE + CHECKSUM_SIZE,
            buf.len(),
        );
        buf.drain(..amount);
    }
}

fn parse_buffer<R, W>(buf: &Buffer) -> Result<Option<AntMessage>, DriverError<R, W>> {
    // Not enough bytes
    if buf.len() < HEADER_SIZE {
        return Ok(None);
    }

    // no need to check sync byte as we already used that to position ourselves
    let header = RxMessageHeader::unpack_from_slice(&buf[..HEADER_SIZE])?;
    let msg_size = (header.msg_length as usize) + HEADER_SIZE + CHECKSUM_SIZE;

    if buf.capacity() < msg_size {
        return Err(DriverError::BufferTooSmall(msg_size, buf.capacity()));
    }

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
            RxMessageType::StartUpMessage(StartUpMessage::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::SerialErrorMessage => {
            RxMessageType::SerialErrorMessage(SerialErrorMessage::unpack_from_slice(msg_slice)?)
        }

        RxMessageId::BroadcastData => {
            RxMessageType::BroadcastData(BroadcastData::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::AcknowledgedData => {
            RxMessageType::AcknowledgedData(AcknowledgedData::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::BurstTransferData => {
            RxMessageType::BurstTransferData(BurstTransferData::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::AdvancedBurstData => {
            RxMessageType::AdvancedBurstData(AdvancedBurstData::unpack_from_slice(msg_slice)?)
        }

        RxMessageId::ChannelEvent => {
            if msg_slice[1] == 1 {
                RxMessageType::ChannelEvent(ChannelEvent::unpack_from_slice(msg_slice)?)
            } else {
                RxMessageType::ChannelResponse(ChannelResponse::unpack_from_slice(msg_slice)?)
            }
        }
        // TODO replace this with the optional field handling
        RxMessageId::ChannelStatus => {
            RxMessageType::ChannelStatus(ChannelStatus::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::ChannelId => {
            RxMessageType::ChannelId(ChannelId::unpack_from_slice(msg_slice)?)
        }

        RxMessageId::AntVersion => {
            RxMessageType::AntVersion(AntVersion::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::Capabilities => {
            RxMessageType::Capabilities(Capabilities::unpack_from_slice(msg_slice)?)
        }

        RxMessageId::SerialNumber => {
            RxMessageType::SerialNumber(SerialNumber::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::EventBufferConfiguration => RxMessageType::EventBufferConfiguration(
            EventBufferConfiguration::unpack_from_slice(msg_slice)?,
        ),

        RxMessageId::AdvancedBurstCapabilities => match buf.len() {
            5 => RxMessageType::AdvancedBurstCapabilities(
                AdvancedBurstCapabilities::unpack_from_slice(msg_slice)?,
            ),
            12 => RxMessageType::AdvancedBurstConfiguration(
                AdvancedBurstCurrentConfiguration::unpack_from_slice(msg_slice)?,
            ),
            _ => return Err(DriverError::BadLength(0, buf.len())),
        },

        RxMessageId::EventFilter => {
            RxMessageType::EventFilter(EventFilter::unpack_from_slice(msg_slice)?)
        }
        RxMessageId::SelectiveDataUpdateMaskSetting => {
            RxMessageType::SelectiveDataUpdateMaskSetting(
                SelectiveDataUpdateMaskSetting::unpack_from_slice(msg_slice)?,
            )
        }

        // TODO handle data payload
        RxMessageId::UserNvm => RxMessageType::UserNvm(UserNvm::unpack_from_slice(msg_slice)?),

        RxMessageId::EncryptionModeParameters => RxMessageType::EncryptionModeParameters(
            EncryptionModeParameters::unpack_from_slice(msg_slice)?,
        ),
    };

    Ok(Some(AntMessage {
        header,
        message: body,
        checksum,
    }))
}

const SYNC_BYTE: u8 = 0xA4;

fn calculate_checksum(buf: &[u8]) -> u8 {
    buf.iter().fold(0, |acc, x| acc ^ x)
}

// TODO implement SPI driver
// check for write byte? maybe, not sure 0xA5

impl<SERIAL, SLEEP, R, W> Driver<R, W> for SerialDriver<SERIAL, SLEEP>
where
    SERIAL: Read<u8, Error = R> + Write<u8, Error = W>,
    SLEEP: OutputPin,
{
    fn get_message(&mut self) -> Result<Option<AntMessage>, DriverError<R, W>> {
        // if we recursed this will fail
        let mut buf = match self.buffer.try_borrow_mut() {
            Err(_) => return Err(DriverError::ReferenceError()),
            Ok(b) => b,
        };

        // Attempt to parse remaining contents of the buffer before reading
        // find the start of a message if not a usb driver. USB does bulk transfer with full
        // message per transfer so this logic does not apply.

        if !buf.is_empty() {
            // TODO analyze this, shouldnt we toss the buffer in this case?
            let msg_start = buf.iter().position(|&x| x == SYNC_BYTE).unwrap_or(0);
            if msg_start != 0 {
                buf.drain(msg_start..);
            }
        }
        let msg = parse_buffer(&buf);

        update_buffer(&msg, &mut buf);

        if Ok(None) != msg {
            return msg;
        }

        loop {
            let data = self.serial.read();
            match data {
                Ok(d) => buf.push(d),
                Err(nb::Error::WouldBlock) => break,
                Err(e) => return Err(DriverError::ReadError(e)),
            }

            if buf.is_full() {
                break;
            }
        }

        let msg_result = parse_buffer(&buf);

        update_buffer(&msg_result, &mut buf);

        msg_result
    }

    fn send_message(&mut self, msg: &dyn AntTxMessageType) -> Result<(), DriverError<R, W>> {
        // TODO update with variable sized buf
        // TODO add sleep pin handling
        // TODO fix io error propotation
        let mut buf: [u8; ANT_MESSAGE_SIZE] = [0; ANT_MESSAGE_SIZE];

        let buf_slice = create_packed_message(&mut buf, msg)?;

        if let Some(pin) = &mut self.sleep {
            // TODO to return type eventually
            if pin.set_low().is_err() {
                return Err(DriverError::PinChangeBug(PinState::Low));
            }
        }

        // TODO handle case where driver is full, flush and keep going or switch to blocking API
        for byte in buf_slice.iter() {
            if let Err(e) = self.serial.write(*byte) {
                return Err(DriverError::WriteError(e));
            }
        }

        if let Err(e) = self.serial.flush() {
            return Err(DriverError::WriteError(e));
        }

        if let Some(pin) = &mut self.sleep {
            if pin.set_high().is_err() {
                return Err(DriverError::PinChangeBug(PinState::High));
            }
        }

        Ok(())
    }
}

fn create_packed_message<'a, R, W>(
    buf: &'a mut [u8],
    msg: &dyn AntTxMessageType,
) -> Result<&'a [u8], DriverError<R, W>> {
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

#[derive(Error, Debug)]
pub enum DriverError<R, W> {
    #[error("IO read error: {0}")]
    ReadError(nb::Error<R>),
    #[error("IO write error: {0}")]
    WriteError(nb::Error<W>),
    #[error("Message has bad checksum: {0}, expected {0}")]
    BadChecksum(u8, u8),
    #[error("Got {0} bytes but expected {1}")]
    BadLength(usize, usize),
    #[error("Invalid byte pattern: {0}")]
    PackingError(PackingError),
    #[error("Refcell already in use, hint: don't get messages from within callbacks")]
    ReferenceError(),
    #[error("Field parsing error for message")]
    InvalidData(),
    #[error("Messsage is of size {0}, allocated buffer is {1}")]
    BufferTooSmall(usize, usize),
    #[error("Slicing bug")]
    SliceError(TryFromSliceError),
    #[error("Capacity bug")]
    CapacityError(CapacityError),
    #[error("Pin Change error")]
    PinChangeBug(PinState), // TODO update this to use the type provided by the pin trait
}

impl<R, W> std::cmp::PartialEq for DriverError<R, W> {
    fn eq(&self, other: &Self) -> bool {
        use std::mem::discriminant;
        discriminant(self) == discriminant(other)
    }
}

// TODO remove this once https://github.com/rust-lang/rust/issues/35121 is done
pub struct StubPin {}

impl OutputPin for StubPin {
    type Error = u8;
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::{
        ChannelState, ChannelType, DeviceType, RxSyncByte, TransmissionChannelType,
        TransmissionGlobalDataPages, TransmissionType,
    };

    enum TestData {
        Data(Vec<u8>),
        Error(nb::Error<SerialError>),
    }

    #[derive(Debug, PartialEq, Error, Clone, Copy)]
    enum SerialError {
        #[error("fake error A")]
        A,
    }

    struct ValidationContext {
        in_bytes: Vec<TestData>,
        out_bytes: Vec<TestData>,
    }

    impl ValidationContext {
        fn validate(&self) {
            assert!(self.in_bytes.is_empty());
            assert!(self.out_bytes.is_empty());
        }
    }

    impl Read<u8> for ValidationContext {
        type Error = SerialError;

        fn read(&mut self) -> Result<u8, nb::Error<SerialError>> {
            let first = self.in_bytes.get_mut(0).unwrap();
            let data = match first {
                TestData::Data(d) => Ok(d.remove(0)),
                TestData::Error(e) => Err(*e),
            };
            match first {
                TestData::Data(d) => {
                    if d.is_empty() {
                        self.in_bytes.remove(0);
                    }
                }
                TestData::Error(_) => {
                    self.in_bytes.remove(0);
                }
            }
            return data;
        }
    }

    impl Write<u8> for ValidationContext {
        type Error = SerialError;

        fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
            let first = self.out_bytes.get_mut(0).unwrap();
            match first {
                TestData::Data(d) => {
                    assert_eq!(d.remove(0), word);
                    if d.is_empty() {
                        self.out_bytes.remove(0);
                    }
                }
                TestData::Error(e) => {
                    let val = Err(*e);
                    self.out_bytes.remove(0);
                    return val;
                }
            }
            Ok(())
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn message_packing() {
        let mut buf: [u8; 12] = [0; 12];
        let mut transmission_type = TransmissionType::default();
        transmission_type.device_number_extension = 2.into();
        transmission_type.transmission_channel_type =
            TransmissionChannelType::SharedChannel1ByteAddress;
        create_packed_message::<SerialError, SerialError>(
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

    // TODO write test where sync byte is in a bad typed message to cause buffer lock

    #[test]
    fn checksum() {
        let data = [0xA4, 6, 0x59, 2, 0x44, 0x33, 120, 34, 2];
        assert_eq!(calculate_checksum(&data), 214);
    }

    #[test]
    fn update_buffer_verify() {
        let context = ValidationContext {
            in_bytes: vec![],
            out_bytes: vec![],
        };
        let driver = SerialDriver::<_, StubPin>::new(context, None);
        let mut buf = driver.buffer.borrow_mut();
        [2, 3, 4, 5, 6].iter().for_each(|x| buf.push(*x));
        update_buffer::<SerialError, SerialError>(&Err(DriverError::BadChecksum(0, 0)), &mut buf);
        assert_eq!(buf.as_slice(), [3, 4, 5, 6]);
        buf.clear();
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            .iter()
            .for_each(|x| buf.push(*x));
        update_buffer::<SerialError, SerialError>(
            &Ok(Some(AntMessage {
                header: RxMessageHeader {
                    sync: RxSyncByte::Write,
                    msg_length: 6,
                    msg_id: RxMessageId::ChannelStatus,
                },
                message: RxMessageType::ChannelStatus(ChannelStatus {
                    channel_number: 1,
                    channel_type: ChannelType::SharedBidirectionalMaster,
                    network_number: 1,
                    channel_state: ChannelState::Searching,
                }),
                checksum: 0xFF,
            })),
            &mut buf,
        );
        assert_eq!(buf.as_slice(), []);
        [2, 3, 4, 5, 6, 7].iter().for_each(|x| buf.push(*x));
        update_buffer::<SerialError, SerialError>(&Ok(None), &mut buf);
        assert_eq!(buf.as_slice(), [2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn serial_read() {
        let context = ValidationContext {
            in_bytes: vec![
                TestData::Data(vec![0xA4, 5, 0x51, 1]),
                TestData::Data(vec![0x44, 0x33, 120, 34, 220]),
                TestData::Error(nb::Error::WouldBlock),
            ],
            out_bytes: vec![],
        };
        let mut driver = SerialDriver::<_, StubPin>::new(context, None);
        let mut transmission_type = TransmissionType::default();
        transmission_type.transmission_channel_type =
            TransmissionChannelType::SharedChannel1ByteAddress;
        transmission_type.global_datapages_used =
            TransmissionGlobalDataPages::GlobalDataPagesNotUsed;
        transmission_type.device_number_extension = 0x2.into();
        assert_eq!(
            driver.get_message(),
            Ok(Some(AntMessage {
                header: RxMessageHeader {
                    sync: RxSyncByte::Write,
                    msg_length: 5,
                    msg_id: RxMessageId::ChannelId,
                },
                message: RxMessageType::ChannelId(ChannelId {
                    channel_number: 1,
                    device_number: 0x3344,
                    device_type: DeviceType {
                        device_type_id: 120.into(),
                        pairing_request: false,
                    },
                    transmission_type: transmission_type,
                }),
                checksum: 220,
            }))
        );
        assert!(driver.buffer.borrow().is_empty());
        driver.serial.validate();
    }

    #[test]
    fn serial_two_messages_bulk() {
        let context = ValidationContext {
            in_bytes: vec![
                TestData::Data(vec![
                    0xA4, 5, 0x51, 1, 0x44, 0x33, 120, 34, 220, 0xA4, 5, 0x51, 1, 0x44, 0x33, 120,
                    34, 220,
                ]),
                TestData::Error(nb::Error::WouldBlock),
            ],
            out_bytes: vec![],
        };
        let mut driver = SerialDriver::<_, StubPin>::new(context, None);
        let mut transmission_type = TransmissionType::default();
        transmission_type.transmission_channel_type =
            TransmissionChannelType::SharedChannel1ByteAddress;
        transmission_type.global_datapages_used =
            TransmissionGlobalDataPages::GlobalDataPagesNotUsed;
        transmission_type.device_number_extension = 0x2.into();
        assert_eq!(
            driver.get_message(),
            Ok(Some(AntMessage {
                header: RxMessageHeader {
                    sync: RxSyncByte::Write,
                    msg_length: 5,
                    msg_id: RxMessageId::ChannelId,
                },
                message: RxMessageType::ChannelId(ChannelId {
                    channel_number: 1,
                    device_number: 0x3344,
                    device_type: DeviceType {
                        device_type_id: 120.into(),
                        pairing_request: false,
                    },
                    transmission_type: transmission_type,
                }),
                checksum: 220,
            }))
        );
        assert_eq!(
            driver.get_message(),
            Ok(Some(AntMessage {
                header: RxMessageHeader {
                    sync: RxSyncByte::Write,
                    msg_length: 5,
                    msg_id: RxMessageId::ChannelId,
                },
                message: RxMessageType::ChannelId(ChannelId {
                    channel_number: 1,
                    device_number: 0x3344,
                    device_type: DeviceType {
                        device_type_id: 120.into(),
                        pairing_request: false,
                    },
                    transmission_type: transmission_type,
                }),
                checksum: 220,
            }))
        );
        assert!(driver.buffer.borrow().is_empty());
        driver.serial.validate();
    }

    #[test]
    fn serial_write_out() {
        let context = ValidationContext {
            in_bytes: vec![],
            out_bytes: vec![TestData::Data(vec![
                0xA4, 6, 0x59, 2, 0x44, 0x33, 120, 34, 2, 214,
            ])],
        };
        let mut driver = SerialDriver::<_, StubPin>::new(context, None);
        let mut transmission_type = TransmissionType::default();
        transmission_type.transmission_channel_type =
            TransmissionChannelType::SharedChannel1ByteAddress;
        transmission_type.global_datapages_used =
            TransmissionGlobalDataPages::GlobalDataPagesNotUsed;
        transmission_type.device_number_extension = 0x2.into();
        assert!(driver
            .send_message(&AddChannelIdToList {
                channel_number: 2,
                device_number: 0x3344,
                device_type: DeviceType {
                    device_type_id: 120.into(),
                    pairing_request: false,
                },
                transmission_type: transmission_type,
                list_index: 2,
            })
            .is_ok());
        driver.serial.validate();
    }

    #[test]
    fn serial_error() {
        let context = ValidationContext {
            in_bytes: vec![],
            out_bytes: vec![TestData::Error(nb::Error::Other(SerialError::A))],
        };
        let mut driver = SerialDriver::<_, StubPin>::new(context, None);
        let mut transmission_type = TransmissionType::default();
        transmission_type.transmission_channel_type =
            TransmissionChannelType::SharedChannel1ByteAddress;
        transmission_type.global_datapages_used =
            TransmissionGlobalDataPages::GlobalDataPagesNotUsed;
        transmission_type.device_number_extension = 0x2.into();
        let err = driver.send_message(&AddChannelIdToList {
            channel_number: 2,
            device_number: 0x3344,
            device_type: DeviceType {
                device_type_id: 120.into(),
                pairing_request: false,
            },
            transmission_type: transmission_type,
            list_index: 2,
        });
        assert_eq!(
            err,
            Err(DriverError::WriteError(nb::Error::Other(SerialError::A)))
        );
        driver.serial.validate();
    }
}
