// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::drivers::*;
use crate::messages::config::UnAssignChannel;
use crate::messages::control::{CloseChannel, RequestMessage, RequestableMessageId, ResetSystem};
use crate::messages::requested_response::Capabilities;
use crate::messages::{AntMessage, RxMessage, TransmitableMessage};
use crate::plus::{Channel, ChannelAssignment};

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;

#[cfg(not(feature = "std"))]
use alloc::rc::Rc;
#[cfg(feature = "std")]
use std::rc::Rc;

#[derive(Debug)]
pub enum RouterError {
    ChannelError(ChannelError),
    OutOfChannels(),
    OutOfNetworks(),
    /// This means that we have not recieved the capabilities yet for the hardware. Usually this
    /// means you haven't called process yet or you have a communication problem with your device.
    DeviceCapabilitiesUnknown(),
    ChannelAlreadyAssigned(),
    DriverError(),
    ChannelOutOfBounds(),
    ChannelNotAssociated(),
    NetworkIndexInUse(),
    FailedToGetCapabilities(),
}

/// Channel Errors specific to router interfacing
#[derive(Debug)]
pub enum ChannelError {
    AlreadyAssociated(),
    IOErrorOnRestore(),
    NetworkKeyNotSet(),
}

// This in theory is infinite, but its what the current hardware limit is.
/// Highest known supported channel count on a ANT device
pub const MAX_CHANNELS: usize = 15;

type SharedChannel = Rc<RefCell<dyn Channel>>;

pub struct Router<R, W, D: Driver<R, W>> {
    channels: [Option<SharedChannel>; MAX_CHANNELS],
    max_channels: Cell<usize>, // what the hardware reports as some have less than max
    driver: RefCell<D>,
    reset_restore: Cell<bool>,
    rx_message_callback: Option<fn(&AntMessage)>,
    _read_marker: PhantomData<R>,
    _write_marker: PhantomData<W>,
}

impl<R, W> From<DriverError<R, W>> for RouterError {
    fn from(_err: DriverError<R, W>) -> Self {
        // TODO encapsilate error
        RouterError::DriverError()
    }
}

impl From<ChannelError> for RouterError {
    fn from(err: ChannelError) -> Self {
        RouterError::ChannelError(err)
    }
}

const ROUTER_CAPABILITIES_RETRIES: u8 = 25;

impl<R, W, D: Driver<R, W>> Router<R, W, D> {
    pub fn new(mut driver: D) -> Result<Self, RouterError> {
        // Reset system so we are coherent
        driver.send_message(&ResetSystem::new())?;
        // Purge driver state
        while driver.get_message().unwrap_or(None).is_some() {}
        // When we do first message fetch this should be the first message in the queue
        driver.send_message(&RequestMessage::new(
            0,
            RequestableMessageId::Capabilities,
            None,
        ))?;
        let router = Self {
            channels: [
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None,
            ],
            max_channels: Cell::new(0),
            reset_restore: Cell::new(false),
            driver: RefCell::new(driver),
            rx_message_callback: None,
            _read_marker: PhantomData,
            _write_marker: PhantomData,
        };
        // If we don't get a response within 25ms give up
        let mut i = 0;
        while router.max_channels.get() == 0 && i < ROUTER_CAPABILITIES_RETRIES {
            router.process()?;
            i += 1;
        }
        if i == ROUTER_CAPABILITIES_RETRIES {
            return Err(RouterError::FailedToGetCapabilities());
        }
        Ok(router)
    }

    /// Add a channel at next available index
    pub fn add_channel(&mut self, channel: SharedChannel) -> Result<(), RouterError> {
        let index = self.channels.iter().position(|x| x.is_none());
        let index = match index {
            Some(x) => x,
            None => return Err(RouterError::OutOfChannels()),
        };
        channel
            .borrow_mut()
            .set_channel(ChannelAssignment::Assigned(index as u8));
        self.channels[index] = Some(channel);
        Ok(())
    }

    /// Add channel at a specific index
    pub fn add_channel_at_index(
        &mut self,
        channel: SharedChannel,
        index: usize,
    ) -> Result<(), RouterError> {
        if index >= self.max_channels.get() {
            return Err(RouterError::ChannelOutOfBounds());
        }
        if self.channels[index].is_some() {
            return Err(RouterError::ChannelAlreadyAssigned());
        }
        channel
            .borrow_mut()
            .set_channel(ChannelAssignment::Assigned(index as u8));
        self.channels[index] = Some(channel);
        Ok(())
    }

    /// Reboot radio via reset message
    /// If `restore` is false: dissociate all channels and reset the hardware, router stays associated to
    /// the driver, if true restore system state.
    ///
    /// If you think the radio is not responding it is best to [Router::release] the driver and issue a
    /// reset via a hardware mechanism then rebuild.
    pub fn reset(&self, restore: bool) -> Result<(), DriverError<R, W>> {
        self.driver.borrow_mut().send_message(&ResetSystem::new())?;
        self.reset_restore.set(restore);
        if !restore {
            // TODO release profiles
        }
        Ok(())
    }

    /// Transmit a message to the radio
    pub fn send(&self, msg: &dyn TransmitableMessage) -> Result<(), RouterError> {
        self.driver.borrow_mut().send_message(msg)?;
        Ok(())
    }

    // TODO add a send and get response
    //
    // Logically since this is single threaded, if we send and recieve in the same call, all
    // messages that may come inbetween send and recieve have no consequence on the code flow. The
    // only challenge will be handling ownership since we will likely be holding the sender in a
    // mutable state and if they recieve another message it will be a problem

    /// Given a reference channel remove it from the router
    // TODO test
    pub fn remove_channel(&mut self, channel: &SharedChannel) -> Result<(), RouterError> {
        let index = self
            .channels
            .iter()
            .flatten()
            .position(|x| std::ptr::eq(x, channel));
        if let Some(x) = index {
            let chan = self.channels[x].take();
            if let Some(chan) = chan {
                chan.borrow_mut()
                    .set_channel(ChannelAssignment::UnAssigned());
            }
            // TODO maybe reset channel?
            let mut driver = self.driver.borrow_mut();
            driver.send_message(&CloseChannel::new(x as u8))?;
            driver.send_message(&UnAssignChannel::new(x as u8))?;
            return Ok(());
        }
        Err(RouterError::ChannelNotAssociated())
    }

    /// Register a callback to obersve all messages, this is meant for debugging or
    /// handling some radio specifics not handled by the router or a specific channel, e.g.
    /// capabilities messages
    pub fn set_rx_message_callback(&mut self, f: Option<fn(&AntMessage)>) {
        self.rx_message_callback = f;
    }

    fn route_message(&self, channel: u8, msg: &AntMessage) -> Result<(), RouterError> {
        if channel as usize >= MAX_CHANNELS {
            return Err(RouterError::ChannelOutOfBounds());
        }
        match &self.channels[channel as usize] {
            Some(handler) => handler.borrow_mut().receive_message(msg),
            None => return Err(RouterError::ChannelNotAssociated()),
        };
        Ok(())
    }

    fn broadcast_message(&self, msg: &AntMessage) {
        self.channels
            .iter()
            .flatten()
            .for_each(|x| x.borrow_mut().receive_message(msg));
    }

    fn parse_capabilities(&self, msg: &Capabilities) {
        self.max_channels
            .set(msg.base_capabilities.max_ant_channels as usize);
    }

    fn handle_message(&self, msg: &AntMessage) -> Result<(), RouterError> {
        if let Some(f) = self.rx_message_callback {
            f(msg);
        }
        match &msg.message {
            // These messages all have channel information, forward it accordingly
            RxMessage::BroadcastData(data) => self.route_message(data.payload.channel_number, msg),
            RxMessage::AcknowledgedData(data) => {
                self.route_message(data.payload.channel_number, msg)
            }
            RxMessage::BurstTransferData(data) => {
                self.route_message(data.payload.channel_sequence.channel_number.into(), msg)
            }
            RxMessage::AdvancedBurstData(data) => {
                self.route_message(data.channel_sequence.channel_number.into(), msg)
            }
            RxMessage::ChannelEvent(data) => self.route_message(data.payload.channel_number, msg),
            RxMessage::ChannelResponse(data) => self.route_message(data.channel_number, msg),
            RxMessage::ChannelStatus(data) => self.route_message(data.channel_number, msg),
            RxMessage::ChannelId(data) => self.route_message(data.channel_number, msg),
            // These messages can all provide actionable information to the profile but are not
            // channel specific
            RxMessage::StartUpMessage(_) => {
                self.broadcast_message(msg);
                Ok(())
            }
            RxMessage::Capabilities(data) => {
                self.broadcast_message(msg);
                self.parse_capabilities(data);
                Ok(())
            }
            RxMessage::AdvancedBurstCapabilities(_) => {
                self.broadcast_message(msg);
                Ok(())
            }
            RxMessage::AdvancedBurstCurrentConfiguration(_) => {
                self.broadcast_message(msg);
                Ok(())
            }
            RxMessage::EncryptionModeParameters(_) => {
                self.broadcast_message(msg);
                Ok(())
            }
            // These message are not channel specific and operate at the router scope, should be
            // consumed directly at router callback
            RxMessage::EventFilter(_) => Ok(()),
            RxMessage::SerialErrorMessage(_) => Ok(()),
            RxMessage::AntVersion(_) => Ok(()),
            RxMessage::SerialNumber(_) => Ok(()),
            RxMessage::EventBufferConfiguration(_) => Ok(()),
            RxMessage::SelectiveDataUpdateMaskSetting(_) => Ok(()),
            RxMessage::UserNvm(_) => Ok(()),
        }?;
        Ok(())
    }

    /// Parse all incoming messages and run callbacks
    pub fn process(&self) -> Result<(), RouterError> {
        while let Some(msg) = self.driver.borrow_mut().get_message()? {
            self.handle_message(&msg)?;
        }
        self.channels
            .iter()
            .flatten()
            .try_for_each(|x| self.send_channel(x))
    }

    pub fn send_channel(&self, channel: &SharedChannel) -> Result<(), RouterError> {
        let mut driver = self.driver.borrow_mut();
        while let Some(msg) = channel.borrow_mut().send_message() {
            driver.send_message(&msg)?;
        }
        Ok(())
    }

    /// Teardown router and return driver
    pub fn release(self) -> D {
        self.driver.into_inner()
    }
}
