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

fn transmission_type_from_state(handler: &MessageHandler) -> TransmissionType {
    match handler.transmission_type {
        TransmissionTypeAssignment::Wildcard() => TransmissionType::new_wildcard(),
        TransmissionTypeAssignment::DeviceNumberExtension(x) => TransmissionType::new(
            handler
                .state_config
                .profile_reference
                .transmission_channel_type,
            handler.state_config.profile_reference.global_datapages_used,
            x,
        ),
    }
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
        Some(
            ChannelId::new(
                channel,
                handler.device_number,
                DeviceType::new(
                    handler.state_config.profile_reference.device_type.into(),
                    matches!(
                        handler.pairing_request,
                        DevicePairingState::PendingSet | DevicePairingState::BitSet
                    ),
                ),
                transmission_type_from_state(handler),
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
            return &IDENTIFY_STATE;
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
    fn transmit_config(&self, _channel: u8, _handler: &MessageHandler) -> Option<TxMessage> {
        None
    }
    fn get_state(&self) -> ConfigureStateId {
        ConfigureStateId::Identify
    }
}
const IDENTIFY_STATE: Identify = Identify {};

#[derive(Clone, Copy, Debug)]
pub enum ConfigureError {
    MessageTimeout(), // TODO add duration
    MessageError(MessageCode),
    ChannelInWrongState {
        current: ChannelState,
        expected: ChannelState,
    },
}

pub type StateError = (ConfigureStateId, ConfigureError);

#[derive(PartialEq)]
enum DevicePairingState {
    PendingSet,
    BitSet,
    PendingClear,
    BitCleared,
}

enum ChannelStateCommand {
    Open,
    Close,
}
#[derive(Clone, Copy, Debug)]
pub enum TransmissionTypeAssignment {
    Wildcard(),
    DeviceNumberExtension(Integer<u8, packed_bits::Bits<4>>),
}

impl From<TransmissionType> for TransmissionTypeAssignment {
    fn from(transmission_type: TransmissionType) -> TransmissionTypeAssignment {
        TransmissionTypeAssignment::DeviceNumberExtension(transmission_type.device_number_extension)
    }
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
struct StateConfig<'a> {
    device_number: u16,
    transmission_type: TransmissionTypeAssignment,
    network_key_index: u8,
    profile_reference: &'a ProfileReference,
}

pub struct MessageHandler<'a> {
    channel: ChannelAssignment,
    // TODO handle profiles that wildcard device type
    /// Are we setting the pairing bit?
    pairing_request: DevicePairingState,
    /// Configuration state machine pointer
    configure_state: &'a dyn ConfigureState,
    /// State machine confgi message pending response
    configure_pending_response: bool,
    /// Previous TX transmission sent, ready for new message
    tx_ready: bool,
    /// Device number
    /// For master's this is their ID
    /// For slaves this is the masters' ID
    device_number: u16,
    /// Transmisison type of the channel
    transmission_type: TransmissionTypeAssignment,
    /// Pending command to open/close the channel
    set_channel_state: Option<ChannelStateCommand>,
    /// Original passed in arguements. This is used to differentiate in slaves from wildcarded
    /// fields versus discovered data
    state_config: StateConfig<'a>,
    /// Last state of the channel we were aware of
    channel_state: ChannelState,
    /// Transmit a request for channel id on next TX window
    tx_channel_id_request: bool,
}

impl<'a> MessageHandler<'a> {
    pub fn new(
        device_number: u16,
        transmission_type: TransmissionTypeAssignment,
        ant_plus_key_index: u8,
        profile_reference: &'a ProfileReference,
    ) -> Self {
        Self {
            channel: ChannelAssignment::UnAssigned(),
            configure_state: &UNKNOWN_CLOSE_STATE,
            set_channel_state: None,
            tx_ready: true,
            pairing_request: DevicePairingState::BitCleared,
            configure_pending_response: false,
            channel_state: ChannelState::UnAssigned,
            device_number,
            transmission_type,
            state_config: StateConfig {
                device_number,
                transmission_type,
                network_key_index: ant_plus_key_index,
                profile_reference,
            },
            tx_channel_id_request: false,
        }
    }

    pub fn get_channel(&self) -> ChannelAssignment {
        self.channel
    }

    /// Returns the current device_number in use
    ///  TOOD slave/master
    /// If a wildcard was set and device has not connected yet a wildcard will be returned.
    /// Recommended to be called after [get_channel_state] returns tracking at least once or you
    /// have observed a datapage recieved
    pub fn get_device_id(&self) -> u16 {
        self.device_number
    }

    pub fn is_tracking(&self) -> bool {
        self.channel_state == ChannelState::Tracking
    }

    /// Returns true if a TX_EVENT has been recieved since last call.
    pub fn is_tx_ready(&self) -> bool {
        self.tx_ready
    }

    /// Signal that a datapage has been sent and we need to track the next TX_EVENT
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
        let state = self.configure_state.get_state();
        if state != ConfigureStateId::Done && state != ConfigureStateId::Identify {
            return None;
        }

        if matches!(
            self.pairing_request,
            DevicePairingState::PendingSet | DevicePairingState::PendingClear
        ) {
            let bit_state = self.pairing_request == DevicePairingState::PendingSet;
            match self.pairing_request {
                DevicePairingState::PendingSet => self.pairing_request = DevicePairingState::BitSet,
                DevicePairingState::PendingClear => {
                    self.pairing_request = DevicePairingState::BitCleared
                }
                _ => (),
            }
            return Some(
                ChannelId::new(
                    channel,
                    self.state_config.device_number,
                    DeviceType::new(
                        self.state_config.profile_reference.device_type.into(),
                        bit_state,
                    ),
                    transmission_type_from_state(self),
                )
                .into(),
            );
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

        if self.tx_channel_id_request {
            self.tx_channel_id_request = false;
            return Some(RequestMessage::new(channel, RequestableMessageId::ChannelId, None).into());
        }

        None
    }

    pub fn receive_message(&mut self, msg: &AntMessage) -> Result<(), StateError> {
        match &msg.message {
            RxMessage::ChannelResponse(msg) => self.handle_response(msg),
            RxMessage::ChannelEvent(msg) => self.handle_event(msg),
            RxMessage::ChannelId(msg) => self.handle_id(msg),
            RxMessage::ChannelStatus(msg) => self.handle_status(msg),
            RxMessage::BroadcastData(_)
                | RxMessage::AcknowledgedData(_)
                | RxMessage::BurstTransferData(_)
                | RxMessage::AdvancedBurstData(_) => {
                    if self.configure_state.get_state() == ConfigureStateId::Identify {
                        self.tx_channel_id_request = true;
                    }
                    Ok(())
            },
            _ => Ok(()),
        }
    }

    fn handle_status(&mut self, msg: &ChannelStatus) -> Result<(), StateError> {
        self.channel_state = msg.channel_state;
        Ok(())
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

    fn handle_id(&mut self, msg: &ChannelId) -> Result<(), StateError> {
        if self.configure_state.get_state() == ConfigureStateId::Identify {
            self.configure_state = &DONE_STATE;
            self.configure_pending_response = false;
        }
        self.device_number = msg.device_number;
        self.transmission_type = msg.transmission_type.into();
        Ok(())
    }

    /// Set pairing bit
    /// For slaves this must be done while the channel is closed but will be auto cleared on bond
    ///
    /// For masters this can be done while the channel is open or closed but must be manually
    /// cleared
    pub fn set_pairing_bit(&mut self, state: bool) -> Result<(), ConfigureError> {
        if matches!(
            self.state_config.profile_reference.channel_type,
            ChannelType::BidirectionalSlave
                | ChannelType::SharedBidirectionalSlave
                | ChannelType::SharedReceiveOnly
        ) && matches!(
            self.channel_state,
            ChannelState::Searching | ChannelState::Tracking
        ) {
            return Err(ConfigureError::ChannelInWrongState {
                current: self.channel_state,
                expected: ChannelState::Assigned,
            });
        }
        self.pairing_request = match state {
            false => DevicePairingState::PendingClear,
            true => DevicePairingState::PendingSet,
        };
        Ok(())
    }

    pub fn open(&mut self) {
        self.set_channel_state = Some(ChannelStateCommand::Open);
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
    fn inert_start() {
        // TODO
    }

    #[test]
    fn config() {
        // TODO
    }

    #[test]
    fn pairing_bit_slave() {
        // TODO
    }

    #[test]
    fn pairing_bit_master() {
        // TODO
    }

    #[test]
    fn state_transition_on_failure() {
        // TODO
    }

    #[test]
    fn signal_loss() {
        // TODO
    }

    #[test]
    fn open_channel() {
        // TODO
    }

    #[test]
    fn close_channel() {
        // TODO
    }

    #[test]
    fn check_config_close_open() {
        // TODO
    }

    #[test]
    fn check_config_reset_open() {
        // TODO
    }

    #[test]
    fn tx_signaling() {
        // TODO
    }
}
