// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::drivers::{Driver, DriverError};
use crate::messages::config::UnAssignChannel;
use crate::messages::control::{CloseChannel, RequestMessage, RequestableMessageId, ResetSystem};
use crate::messages::requested_response::Capabilities;
use crate::messages::{AntMessage, RxMessage, TransmitableMessage, TxMessage};
use crate::channel::{TxHandler, RxHandler, TxError, RxError};

use std::cell::Cell;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum RouterError {
    OutOfChannels(),
    ChannelAlreadyAssigned(),
    DriverError(),
    ChannelOutOfBounds(),
    ChannelNotAssociated(),
    FailedToGetCapabilities(),
}

// This in theory is infinite, but its what the current hardware limit is.
/// Highest known supported channel count on a ANT device
pub const MAX_CHANNELS: usize = 15;

pub struct Router<E, D: Driver<E>, T: TxHandler<AntMessage>, R: RxHandler<TxMessage>> {
    channels: [Option<T>; MAX_CHANNELS],
    max_channels: Cell<usize>, // what the hardware reports as some have less than max
    driver: D,
    reset_restore: Cell<bool>,
    rx_message_callback: Option<fn(&AntMessage)>,
    receiver: R,
    _marker: PhantomData<E>,
}

impl<E> From<DriverError<E>> for RouterError {
    fn from(_err: DriverError<E>) -> Self {
        // TODO encapsilate error
        RouterError::DriverError()
    }
}

const ROUTER_CAPABILITIES_RETRIES: u8 = 25;

impl<E, D: Driver<E>, T: TxHandler<AntMessage>, R: RxHandler<TxMessage>> Router<E, D, T, R> {
    // TODO change to generic receiver
    pub fn new(mut driver: D, receiver: R) -> Result<Self, RouterError> {
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
        let mut router = Self {
            channels: std::array::from_fn(|_| None),
            max_channels: Cell::new(0),
            reset_restore: Cell::new(false),
            driver,
            rx_message_callback: None,
            receiver,
            _marker: PhantomData,
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
    pub fn add_channel(&mut self, channel: T) -> Result<u8, RouterError> {
        let index = self.channels.iter().position(|x| x.is_none());
        let index = match index {
            Some(x) => x,
            None => return Err(RouterError::OutOfChannels()),
        };
        self.channels[index] = Some(channel);
        Ok(index as u8)
    }

    /// Add channel at a specific index
    pub fn add_channel_at_index(
        &mut self,
        channel: T,
        index: usize,
    ) -> Result<(), RouterError> {
        if index >= self.max_channels.get() {
            return Err(RouterError::ChannelOutOfBounds());
        }
        if self.channels[index].is_some() {
            return Err(RouterError::ChannelAlreadyAssigned());
        }
        self.channels[index] = Some(channel);
        Ok(())
    }

    /// Reboot radio via reset message
    /// If `restore` is false: dissociate all channels and reset the hardware, router stays associated to
    /// the driver, if true restore system state.
    ///
    /// If you think the radio is not responding it is best to [Router::release] the driver and issue a
    /// reset via a hardware mechanism then rebuild.
    pub fn reset(&mut self, restore: bool) -> Result<(), DriverError<E>> {
        self.driver.send_message(&ResetSystem::new())?;
        self.reset_restore.set(restore);
        if !restore {
            // TODO release profiles
        }
        Ok(())
    }

    /// Transmit a message to the radio
    pub fn send(&mut self, msg: &dyn TransmitableMessage) -> Result<(), RouterError> {
        self.driver.send_message(msg)?;
        Ok(())
    }

    /// Given a reference channel remove it from the router
    // TODO test
    pub fn remove_channel(&mut self, channel: u8) -> Result<(), RouterError> {
        // TODO make indux lookup safe
        let chan = self.channels[channel as usize].take();
        if chan.is_none() {
            return Err(RouterError::ChannelNotAssociated());
        }
        self.driver.send_message(&CloseChannel::new(channel))?;
        self.driver.send_message(&UnAssignChannel::new(channel))?;
        Ok(())
    }

    /// Register a callback to obersve all messages, this is meant for debugging or
    /// handling some radio specifics not handled by the router or a specific channel, e.g.
    /// capabilities messages
    pub fn set_rx_message_callback(&mut self, f: Option<fn(&AntMessage)>) {
        self.rx_message_callback = f;
    }

    fn route_message(&self, channel: u8, msg: AntMessage) -> Result<(), RouterError> {
        if channel as usize >= MAX_CHANNELS {
            return Err(RouterError::ChannelOutOfBounds());
        }
        match &self.channels[channel as usize] {
            Some(handler) => handler.try_send(msg).expect("TODO"),
            None => return Err(RouterError::ChannelNotAssociated()),
        };
        Ok(())
    }

    fn broadcast_message(&self, msg: AntMessage) {
        self.channels
            .iter()
            .flatten()
            .for_each(|x| x.try_send(msg.clone()).expect("TODO"));
    }

    fn parse_capabilities(&self, msg: &Capabilities) {
        self.max_channels
            .set(msg.base_capabilities.max_ant_channels as usize);
    }

    fn handle_message(&self, msg: AntMessage) -> Result<(), RouterError> {
        if let Some(f) = self.rx_message_callback {
            f(&msg);
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
                self.broadcast_message(msg.clone());
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
    pub fn process(&mut self) -> Result<(), RouterError> {
        while let Some(msg) = self.driver.get_message()? {
            self.handle_message(msg)?;
        }
        while let Ok(msg) = self.receiver.try_recv() {
            self.driver.send_message(&msg)?;
        }
        Ok(())
    }

    /// Teardown router and return driver
    pub fn release(self) -> D {
        self.driver
    }
}
