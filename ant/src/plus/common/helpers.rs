// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::fields::{
    ChannelType, DeviceType, MessageCode, TransmissionChannelType, TransmissionGlobalDataPages,
    TransmissionType, Wildcard,
};
use crate::messages::{
    AntMessage, AssignChannel, ChannelEvent, ChannelId, ChannelPeriod, ChannelResponse,
    ChannelRfFrequency, ChannelStatus, CloseChannel, OpenChannel, RxMessageType, SearchTimeout,
    TxMessage,
};
use crate::plus::ChannelAssignment;
use packed_struct::prelude::{packed_bits, Integer};

enum ConfigureState {
    Assign,
    Period,
    Id,
    Rf,
    Timeout,
    Done,
}

enum ChannelStateCommand {
    Open,
    Close,
}

pub enum TransmissionTypeAssignment {
    Wildcard(),
    DeviceNumberExtension(Integer<u8, packed_bits::Bits4>),
}

pub struct ProfileReference {
    pub device_type: u8,
    pub channel_type: TransmissionChannelType, // ignoring device number extension
    pub global_datapages_used: TransmissionGlobalDataPages,
    pub radio_frequency: u8,
    pub timeout_duration: u8,
    pub channel_period: u16,
}

pub struct MessageHandler {
    channel: ChannelAssignment,
    device_number: u16,
    transmission_type: TransmissionTypeAssignment,
    network_key_index: u8,
    pairing_request: bool,
    configure_state: ConfigureState,
    set_channel_state: Option<ChannelStateCommand>,
    pending_datapage: Option<TxMessage>,
    profile_reference: &'static ProfileReference,
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
            device_number,
            transmission_type,
            network_key_index: ant_plus_key_index,
            configure_state: ConfigureState::Assign,
            set_channel_state: None,
            pending_datapage: None,
            profile_reference,
            pairing_request: false,
        }
    }

    pub fn get_channel(&self) -> ChannelAssignment {
        self.channel
    }

    pub fn is_pending(&self) -> bool {
        self.pending_datapage.is_some()
    }

    pub fn set_sending(&mut self, msg: TxMessage) {
        self.pending_datapage = Some(msg);
    }

    pub fn send_message(&mut self) -> Option<TxMessage> {
        let channel = match self.channel {
            ChannelAssignment::UnAssigned() => return None,
            ChannelAssignment::Assigned(channel) => channel,
        };

        match self.configure_state {
            ConfigureState::Assign => {
                self.configure_state = ConfigureState::Period;
                return Some(
                    AssignChannel::new(
                        channel,
                        ChannelType::BidirectionalSlave,
                        self.network_key_index,
                        None,
                    )
                    .into(),
                );
            }
            ConfigureState::Period => {
                self.configure_state = ConfigureState::Id;
                return Some(
                    ChannelPeriod::new(channel, self.profile_reference.channel_period).into(),
                );
            }
            ConfigureState::Id => {
                self.configure_state = ConfigureState::Rf;
                return Some(
                    ChannelId::new(
                        channel,
                        self.device_number,
                        DeviceType::new(
                            self.profile_reference.device_type.into(),
                            self.pairing_request,
                        ),
                        match self.transmission_type {
                            TransmissionTypeAssignment::Wildcard() => {
                                TransmissionType::new_wildcard()
                            }
                            TransmissionTypeAssignment::DeviceNumberExtension(dev) => {
                                TransmissionType::new(
                                    self.profile_reference.channel_type,
                                    self.profile_reference.global_datapages_used,
                                    dev,
                                )
                            }
                        },
                    )
                    .into(),
                );
            }
            ConfigureState::Rf => {
                self.configure_state = ConfigureState::Timeout;
                return Some(
                    ChannelRfFrequency::new(channel, self.profile_reference.radio_frequency).into(),
                );
            }
            ConfigureState::Timeout => {
                self.configure_state = ConfigureState::Done;
                return Some(
                    SearchTimeout::new(channel, self.profile_reference.timeout_duration).into(),
                );
            }
            ConfigureState::Done => (),
        };
        if let Some(command) = &self.set_channel_state {
            let msg = match command {
                ChannelStateCommand::Open => OpenChannel::new(channel).into(),
                ChannelStateCommand::Close => CloseChannel::new(channel).into(),
            };
            self.set_channel_state = None;
            return Some(msg);
        };
        // TODO check if we need to request channel info once bonded
        self.pending_datapage.take()
    }

    pub fn receive_message(&mut self, msg: &AntMessage) {
        match &msg.message {
            RxMessageType::ChannelResponse(msg) => self.handle_response(msg),
            RxMessageType::ChannelEvent(msg) => self.handle_event(msg),
            RxMessageType::ChannelId(msg) => self.handle_id(msg),
            RxMessageType::ChannelStatus(msg) => self.handle_status(msg),
            _ => (),
        }
    }

    fn handle_status(&mut self, msg: &ChannelStatus) {
        match msg.channel_state {
            _ => (),
        }
    }

    fn handle_response(&mut self, msg: &ChannelResponse) {
        match msg.message_code {
            _ => (),
        }
    }

    fn handle_event(&mut self, msg: &ChannelEvent) {
        match msg.payload.message_code {
            MessageCode::EventChannelClosed => (),
            MessageCode::EventRxFailGoToSearch => (),
            _ => (),
        }
    }

    fn handle_id(&mut self, msg: &ChannelId) {
        self.device_number = msg.device_number;
        // TODO copy rest of state
    }

    pub fn open(&mut self) {
        self.set_channel_state = Some(ChannelStateCommand::Open);
    }

    pub fn close(&mut self) {
        self.set_channel_state = Some(ChannelStateCommand::Close);
    }

    pub fn set_channel(&mut self, channel: ChannelAssignment) {
        self.channel = channel;
        self.configure_state = ConfigureState::Assign;
    }
}
