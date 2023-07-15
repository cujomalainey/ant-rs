// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::channel::ChannelAssignment;
use crate::messages::channel::{ChannelEvent, ChannelResponse, MessageCode};
use crate::messages::config::{
    AssignChannel, ChannelId, ChannelPeriod, ChannelRfFrequency, ChannelType, DeviceType,
    SearchTimeout, TransmissionChannelType, TransmissionGlobalDataPages, TransmissionType,
    UnAssignChannel,
};
use crate::messages::control::{CloseChannel, OpenChannel, RequestMessage, RequestableMessageId};
use crate::messages::requested_response::{ChannelState, ChannelStatus};
use crate::messages::{AntMessage, RxMessage, TxMessage, TxMessageId};
use packed_struct::prelude::{packed_bits, Integer};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConfigureStateId {
    UnknownClose,
    UnknownUnAssign,
    Assign,
    Period,
    Id,
    Rf,
    Timeout,
    Error,
    Done,
    Identify,
}

trait ConfigureState {
    fn handle_response(&self, response: &ChannelResponse) -> &dyn ConfigureState;
    fn transmit_config(&self, channel: u8, handler: &MessageHandler) -> Option<TxMessage>;
    fn get_state(&self) -> ConfigureStateId;
}

struct Assign {}
impl ConfigureState for Assign {
    fn handle_response(&self, response: &ChannelResponse) -> &dyn ConfigureState {
        if response.message_id != TxMessageId::AssignChannel {
            return self;
        }
        if response.message_code == MessageCode::ResponseNoError {
            return &ID_STATE;
        }
        &ERROR_STATE
    }
    fn transmit_config(&self, channel: u8, handler: &MessageHandler) -> Option<TxMessage> {
        Some(
            AssignChannel::new(
                channel,
                handler.state_config.profile_reference.channel_type,
                handler.state_config.network_key_index,
                None,
            )
            .into(),
        )
    }
    fn get_state(&self) -> ConfigureStateId {
        ConfigureStateId::Assign
    }
}
const ASSIGN_STATE: Assign = Assign {};
struct Period {}
impl ConfigureState for Period {
    fn handle_response(&self, response: &ChannelResponse) -> &dyn ConfigureState {
        if response.message_id != TxMessageId::ChannelPeriod {
            return self;
        }
        if response.message_code == MessageCode::ResponseNoError {
            return &TIMEOUT_STATE;
        }
        &ERROR_STATE
    }
    fn transmit_config(&self, channel: u8, handler: &MessageHandler) -> Option<TxMessage> {
        Some(
            ChannelPeriod::new(
                channel,
                handler.state_config.profile_reference.channel_period,
            )
            .into(),
        )
    }
    fn get_state(&self) -> ConfigureStateId {
        ConfigureStateId::Period
    }
}
const PERIOD_STATE: Period = Period {};
struct Id {}
impl ConfigureState for Id {
    fn handle_response(&self, response: &ChannelResponse) -> &dyn ConfigureState {
        if response.message_id != TxMessageId::ChannelId {
            return self;
        }
        if response.message_code == MessageCode::ResponseNoError {
            return &RF_STATE;
        }
        &ERROR_STATE
    }
    fn transmit_config(&self, channel: u8, handler: &MessageHandler) -> Option<TxMessage> {
        let transmission_type = match handler.transmission_type {
            TransmissionTypeAssignment::Wildcard() => TransmissionType::new_wildcard(),
            TransmissionTypeAssignment::DeviceNumberExtension(x) => TransmissionType::new(
                handler
                    .state_config
                    .profile_reference
                    .transmission_channel_type,
                handler.state_config.profile_reference.global_datapages_used,
                x,
            ),
        };
        Some(
            ChannelId::new(
                channel,
                handler.device_number,
                DeviceType::new(
                    handler.state_config.profile_reference.device_type.into(),
                    handler.pairing_request,
                ),
                transmission_type,
            )
            .into(),
        )
    }
    fn get_state(&self) -> ConfigureStateId {
        ConfigureStateId::Id
    }
}
const ID_STATE: Id = Id {};
struct Rf {}
impl ConfigureState for Rf {
    fn handle_response(&self, response: &ChannelResponse) -> &dyn ConfigureState {
        if response.message_id != TxMessageId::ChannelRfFrequency {
            return self;
        }
        if response.message_code == MessageCode::ResponseNoError {
            return &PERIOD_STATE;
        }
        &ERROR_STATE
    }
    fn transmit_config(&self, channel: u8, handler: &MessageHandler) -> Option<TxMessage> {
        Some(
            ChannelRfFrequency::new(
                channel,
                handler.state_config.profile_reference.radio_frequency,
            )
            .into(),
        )
    }
    fn get_state(&self) -> ConfigureStateId {
        ConfigureStateId::Rf
    }
}
const RF_STATE: Rf = Rf {};
struct Timeout {}
impl ConfigureState for Timeout {
    fn handle_response(&self, response: &ChannelResponse) -> &dyn ConfigureState {
        if response.message_id != TxMessageId::SearchTimeout {
            return self;
        }
        if response.message_code == MessageCode::ResponseNoError {
            return &DONE_STATE;
        }
        &ERROR_STATE
    }
    fn transmit_config(&self, channel: u8, handler: &MessageHandler) -> Option<TxMessage> {
        Some(
            SearchTimeout::new(
                channel,
                handler.state_config.profile_reference.timeout_duration,
            )
            .into(),
        )
    }
    fn get_state(&self) -> ConfigureStateId {
        ConfigureStateId::Timeout
    }
}
const TIMEOUT_STATE: Timeout = Timeout {};
struct Error {}
impl ConfigureState for Error {
    fn handle_response(&self, _response: &ChannelResponse) -> &dyn ConfigureState {
        self
    }
    fn transmit_config(&self, _channel: u8, _handler: &MessageHandler) -> Option<TxMessage> {
        None
    }
    fn get_state(&self) -> ConfigureStateId {
        ConfigureStateId::Error
    }
}
const ERROR_STATE: Error = Error {};
struct Done {}
impl ConfigureState for Done {
    fn handle_response(&self, _response: &ChannelResponse) -> &dyn ConfigureState {
        self
    }
    fn transmit_config(&self, _channel: u8, _handler: &MessageHandler) -> Option<TxMessage> {
        None
    }
    fn get_state(&self) -> ConfigureStateId {
        ConfigureStateId::Done
    }
}
const DONE_STATE: Done = Done {};
struct UnknownClose {}
impl ConfigureState for UnknownClose {
    fn handle_response(&self, response: &ChannelResponse) -> &dyn ConfigureState {
        if response.message_id != TxMessageId::CloseChannel {
            return self;
        }
        &UNKNOWN_UNASSIGN_STATE
    }
    fn transmit_config(&self, channel: u8, _handler: &MessageHandler) -> Option<TxMessage> {
        Some(CloseChannel::new(channel).into())
    }
    fn get_state(&self) -> ConfigureStateId {
        ConfigureStateId::UnknownClose
    }
}
const UNKNOWN_CLOSE_STATE: UnknownClose = UnknownClose {};
struct UnknownUnAssign {}
impl ConfigureState for UnknownUnAssign {
    fn handle_response(&self, response: &ChannelResponse) -> &dyn ConfigureState {
        if response.message_id != TxMessageId::UnAssignChannel {
            return self;
        }
        &ASSIGN_STATE
    }
    fn transmit_config(&self, channel: u8, _handler: &MessageHandler) -> Option<TxMessage> {
        Some(UnAssignChannel::new(channel).into())
    }
    fn get_state(&self) -> ConfigureStateId {
        ConfigureStateId::UnknownUnAssign
    }
}
const UNKNOWN_UNASSIGN_STATE: UnknownUnAssign = UnknownUnAssign {};
struct Identify {}
impl ConfigureState for Identify {
    fn handle_response(&self, _response: &ChannelResponse) -> &dyn ConfigureState {
        self
    }
    fn transmit_config(&self, channel: u8, _handler: &MessageHandler) -> Option<TxMessage> {
        Some(RequestMessage::new(channel, RequestableMessageId::ChannelId, None).into())
    }
    fn get_state(&self) -> ConfigureStateId {
        ConfigureStateId::Identify
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ConfigureError {
    MessageTimeout(), // TODO add duration
    MessageError(MessageCode),
}

pub type StateError = (ConfigureStateId, ConfigureError);

enum ChannelStateCommand {
    Open,
    Close,
}
#[derive(Clone, Copy, Debug)]
pub enum TransmissionTypeAssignment {
    Wildcard(),
    DeviceNumberExtension(Integer<u8, packed_bits::Bits<4>>),
}

pub struct ProfileReference {
    pub device_type: u8,
    pub channel_type: ChannelType,
    pub transmission_channel_type: TransmissionChannelType, // ignoring device number extension
    pub global_datapages_used: TransmissionGlobalDataPages,
    pub radio_frequency: u8,
    pub timeout_duration: u8,
    pub channel_period: u16,
}

/// This struct constains everything constant from the point we passed in from the initialization,
/// nothing in it should change even if we reset
struct StateConfig {
    device_number: u16,
    transmission_type: TransmissionTypeAssignment,
    network_key_index: u8,
    profile_reference: &'static ProfileReference,
}

pub struct MessageHandler {
    channel: ChannelAssignment,
    // TODO check to see if this bit auto clears on the radio after a connect
    // TODO handle profiles that wildcard device type
    pairing_request: bool,
    configure_state: &'static dyn ConfigureState,
    configure_pending_response: bool,
    tx_ready: bool,
    device_number: u16,
    transmission_type: TransmissionTypeAssignment,
    set_channel_state: Option<ChannelStateCommand>,
    state_config: StateConfig,
}

impl MessageHandler {
    pub fn new(
        device_number: u16,
        transmission_type: TransmissionTypeAssignment,
        ant_plus_key_index: u8,
        profile_reference: &'static ProfileReference,
    ) -> Self {
        Self {
            channel: ChannelAssignment::UnAssigned(),
            configure_state: &UNKNOWN_CLOSE_STATE,
            set_channel_state: None,
            tx_ready: true,
            pairing_request: false,
            configure_pending_response: false,
            device_number,
            transmission_type,
            state_config: StateConfig {
                device_number,
                transmission_type,
                network_key_index: ant_plus_key_index,
                profile_reference,
            },
        }
    }

    pub fn get_channel(&self) -> ChannelAssignment {
        self.channel
    }

    /// Returns true if a TX_EVENT has been recieved since last call.
    pub fn is_tx_ready(&self) -> bool {
        self.tx_ready
    }

    pub fn tx_sent(&mut self) {
        self.tx_ready = false;
    }

    pub fn send_message(&mut self) -> Option<TxMessage> {
        // Skip if we are not assigned
        let channel = match self.channel {
            ChannelAssignment::UnAssigned() => return None,
            ChannelAssignment::Assigned(channel) => channel,
        };

        // Walk through configure state machine
        if !self.configure_pending_response {
            let msg = self.configure_state.transmit_config(channel, self);
            if msg.is_some() {
                self.configure_pending_response = true;
                return msg;
            }
        }

        // Block all data and runtime config until we complete config
        if self.configure_state.get_state() != ConfigureStateId::Done {
            return None;
        }

        // Handle channel open close command
        if let Some(command) = &self.set_channel_state {
            let msg = match command {
                ChannelStateCommand::Open => OpenChannel::new(channel).into(),
                ChannelStateCommand::Close => CloseChannel::new(channel).into(),
            };
            self.set_channel_state = None;
            return Some(msg);
        };
        // TODO check if we need to request channel info once bonded
        None
    }

    pub fn receive_message(&mut self, msg: &AntMessage) -> Result<(), StateError> {
        match &msg.message {
            RxMessage::ChannelResponse(msg) => self.handle_response(msg),
            RxMessage::ChannelEvent(msg) => self.handle_event(msg),
            RxMessage::ChannelId(msg) => self.handle_id(msg),
            RxMessage::ChannelStatus(msg) => self.handle_status(msg),
            _ => Ok(()),
        }
    }

    // TODO add logic to request this on setup
    // TODO This does not take into account user initited requests and could break state
    fn handle_status(&mut self, msg: &ChannelStatus) -> Result<(), StateError> {
        let state = self.configure_state.get_state();
        if state == ConfigureStateId::Error || state == ConfigureStateId::UnknownClose {
            // We don't care about state because we know we are broken or resetting
            return Ok(());
        }
        match msg.channel_state {
            ChannelState::UnAssigned => {
                if state == ConfigureStateId::Assign || state == ConfigureStateId::UnknownUnAssign {
                    return Ok(());
                }
                self.reset_state(true);
                Ok(())
            }
            ChannelState::Assigned | ChannelState::Searching | ChannelState::Tracking => {
                match state {
                    ConfigureStateId::Id
                    | ConfigureStateId::Period
                    | ConfigureStateId::Rf
                    | ConfigureStateId::Timeout
                    | ConfigureStateId::Done => return Ok(()),
                    _ => (),
                }
                self.reset_state(true);
                Ok(())
            }
        }
    }

    fn handle_response(&mut self, msg: &ChannelResponse) -> Result<(), StateError> {
        let new_state = self.configure_state.handle_response(msg);
        // TODO add timeout logic here
        if new_state.get_state() == ConfigureStateId::Error {
            let err = Err((
                self.configure_state.get_state(),
                ConfigureError::MessageError(msg.message_code),
            ));
            self.configure_state = new_state;
            return err;
        }
        if new_state.get_state() != self.configure_state.get_state() {
            self.configure_pending_response = false;
            self.configure_state = new_state;
        }
        Ok(())
    }

    fn handle_event(&mut self, msg: &ChannelEvent) -> Result<(), StateError> {
        // TODO check how collisions and TransfersFailed should be handled here
        match msg.payload.message_code {
            MessageCode::EventTx | MessageCode::EventTransferTxCompleted => self.tx_ready = true,
            _ => (),
        }
        Ok(())
    }

    // TODO Request this on connect
    fn handle_id(&mut self, msg: &ChannelId) -> Result<(), StateError> {
        if self.configure_state.get_state() == ConfigureStateId::Identify {
            // TODO handle state identification
            self.configure_state = &UNKNOWN_CLOSE_STATE;
            self.configure_pending_response = false;
            return Ok(());
        }
        self.device_number = msg.device_number;
        // TODO copy rest of state
        Ok(())
    }

    pub fn open(&mut self, pairing_request: bool) {
        self.set_channel_state = Some(ChannelStateCommand::Open);
        self.pairing_request = pairing_request;
    }

    pub fn close(&mut self) {
        self.set_channel_state = Some(ChannelStateCommand::Close);
    }

    pub fn set_channel(&mut self, channel: ChannelAssignment) {
        self.channel = channel;
        self.configure_state = &UNKNOWN_CLOSE_STATE;
    }

    fn reset_state(&mut self, reset_id_data: bool) {
        self.configure_state = &UNKNOWN_CLOSE_STATE;
        self.configure_pending_response = false;
        if reset_id_data {
            self.device_number = self.state_config.device_number;
            self.transmission_type = self.state_config.transmission_type;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::duration_to_search_timeout;
    use core::time::Duration;
    const TEST_REFERENCE: ProfileReference = ProfileReference {
        device_type: 5,
        transmission_channel_type: TransmissionChannelType::IndependentChannel,
        global_datapages_used: TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
        channel_type: ChannelType::BidirectionalSlave,
        radio_frequency: 25,
        timeout_duration: duration_to_search_timeout(Duration::from_secs(30)),
        channel_period: 123,
    };

    #[test]
    fn inert_start() {}
}
