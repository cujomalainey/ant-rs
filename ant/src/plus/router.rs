// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::drivers::*;
use crate::messages::*;

use std::cell::{Cell, RefCell};

#[cfg(not(feature = "std"))]
use alloc::rc::{Rc, Weak};
#[cfg(feature = "std")]
use std::rc::{Rc, Weak};

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

/// Used with [set_key](Router::set_key) to identify what key is being set
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkKey {
    AntPlusKey,
    AntFsKey,
    Public,
    Custom(u8),
}

// This in theory is infinite, but its what the current hardware limit is.
/// Highest known supported channel count on a ANT device
pub const MAX_CHANNELS: usize = 15;
/// Highest known supported network count on a ANT device
pub const MAX_NETWORKS: usize = 8;

type ChannelHandler<R, W, D> = Rc<RefCell<dyn Channel<R, W, D>>>;

/// Channel is the trait all channels must implement to register with the router
pub trait Channel<R, W, D: Driver<R, W>> {
    /// All channels must be able to recieve messages and must be infalliable. If you have an
    /// error with a recieved message, deal with it internally, the router does not care.
    fn receive_message(&mut self, msg: &AntMessage);
    /// This is the callback when a channels is associated or dissociated with a router (depending
    /// on whether the Weak contains a ref). This is also called when the router sees the radio
    /// reboot and is configured in "Do Nothing" state. This will signal the channels to reset
    /// their internal state.
    fn set_router(
        &mut self,
        router: Weak<RefCell<Router<R, W, D>>>,
        channel: u8,
    ) -> Result<(), ChannelError>;
}

pub struct Router<R, W, D: Driver<R, W>> {
    channels: [Option<ChannelHandler<R, W, D>>; MAX_CHANNELS],
    max_channels: Cell<usize>, // what the hardware reports as some have less than max
    driver: RefCell<D>,
    rc_ref: Weak<RefCell<Self>>,
    network_key_indexes: [Option<NetworkKey>; MAX_NETWORKS],
    max_networks: Cell<usize>,
    reset_restore: Cell<bool>,
    rx_message_callback: Option<fn(&AntMessage)>,
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
    pub fn new(mut driver: D) -> Result<Rc<RefCell<Self>>, RouterError> {
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
        let router = Rc::new_cyclic(|me| {
            RefCell::new(Self {
                channels: [
                    None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None,
                ],
                network_key_indexes: [None; MAX_NETWORKS],
                max_channels: Cell::new(0),
                max_networks: Cell::new(0),
                reset_restore: Cell::new(false),
                driver: RefCell::new(driver),
                rc_ref: me.clone(),
                rx_message_callback: None,
            })
        });
        // If we don't get a response within 10ms give up
        let mut i = 0;
        while router.borrow().max_networks.get() == 0 && i < ROUTER_CAPABILITIES_RETRIES {
            router.borrow().process()?;
            i += 1;
        }
        if i == ROUTER_CAPABILITIES_RETRIES {
            return Err(RouterError::FailedToGetCapabilities());
        }
        Ok(router)
    }

    /// Set keys in a way the router can track. This allows profiles to lookup which network they
    /// should use
    pub fn set_key(
        &mut self,
        key: NetworkKey,
        network_key: &[u8; NETWORK_KEY_SIZE],
    ) -> Result<(), RouterError> {
        let index = self.network_key_indexes.iter().position(|x| x.is_none());
        if let Some(index) = index {
            return self.set_key_at_index(key, network_key, index);
        }
        Err(RouterError::OutOfNetworks())
    }

    pub fn set_key_at_index(
        &mut self,
        key: NetworkKey,
        network_key: &[u8; NETWORK_KEY_SIZE],
        index: usize,
    ) -> Result<(), RouterError> {
        let max_net = self.max_networks.get();
        if max_net == 0 {
            return Err(RouterError::DeviceCapabilitiesUnknown());
        }
        if index >= max_net {
            return Err(RouterError::OutOfNetworks());
        }
        if self.network_key_indexes[index].is_some() {
            return Err(RouterError::NetworkIndexInUse());
        }
        self.driver.borrow_mut().send_message(&SetNetworkKey {
            network_number: index as u8,
            network_key: *network_key,
        })?;
        self.network_key_indexes[index] = Some(key);
        Ok(())
    }

    /// Lookup key by [NetworkKey]
    pub fn get_key_index(&self, key: NetworkKey) -> Option<u8> {
        self.network_key_indexes
            .iter()
            .flatten()
            .position(|x| *x == key)
            .map(|x| x as u8)
    }

    /// Add a channel at next available index
    pub fn add_channel(&mut self, channel: ChannelHandler<R, W, D>) -> Result<(), RouterError> {
        let index = self.channels.iter().position(|x| x.is_none());
        let index = match index {
            Some(x) => x,
            None => return Err(RouterError::OutOfChannels()),
        };
        channel
            .borrow_mut()
            .set_router(self.rc_ref.clone(), index as u8)?;
        self.channels[index] = Some(channel);
        Ok(())
    }

    /// Add channel at a specific index
    pub fn add_channel_at_index(
        &mut self,
        channel: ChannelHandler<R, W, D>,
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
            .set_router(self.rc_ref.clone(), index as u8)?;
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
    pub fn send(&self, msg: &dyn AntTxMessageType) -> Result<(), RouterError> {
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
    pub fn remove_channel(&mut self, channel: &ChannelHandler<R, W, D>) -> Result<(), RouterError> {
        let index = self
            .channels
            .iter()
            .flatten()
            .position(|x| std::ptr::eq(x, channel));
        if let Some(x) = index {
            let chan = self.channels[x].take();
            if let Some(chan) = chan {
                chan.borrow_mut().set_router(Weak::new(), 0)?;
            }
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
        self.max_networks
            .set(msg.base_capabilities.max_networks as usize);
    }

    fn handle_message(&self, msg: &AntMessage) -> Result<(), RouterError> {
        if let Some(f) = self.rx_message_callback {
            f(msg);
        }
        match &msg.message {
            // These messages all have channel information, forward it accordingly
            RxMessageType::BroadcastData(data) => {
                self.route_message(data.payload.channel_number, msg)
            }
            RxMessageType::AcknowledgedData(data) => {
                self.route_message(data.payload.channel_number, msg)
            }
            RxMessageType::BurstTransferData(data) => {
                self.route_message(data.payload.channel_sequence.channel_number.into(), msg)
            }
            RxMessageType::AdvancedBurstData(data) => {
                self.route_message(data.channel_sequence.channel_number.into(), msg)
            }
            RxMessageType::ChannelEvent(data) => {
                self.route_message(data.payload.channel_number, msg)
            }
            RxMessageType::ChannelResponse(data) => self.route_message(data.channel_number, msg),
            RxMessageType::ChannelStatus(data) => self.route_message(data.channel_number, msg),
            RxMessageType::ChannelId(data) => self.route_message(data.channel_number, msg),
            // These messages can all provide actionable information to the profile but are not
            // channel specific
            RxMessageType::StartUpMessage(_) => {
                self.broadcast_message(msg);
                Ok(())
            }
            RxMessageType::Capabilities(data) => {
                self.broadcast_message(msg);
                self.parse_capabilities(data);
                Ok(())
            }
            RxMessageType::AdvancedBurstCapabilities(_) => {
                self.broadcast_message(msg);
                Ok(())
            }
            RxMessageType::AdvancedBurstConfiguration(_) => {
                self.broadcast_message(msg);
                Ok(())
            }
            RxMessageType::EncryptionModeParameters(_) => {
                self.broadcast_message(msg);
                Ok(())
            }
            // These message are not channel specific and operate at the router scope, should be
            // consumed directly at router callback
            RxMessageType::EventFilter(_) => Ok(()),
            RxMessageType::SerialErrorMessage(_) => Ok(()),
            RxMessageType::AntVersion(_) => Ok(()),
            RxMessageType::SerialNumber(_) => Ok(()),
            RxMessageType::EventBufferConfiguration(_) => Ok(()),
            RxMessageType::SelectiveDataUpdateMaskSetting(_) => Ok(()),
            RxMessageType::UserNvm(_) => Ok(()),
        }?;
        Ok(())
    }

    /// Parse all incoming messages and run callbacks
    pub fn process(&self) -> Result<(), RouterError> {
        loop {
            let msg = self.driver.borrow_mut().get_message()?;
            match msg {
                None => return Ok(()),
                Some(x) => self.handle_message(&x)?,
            }
        }
    }

    /// Teardown router and return driver
    pub fn release(self) -> D {
        self.driver.into_inner()
    }
}
