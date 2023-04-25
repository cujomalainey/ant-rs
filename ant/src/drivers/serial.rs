// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::drivers::{
    create_packed_message, parse_buffer, Buffer, Driver, DriverError, align_buffer,ANT_MESSAGE_SIZE,
    update_buffer
};
use crate::messages::{AntMessage, TransmitableMessage};
use embedded_hal::digital::v2::{OutputPin, PinState};
use embedded_hal::serial::Read;
use embedded_hal::serial::Write;
use nb;

pub struct SerialDriver<SERIAL, PIN> {
    serial: SERIAL,
    sleep: Option<PIN>,
    buffer: Buffer, // TODO change this dependency injection so user controls the size
}

impl<SERIAL, SLEEP> SerialDriver<SERIAL, SLEEP>
where
    SERIAL: Read<u8> + Write<u8>,
    SLEEP: OutputPin,
{
    pub fn new(serial: SERIAL, sleep: Option<SLEEP>) -> SerialDriver<SERIAL, SLEEP> {
        SerialDriver {
            serial,
            sleep,
            buffer: Buffer::new(),
        }
    }

    pub fn release(self) -> (SERIAL, Option<SLEEP>) {
        (self.serial, self.sleep)
    }
}

impl<SERIAL, SLEEP, R, W> Driver<R, W> for SerialDriver<SERIAL, SLEEP>
where
    SERIAL: Read<u8, Error = R> + Write<u8, Error = W>,
    SLEEP: OutputPin,
{
    fn get_message(&mut self) -> Result<Option<AntMessage>, DriverError<R, W>> {
        let buf = &mut self.buffer;

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

        buf.drain(..align_buffer(buf));

        let msg_result = parse_buffer(buf);

        buf.drain(..update_buffer(&msg_result, buf));

        msg_result
    }

    fn send_message(&mut self, msg: &dyn TransmitableMessage) -> Result<(), DriverError<R, W>> {
        // TODO update with variable sized buf
        // TODO fix io error propotation
        let mut buf: [u8; ANT_MESSAGE_SIZE] = [0; ANT_MESSAGE_SIZE];

        let buf_slice = create_packed_message(&mut buf, msg)?;

        if let Some(pin) = &mut self.sleep {
            // TODO propogate error
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
            // TODO propogate error
            if pin.set_high().is_err() {
                return Err(DriverError::PinChangeBug(PinState::High));
            }
        }

        Ok(())
    }
}

// TODO remove this once https://github.com/rust-lang/rust/issues/35121 is done
/// This is a Pin type for devices that do not wish to use the pin functions of the drivers, this
/// includes USB use cases
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
    use crate::messages::config::{
        AddChannelIdToList, ChannelType, DeviceType, TransmissionChannelType,
        TransmissionGlobalDataPages, TransmissionType,
    };
    use crate::messages::requested_response::{ChannelId, ChannelState, ChannelStatus};
    use crate::messages::{RxMessage, RxMessageHeader, RxMessageId, RxSyncByte};

    enum TestData {
        Data(Vec<u8>),
        Error(nb::Error<SerialError>),
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    enum SerialError {
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
            let first = match self.in_bytes.get_mut(0) {
                Some(x) => x,
                None => return Err(nb::Error::WouldBlock),
            };
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

    // TODO write test where sync byte is in a bad typed message to cause buffer lock

    #[test]
    fn sleep_pin() {
        // TODO
    }

    #[test]
    fn update_buffer_verify() {
        let context = ValidationContext {
            in_bytes: vec![],
            out_bytes: vec![],
        };
        let driver = SerialDriver::<_, StubPin>::new(context, None);
        let mut buf = driver.buffer;
        [2, 3, 4, 5, 6].iter().for_each(|x| buf.push(*x));
        assert_eq!(1, update_buffer::<SerialError, SerialError>(&Err(DriverError::BadChecksum(0, 0)), &mut buf));
        buf.clear();
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            .iter()
            .for_each(|x| buf.push(*x));
        let remove = update_buffer::<SerialError, SerialError>(
            &Ok(Some(AntMessage {
                header: RxMessageHeader {
                    sync: RxSyncByte::Write,
                    msg_length: 6,
                    msg_id: RxMessageId::ChannelStatus,
                },
                message: RxMessage::ChannelStatus(ChannelStatus {
                    channel_number: 1,
                    channel_type: ChannelType::SharedBidirectionalMaster,
                    network_number: 1,
                    channel_state: ChannelState::Searching,
                }),
                checksum: 0xFF,
            })),
            &mut buf,
        );
        assert_eq!(remove, 10);
        buf.clear();
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
                message: RxMessage::ChannelId(ChannelId {
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
        assert!(driver.buffer.is_empty());
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
                message: RxMessage::ChannelId(ChannelId {
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
                message: RxMessage::ChannelId(ChannelId {
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
        assert!(driver.buffer.is_empty());
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
