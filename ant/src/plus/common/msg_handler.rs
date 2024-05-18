// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::messages::channel::{ChannelEvent, ChannelResponse, MessageCode};
use crate::messages::config::{
    AssignChannel, ChannelId, ChannelPeriod, ChannelRfFrequency, ChannelType, DeviceType,
    SearchTimeout, TransmissionType, UnAssignChannel,
};
use crate::messages::control::{CloseChannel, OpenChannel, RequestMessage, RequestableMessageId};
use crate::messages::requested_response::{ChannelState, ChannelStatus};
use crate::messages::{AntMessage, RxMessage, TxMessage, TxMessageId};

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
                handler.state_config.channel_config.channel_type,
                handler.state_config.channel_config.network_key_index,
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
        Some(ChannelPeriod::new(channel, handler.state_config.channel_config.channel_period).into())
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
                handler.state_config.device_number,
                DeviceType::new(
                    handler.state_config.channel_config.device_type.into(),
                    matches!(
                        handler.pairing_request,
                        DevicePairingState::PendingSet | DevicePairingState::BitSet
                    ),
                ),
                handler.state_config.transmission_type,
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
            ChannelRfFrequency::new(channel, handler.state_config.channel_config.radio_frequency)
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
                handler.state_config.channel_config.timeout_duration,
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

// TODO doc
#[derive(Copy, Clone, Debug)]
pub struct ChannelConfig {
    pub device_number: u16,
    pub device_type: u8,
    pub channel_type: ChannelType,
    pub transmission_type: TransmissionType,
    pub radio_frequency: u8,
    pub timeout_duration: u8,
    pub channel_period: u16,
    pub network_key_index: u8,
}

/// This struct constains everything constant from the point we passed in from the initialization,
/// nothing in it should change even if we reset
struct StateConfig {
    /// Device number
    /// For master's this is their ID
    /// For slaves this is the masters' ID
    device_number: u16,
    device_type: DeviceType,
    /// Actual transmisison type of the channel (extension only)
    transmission_type: TransmissionType,
    /// Static passed in (assigned) config, may contain wildcards and is not reflective of what
    /// system may have bonded to
    channel_config: ChannelConfig,
}

pub struct MessageHandler {
    channel: u8,
    /// Are we setting the pairing bit?
    pairing_request: DevicePairingState,
    /// Configuration state machine pointer
    configure_state: &'static dyn ConfigureState,
    /// State machine confgi message pending response
    configure_pending_response: bool,
    /// Previous TX transmission sent, ready for new message
    tx_ready: bool,
    /// Pending command to open/close the channel
    set_channel_state: Option<ChannelStateCommand>,
    /// Original passed in arguements. This is used to differentiate in slaves from wildcarded
    /// fields versus discovered data
    state_config: StateConfig,
    /// Last state of the channel we were aware of
    channel_state: ChannelState,
    /// Transmit a request for channel id on next TX window
    tx_channel_id_request: bool,
}

impl MessageHandler {
    pub fn new(channel: u8, channel_config: &ChannelConfig) -> Self {
        Self {
            channel,
            configure_state: &UNKNOWN_CLOSE_STATE,
            set_channel_state: None,
            tx_ready: true,
            pairing_request: DevicePairingState::BitCleared,
            configure_pending_response: false,
            channel_state: ChannelState::UnAssigned,
            state_config: StateConfig {
                device_number: channel_config.device_number,
                device_type: DeviceType::new(channel_config.device_type.into(), false),
                transmission_type: channel_config.transmission_type,
                channel_config: *channel_config,
            },
            tx_channel_id_request: false,
        }
        // TODO decide if we want to do check on the radio behalf for invalid config (e.g. wildcard
        // master)
    }

    pub fn get_channel(&self) -> u8 {
        self.channel
    }

    /// Returns the current device_number in use
    ///
    /// Slave channels: If a wildcard was set and device has not connected yet a wildcard will be returned.
    /// Recommended to be called after [MessageHandler::is_tracking] returns tracking at least once or you
    /// have observed a datapage recieved
    ///
    /// Master channels: returns the ID being broadcasted
    pub fn get_device_id(&self) -> u16 {
        self.state_config.device_number
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
        // Walk through configure state machine
        if !self.configure_pending_response {
            let msg = self.configure_state.transmit_config(self.channel, self);
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
                    self.channel,
                    self.state_config.device_number,
                    DeviceType::new(
                        self.state_config.channel_config.device_type.into(),
                        bit_state,
                    ),
                    self.state_config.transmission_type,
                )
                .into(),
            );
        }

        // Handle channel open close command
        if let Some(command) = &self.set_channel_state {
            let msg = match command {
                ChannelStateCommand::Open => OpenChannel::new(self.channel).into(),
                ChannelStateCommand::Close => CloseChannel::new(self.channel).into(),
            };
            self.set_channel_state = None;
            return Some(msg);
        };

        if self.tx_channel_id_request {
            self.tx_channel_id_request = false;
            return Some(
                RequestMessage::new(self.channel, RequestableMessageId::ChannelId, None).into(),
            );
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
            }
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
        self.state_config.device_number = msg.device_number;
        self.state_config.device_type = msg.device_type;
        self.state_config.transmission_type = msg.transmission_type;
        Ok(())
    }

    /// Set pairing bit
    /// For slaves this must be done while the channel is closed but will be auto cleared on bond
    ///
    /// For masters this can be done while the channel is open or closed but must be manually
    /// cleared
    pub fn set_pairing_bit(&mut self, state: bool) -> Result<(), ConfigureError> {
        if matches!(
            self.state_config.channel_config.channel_type,
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

    /// Resets assumed channel state. Maintains bonding information if `reset_id_data` is `false`.
    pub fn reset_state(&mut self, reset_id_data: bool) {
        self.configure_state = &UNKNOWN_CLOSE_STATE;
        self.configure_pending_response = false;
        self.tx_ready = true;
        self.channel_state = ChannelState::UnAssigned;
        if reset_id_data {
            self.state_config.device_number = self.state_config.channel_config.device_number;
            self.state_config.transmission_type =
                self.state_config.channel_config.transmission_type;
            self.state_config.device_type =
                DeviceType::new(self.state_config.channel_config.device_type.into(), false);
            self.pairing_request = DevicePairingState::BitCleared;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::duration_to_search_timeout;
    use crate::messages::config::{TransmissionChannelType, TransmissionGlobalDataPages};
    use crate::messages::{RxMessageHeader, RxSyncByte, TransmitableMessage};
    use core::time::Duration;
    fn get_config() -> ChannelConfig {
        ChannelConfig {
            device_number: 1234,
            network_key_index: 0,
            device_type: 5,
            transmission_type: TransmissionType::new(
                TransmissionChannelType::IndependentChannel,
                TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
                12.into(),
            ),
            channel_type: ChannelType::BidirectionalSlave,
            radio_frequency: 25,
            timeout_duration: duration_to_search_timeout(Duration::from_secs(30)),
            channel_period: 123,
        }
    }

    fn get_response_ok(id: TxMessageId) -> AntMessage {
        AntMessage {
            header: RxMessageHeader {
                sync: RxSyncByte::Write,
                msg_id: crate::messages::RxMessageId::ChannelEvent,
                msg_length: 3,
            },
            message: RxMessage::ChannelResponse(ChannelResponse {
                channel_number: 0,
                message_id: id,
                message_code: MessageCode::ResponseNoError,
            }),
            checksum: 123, // this doesn't matter
        }
    }

    fn get_config_message(msg_handler: &mut MessageHandler, id: TxMessageId) -> TxMessage {
        while let Some(data) = msg_handler.send_message() {
            if data.get_tx_msg_id() == id {
                return data;
            }
            // Not our message, fake ok and resume
            msg_handler
                .receive_message(&get_response_ok(data.get_tx_msg_id()))
                .expect("State machine error");
        }
        panic!("Message not found")
    }

    #[test]
    fn inert_start() {
        let mut msg_handler = MessageHandler::new(4, &get_config());
        assert!(msg_handler.send_message().is_none());
    }

    #[test]
    fn assign_config() {
        let mut msg_handler = MessageHandler::new(4, &get_config());
        let data = get_config_message(&mut msg_handler, TxMessageId::AssignChannel);
        if let TxMessage::AssignChannel(data) = data {
            assert_eq!(data.data.channel_number, 4);
            assert_eq!(data.data.channel_type, ChannelType::BidirectionalSlave);
            assert_eq!(data.data.network_number, 0);
            assert_eq!(data.extended_assignment, None);
            return;
        }
        panic!("Message not found by helper");
    }

    #[test]
    fn close_state() {
        let mut msg_handler = MessageHandler::new(4, &get_config());
        let data = get_config_message(&mut msg_handler, TxMessageId::CloseChannel);
        if let TxMessage::CloseChannel(data) = data {
            assert_eq!(data.channel_number, 4);
            return;
        }
        panic!("Message not found by helper");
    }

    #[test]
    fn unassign_state() {
        let mut msg_handler = MessageHandler::new(&get_config());
        msg_handler.set_channel(ChannelAssignment::Assigned(4));
        let data = get_config_message(&mut msg_handler, TxMessageId::UnAssignChannel);
        if let TxMessage::UnAssignChannel(data) = data {
            assert_eq!(data.channel_number, 4);
            return;
        }
        panic!("Message not found by helper");
    }

    #[test]
    fn channel_id_state() {
        let mut msg_handler = MessageHandler::new(&get_config());
        msg_handler.set_channel(ChannelAssignment::Assigned(4));
        let data = get_config_message(&mut msg_handler, TxMessageId::ChannelId);
        if let TxMessage::ChannelId(data) = data {
            assert_eq!(data.channel_number, 4);
            assert_eq!(data.device_number, 1234);
            assert_eq!(data.device_type, DeviceType::new(5.into(), false));
            assert_eq!(
                data.transmission_type,
                TransmissionType::new(
                    TransmissionChannelType::IndependentChannel,
                    TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
                    12.into()
                )
            );
            return;
        }
        panic!("Message not found by helper");
    }

    #[test]
    fn channel_frequency_state() {
        let mut msg_handler = MessageHandler::new(&get_config());
        msg_handler.set_channel(ChannelAssignment::Assigned(4));
        let data = get_config_message(&mut msg_handler, TxMessageId::ChannelRfFrequency);
        if let TxMessage::ChannelRfFrequency(data) = data {
            assert_eq!(data.channel_number, 4);
            assert_eq!(data.rf_frequency, 25);
            return;
        }
        panic!("Message not found by helper");
    }

    #[test]
    fn channel_period_state() {
        let mut msg_handler = MessageHandler::new(&get_config());
        msg_handler.set_channel(ChannelAssignment::Assigned(4));
        let data = get_config_message(&mut msg_handler, TxMessageId::ChannelPeriod);
        if let TxMessage::ChannelPeriod(data) = data {
            assert_eq!(data.channel_number, 4);
            assert_eq!(data.channel_period, 123);
            return;
        }
        panic!("Message not found by helper");
    }

    #[test]
    fn search_timeout_state() {
        let mut msg_handler = MessageHandler::new(&get_config());
        msg_handler.set_channel(ChannelAssignment::Assigned(4));
        let data = get_config_message(&mut msg_handler, TxMessageId::SearchTimeout);
        if let TxMessage::SearchTimeout(data) = data {
            assert_eq!(data.channel_number, 4);
            assert_eq!(data.search_timeout, 12);
            return;
        }
        panic!("Message not found by helper");
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
    fn tx_signaling() {
        // TODO
    }

    #[test]
    fn reset() {
        // TODO
    }
}
