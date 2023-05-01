// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// MacOS only USB to Serial interface for ANT USB sticks
// Linux does not need this as the sticks show up as proper serial devices

use crate::drivers::{
    align_buffer, create_packed_message, parse_buffer, update_buffer, Driver, DriverError,
    ANT_MESSAGE_SIZE,
};
use crate::messages::{AntMessage, TransmitableMessage};
use rusb::{Device, DeviceHandle, Direction, Interface, TransferType, UsbContext};
use std::cmp::min;
use std::time::Duration;

pub type UsbDriverError = DriverError<rusb::Error, rusb::Error>;

pub struct UsbDriver<T: UsbContext> {
    handle: DeviceHandle<T>,
    in_address: u8,
    out_address: u8,
    iface: u8,
    in_buf: Vec<u8>,
    out_buf: Vec<u8>,
    in_max_packet_size: usize,
    out_max_packet_size: usize,
}

impl<T: UsbContext> Driver<rusb::Error, rusb::Error> for UsbDriver<T> {
    fn get_message(&mut self) -> Result<Option<AntMessage>, UsbDriverError> {
        if let Err(x) = self.read() {
            return Err(DriverError::ReadError(x));
        }
        let buf = &mut self.in_buf;

        buf.drain(..align_buffer(buf));
        let msg = parse_buffer(buf);
        buf.drain(..update_buffer(&msg, buf));
        if Ok(None) != msg {
            return msg;
        }
        Ok(None)
    }

    fn send_message(&mut self, msg: &dyn TransmitableMessage) -> Result<(), UsbDriverError> {
        let mut buf: [u8; ANT_MESSAGE_SIZE] = [0; ANT_MESSAGE_SIZE];
        let buf_slice = create_packed_message(&mut buf, msg)?;

        self.out_buf.extend_from_slice(buf_slice);

        if let Err(x) = self.flush() {
            return Err(DriverError::WriteError(x));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum UsbError {
    CannotFindEndpoint(Direction),
    VidNotRecognized(u16),
    PidNotRecognized(u16),
    FailedToOpenDevice(rusb::Error),
    MissingConfig(rusb::Error),
    FailedToSetConfig(rusb::Error),
    UnableToDetachDriver(rusb::Error),
    FailedToReset(rusb::Error),
    CantClaimIface(rusb::Error),
    NoInterfaces(),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbDevice {
    vendor_id: u16,
    product_id: u16,
}

pub const USB_M_STICK: UsbDevice = UsbDevice {
    vendor_id: 0x0fcf,
    product_id: 0x1009,
};

pub const USB_2_STICK: UsbDevice = UsbDevice {
    vendor_id: 0x0fcf,
    product_id: 0x1008,
};

pub fn is_ant_usb_device(vendor_id: u16, product_id: u16) -> bool {
    matches!(
        (UsbDevice {
            vendor_id,
            product_id
        }),
        USB_M_STICK | USB_2_STICK
    )
}

fn find_endpoint(
    interface: &Interface,
    transfer_type: TransferType,
    endpoint_direction: Direction,
) -> Result<(u8, usize), UsbError> {
    for interface_desc in interface.descriptors() {
        for endpoint_desc in interface_desc.endpoint_descriptors() {
            if endpoint_desc.direction() == endpoint_direction
                && endpoint_desc.transfer_type() == transfer_type
            {
                return Ok((
                    endpoint_desc.address(),
                    endpoint_desc.max_packet_size() as usize,
                ));
            }
        }
    }
    Err(UsbError::CannotFindEndpoint(endpoint_direction))
}

impl<T: UsbContext> UsbDriver<T> {
    pub fn new(device: Device<T>) -> Result<Self, UsbError> {
        let mut handle = match device.open() {
            Ok(h) => h,
            Err(e) => return Err(UsbError::FailedToOpenDevice(e)),
        };

        let config = match device.config_descriptor(0) {
            Ok(c) => c,
            Err(e) => return Err(UsbError::MissingConfig(e)),
        };

        let iface = if let Some(iface) = config.interfaces().next() {
            iface
        } else {
            return Err(UsbError::NoInterfaces());
        };

        let driver_active = matches!(handle.kernel_driver_active(iface.number()), Ok(true));

        let (out_address, out_max_packet_size) =
            find_endpoint(&iface, TransferType::Bulk, Direction::Out)?;

        let (in_address, in_max_packet_size) =
            find_endpoint(&iface, TransferType::Bulk, Direction::In)?;

        if driver_active {
            if let Err(e) = handle.detach_kernel_driver(iface.number()) {
                return Err(UsbError::UnableToDetachDriver(e));
            };
        }

        if let Err(reset) = handle.reset() {
            return Err(UsbError::FailedToReset(reset));
        }

        if let Err(claim) = handle.claim_interface(iface.number()) {
            return Err(UsbError::CantClaimIface(claim));
        }

        // if let Err(e) = handle.set_active_configuration(config.number()) {
        //     return Err(UsbError::FailedToSetConfig(e));
        // };

        Ok(Self {
            handle,
            iface: iface.number(),
            in_address,
            out_address,
            in_buf: Vec::new(),
            out_buf: Vec::new(),
            in_max_packet_size,
            out_max_packet_size,
        })
    }

    pub fn release(mut self) -> Result<Device<T>, rusb::Error> {
        // reatach all drivers and undo usb walk
        // TODO cast into local error type
        self.handle.release_interface(self.iface)?;
        self.handle.unconfigure()?;
        self.handle.attach_kernel_driver(self.iface)?;
        Ok(self.handle.device())
    }

    fn read(&mut self) -> nb::Result<(), rusb::Error> {
        let mut buf = vec![0; self.in_max_packet_size];
        const TIMEOUT: Duration = Duration::from_millis(1);

        loop {
            match self.handle.read_bulk(self.in_address, &mut buf, TIMEOUT) {
                Ok(len) => self.in_buf.extend_from_slice(&buf[..len]),
                Err(rusb::Error::Timeout) => return Ok(()),
                Err(err) => return Err(nb::Error::Other(err)),
            }
        }
    }

    fn write(&mut self) -> nb::Result<(), rusb::Error> {
        let buf = &self.out_buf[..min(self.out_buf.len(), self.out_max_packet_size)];
        // Shortest timeout possible
        let timeout = Duration::from_millis(1);

        let len = match self.handle.write_bulk(self.out_address, buf, timeout) {
            Ok(n) => n,
            Err(rusb::Error::Timeout) => return Err(nb::Error::WouldBlock),
            Err(io) => return Err(nb::Error::Other(io)),
        };
        self.out_buf.drain(..len);
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), rusb::Error> {
        while !self.out_buf.is_empty() {
            self.write()?;
        }
        Ok(())
    }
}

pub fn is_ant_usb_device_from_device<T: UsbContext>(device: &Device<T>) -> bool {
    match device.device_descriptor() {
        Ok(d) => is_ant_usb_device(d.vendor_id(), d.product_id()),
        Err(_) => false,
    }
}
