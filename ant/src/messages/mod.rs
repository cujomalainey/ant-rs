// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::messages::config::{
    AddChannelIdToList, AddEncryptionIdToList, AssignChannel, ChannelId, ChannelPeriod,
    ChannelRfFrequency, ChannelSearchPriority, ChannelSearchSharing, ConfigEncryptionIdList,
    ConfigIdList, ConfigureAdvancedBurst, ConfigureEventBuffer, ConfigureEventFilter,
    ConfigureSelectiveDataUpdates, CrystalEnable, EnableExtRxMessages, EnableLed,
    EnableSingleChannelEncryption, FrequencyAgility, HighDutySearch, LibConfig,
    LoadEncryptionKeyFromNvm, LowPrioritySearchTimeout, ProximitySearch, SearchTimeout,
    SearchWaveform, SerialNumberSetChannelId, Set128BitNetworkKey, SetChannelTransmitPower,
    SetEncryptionInfoEncryptionId, SetEncryptionInfoRandomSeed,
    SetEncryptionInfoUserInformationString, SetEncryptionKey, SetNetworkKey,
    SetSelectiveDataUpdateMask, StoreEncryptionKeyInNvm, TransmitPower, UnAssignChannel,
};
use channel::{ChannelEvent, ChannelResponse};
use control::{CloseChannel, OpenChannel, RequestMessage, ResetSystem, SleepMessage};
use data::{
    AcknowledgedData, AdvancedBurstData, BroadcastData, BurstTransferData,
    ADVANCED_BURST_BUFFER_SIZE,
};
use notifications::{SerialErrorMessage, StartUpMessage};
use packed_struct::prelude::*;
use requested_response::{
    AdvancedBurstCapabilities, AdvancedBurstCurrentConfiguration, AntVersion, Capabilities,
    ChannelStatus, EncryptionModeParameters, EventBufferConfiguration, EventFilter,
    SelectiveDataUpdateMaskSetting, SerialNumber, UserNvm,
};
use test_mode::{CwInit, CwTest};

pub mod channel;
pub mod config;
pub mod control;
pub mod data;
pub mod notifications;
pub mod requested_response;
pub mod test_mode;

// TODO fixup
pub(crate) const MAX_MESSAGE_DATA_SIZE: usize = ADVANCED_BURST_BUFFER_SIZE + 1;

/// All supported RX messages
#[derive(Clone, PartialEq, Debug)]
pub enum RxMessage {
    // Notification Messages
    StartUpMessage(StartUpMessage),
    // #define SERIAL_ERROR_MESSAGE                0xAE
    // Data Messages
    BroadcastData(BroadcastData),
    AcknowledgedData(AcknowledgedData),
    BurstTransferData(BurstTransferData),
    AdvancedBurstData(AdvancedBurstData),
    // Channel Messages
    ChannelEvent(ChannelEvent),
    ChannelResponse(ChannelResponse),
    SerialErrorMessage(SerialErrorMessage),
    // Requested Response Messages
    ChannelStatus(ChannelStatus),
    ChannelId(ChannelId),
    AntVersion(AntVersion),
    Capabilities(Capabilities),
    SerialNumber(SerialNumber),
    EventBufferConfiguration(EventBufferConfiguration),
    AdvancedBurstCapabilities(AdvancedBurstCapabilities),
    AdvancedBurstCurrentConfiguration(AdvancedBurstCurrentConfiguration),
    EventFilter(EventFilter),
    SelectiveDataUpdateMaskSetting(SelectiveDataUpdateMaskSetting),
    UserNvm(UserNvm),
    EncryptionModeParameters(EncryptionModeParameters),
    // Extended Data Messages (Legacy)
    // #define EXTENDED_BROADCAST_DATA             0x5D
    // #define EXTENDED_ACKNOWLEDGED_DATA          0x5E
    // #define EXTENDED_BURST_DATA                 0x5F
}

#[derive(Clone, Debug)]
pub enum TxMessage {
    UnAssignChannel(UnAssignChannel),
    AssignChannel(AssignChannel),
    ChannelId(ChannelId),
    ChannelPeriod(ChannelPeriod),
    SearchTimeout(SearchTimeout),
    ChannelRfFrequency(ChannelRfFrequency),
    SetNetworkKey(SetNetworkKey),
    TransmitPower(TransmitPower),
    SearchWaveform(SearchWaveform),
    AddChannelIdToList(AddChannelIdToList),
    AddEncryptionIdToList(AddEncryptionIdToList),
    ConfigIdList(ConfigIdList),
    ConfigEncryptionIdList(ConfigEncryptionIdList),
    SetChannelTransmitPower(SetChannelTransmitPower),
    LowPrioritySearchTimeout(LowPrioritySearchTimeout),
    SerialNumberSetChannelId(SerialNumberSetChannelId),
    EnableExtRxMessages(EnableExtRxMessages),
    EnableLed(EnableLed),
    CrystalEnable(CrystalEnable),
    LibConfig(LibConfig),
    FrequencyAgility(FrequencyAgility),
    ProximitySearch(ProximitySearch),
    ConfigureEventBuffer(ConfigureEventBuffer),
    ChannelSearchPriority(ChannelSearchPriority),
    Set128BitNetworkKey(Set128BitNetworkKey),
    HighDutySearch(HighDutySearch),
    ConfigureAdvancedBurst(ConfigureAdvancedBurst),
    ConfigureEventFilter(ConfigureEventFilter),
    ConfigureSelectiveDataUpdates(ConfigureSelectiveDataUpdates),
    SetSelectiveDataUpdateMask(SetSelectiveDataUpdateMask),
    // ConfigureUserNvm(ConfigureUserNvm),
    EnableSingleChannelEncryption(EnableSingleChannelEncryption),
    SetEncryptionKey(SetEncryptionKey),
    SetEncryptionInfoEncryptionId(SetEncryptionInfoEncryptionId),
    SetEncryptionInfoRandomSeed(SetEncryptionInfoRandomSeed),
    SetEncryptionInfoUserInformationString(SetEncryptionInfoUserInformationString),
    ChannelSearchSharing(ChannelSearchSharing),
    LoadEncryptionKeyFromNvm(LoadEncryptionKeyFromNvm),
    StoreEncryptionKeyInNvm(StoreEncryptionKeyInNvm),
    // SetUsbDescriptorString(SetUsbDescriptorString),
    ResetSystem(ResetSystem),
    OpenChannel(OpenChannel),
    CloseChannel(CloseChannel),
    RequestMessage(RequestMessage),
    // OpenRxScanMode(OpenRxScanMode),
    SleepMessage(SleepMessage),
    BroadcastData(BroadcastData),
    AcknowledgedData(AcknowledgedData),
    BurstTransferData(BurstTransferData),
    AdvancedBurstData(AdvancedBurstData),
    CwInit(CwInit),
    CwTest(CwTest),
}

// Hack to allow channels to recycle memory, not for actual use
impl Default for TxMessage {
    fn default() -> TxMessage {
        TxMessage::UnAssignChannel(UnAssignChannel { channel_number: 0 })
    }
}

impl TransmitableMessage for TxMessage {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
        match self {
            TxMessage::UnAssignChannel(uc) => uc.serialize_message(buf),
            TxMessage::AssignChannel(ac) => ac.serialize_message(buf),
            TxMessage::ChannelId(id) => id.serialize_message(buf),
            TxMessage::ChannelPeriod(cp) => cp.serialize_message(buf),
            TxMessage::SearchTimeout(st) => st.serialize_message(buf),
            TxMessage::ChannelRfFrequency(cr) => cr.serialize_message(buf),
            TxMessage::SetNetworkKey(cc) => cc.serialize_message(buf),
            TxMessage::TransmitPower(tp) => tp.serialize_message(buf),
            TxMessage::SearchWaveform(sw) => sw.serialize_message(buf),
            TxMessage::AddChannelIdToList(ac) => ac.serialize_message(buf),
            TxMessage::AddEncryptionIdToList(ae) => ae.serialize_message(buf),
            TxMessage::ConfigIdList(cl) => cl.serialize_message(buf),
            TxMessage::ConfigEncryptionIdList(ce) => ce.serialize_message(buf),
            TxMessage::SetChannelTransmitPower(sc) => sc.serialize_message(buf),
            TxMessage::LowPrioritySearchTimeout(lp) => lp.serialize_message(buf),
            TxMessage::SerialNumberSetChannelId(sn) => sn.serialize_message(buf),
            TxMessage::EnableExtRxMessages(ee) => ee.serialize_message(buf),
            TxMessage::EnableLed(el) => el.serialize_message(buf),
            TxMessage::CrystalEnable(ce) => ce.serialize_message(buf),
            TxMessage::LibConfig(lc) => lc.serialize_message(buf),
            TxMessage::FrequencyAgility(fa) => fa.serialize_message(buf),
            TxMessage::ProximitySearch(ps) => ps.serialize_message(buf),
            TxMessage::ConfigureEventBuffer(ce) => ce.serialize_message(buf),
            TxMessage::ChannelSearchPriority(cs) => cs.serialize_message(buf),
            TxMessage::Set128BitNetworkKey(sb) => sb.serialize_message(buf),
            TxMessage::HighDutySearch(hd) => hd.serialize_message(buf),
            TxMessage::ConfigureAdvancedBurst(ca) => ca.serialize_message(buf),
            TxMessage::ConfigureEventFilter(ce) => ce.serialize_message(buf),
            TxMessage::ConfigureSelectiveDataUpdates(cs) => cs.serialize_message(buf),
            TxMessage::SetSelectiveDataUpdateMask(ss) => ss.serialize_message(buf),
            // ConfigureUserNvm(ConfigureUserNvm),
            TxMessage::EnableSingleChannelEncryption(es) => es.serialize_message(buf),
            TxMessage::SetEncryptionKey(se) => se.serialize_message(buf),
            TxMessage::SetEncryptionInfoEncryptionId(se) => se.serialize_message(buf),
            TxMessage::SetEncryptionInfoRandomSeed(se) => se.serialize_message(buf),
            TxMessage::SetEncryptionInfoUserInformationString(se) => se.serialize_message(buf),
            TxMessage::ChannelSearchSharing(cs) => cs.serialize_message(buf),
            TxMessage::LoadEncryptionKeyFromNvm(le) => le.serialize_message(buf),
            TxMessage::StoreEncryptionKeyInNvm(se) => se.serialize_message(buf),
            // SetUsbDescriptorString(SetUsbDescriptorString),
            TxMessage::ResetSystem(rs) => rs.serialize_message(buf),
            TxMessage::OpenChannel(oc) => oc.serialize_message(buf),
            TxMessage::CloseChannel(cc) => cc.serialize_message(buf),
            TxMessage::RequestMessage(rm) => rm.serialize_message(buf),
            // TxMessage::OpenRxScanMode(or) => or.serialize_message(buf),
            TxMessage::SleepMessage(sm) => sm.serialize_message(buf),
            TxMessage::BroadcastData(bd) => bd.serialize_message(buf),
            TxMessage::AcknowledgedData(ad) => ad.serialize_message(buf),
            TxMessage::BurstTransferData(bt) => bt.serialize_message(buf),
            TxMessage::AdvancedBurstData(ab) => ab.serialize_message(buf),
            TxMessage::CwInit(ci) => ci.serialize_message(buf),
            TxMessage::CwTest(ct) => ct.serialize_message(buf),
        }
    }

    fn get_tx_msg_id(&self) -> TxMessageId {
        match self {
            TxMessage::UnAssignChannel(uc) => uc.get_tx_msg_id(),
            TxMessage::AssignChannel(ac) => ac.get_tx_msg_id(),
            TxMessage::ChannelId(id) => id.get_tx_msg_id(),
            TxMessage::ChannelPeriod(cp) => cp.get_tx_msg_id(),
            TxMessage::SearchTimeout(st) => st.get_tx_msg_id(),
            TxMessage::ChannelRfFrequency(cr) => cr.get_tx_msg_id(),
            TxMessage::SetNetworkKey(cc) => cc.get_tx_msg_id(),
            TxMessage::TransmitPower(tp) => tp.get_tx_msg_id(),
            TxMessage::SearchWaveform(sw) => sw.get_tx_msg_id(),
            TxMessage::AddChannelIdToList(ac) => ac.get_tx_msg_id(),
            TxMessage::AddEncryptionIdToList(ae) => ae.get_tx_msg_id(),
            TxMessage::ConfigIdList(cl) => cl.get_tx_msg_id(),
            TxMessage::ConfigEncryptionIdList(ce) => ce.get_tx_msg_id(),
            TxMessage::SetChannelTransmitPower(sc) => sc.get_tx_msg_id(),
            TxMessage::LowPrioritySearchTimeout(lp) => lp.get_tx_msg_id(),
            TxMessage::SerialNumberSetChannelId(sn) => sn.get_tx_msg_id(),
            TxMessage::EnableExtRxMessages(ee) => ee.get_tx_msg_id(),
            TxMessage::EnableLed(el) => el.get_tx_msg_id(),
            TxMessage::CrystalEnable(ce) => ce.get_tx_msg_id(),
            TxMessage::LibConfig(lc) => lc.get_tx_msg_id(),
            TxMessage::FrequencyAgility(fa) => fa.get_tx_msg_id(),
            TxMessage::ProximitySearch(ps) => ps.get_tx_msg_id(),
            TxMessage::ConfigureEventBuffer(ce) => ce.get_tx_msg_id(),
            TxMessage::ChannelSearchPriority(cs) => cs.get_tx_msg_id(),
            TxMessage::Set128BitNetworkKey(sb) => sb.get_tx_msg_id(),
            TxMessage::HighDutySearch(hd) => hd.get_tx_msg_id(),
            TxMessage::ConfigureAdvancedBurst(ca) => ca.get_tx_msg_id(),
            TxMessage::ConfigureEventFilter(ce) => ce.get_tx_msg_id(),
            TxMessage::ConfigureSelectiveDataUpdates(cs) => cs.get_tx_msg_id(),
            TxMessage::SetSelectiveDataUpdateMask(ss) => ss.get_tx_msg_id(),
            // ConfigureUserNvm(ConfigureUserNvm),
            TxMessage::EnableSingleChannelEncryption(es) => es.get_tx_msg_id(),
            TxMessage::SetEncryptionKey(se) => se.get_tx_msg_id(),
            TxMessage::SetEncryptionInfoEncryptionId(se) => se.get_tx_msg_id(),
            TxMessage::SetEncryptionInfoRandomSeed(se) => se.get_tx_msg_id(),
            TxMessage::SetEncryptionInfoUserInformationString(se) => se.get_tx_msg_id(),
            TxMessage::ChannelSearchSharing(cs) => cs.get_tx_msg_id(),
            TxMessage::LoadEncryptionKeyFromNvm(le) => le.get_tx_msg_id(),
            TxMessage::StoreEncryptionKeyInNvm(se) => se.get_tx_msg_id(),
            // TODO SetUsbDescriptorString(SetUsbDescriptorString),
            TxMessage::ResetSystem(rs) => rs.get_tx_msg_id(),
            TxMessage::OpenChannel(oc) => oc.get_tx_msg_id(),
            TxMessage::CloseChannel(cc) => cc.get_tx_msg_id(),
            TxMessage::RequestMessage(rm) => rm.get_tx_msg_id(),
            // TODO TxMessage::OpenRxScanMode(or) => or.serialize_message(buf),
            TxMessage::SleepMessage(sm) => sm.get_tx_msg_id(),
            TxMessage::BroadcastData(bd) => bd.get_tx_msg_id(),
            TxMessage::AcknowledgedData(ad) => ad.get_tx_msg_id(),
            TxMessage::BurstTransferData(bt) => bt.get_tx_msg_id(),
            TxMessage::AdvancedBurstData(ab) => ab.get_tx_msg_id(),
            TxMessage::CwInit(ci) => ci.get_tx_msg_id(),
            TxMessage::CwTest(ct) => ct.get_tx_msg_id(),
        }
    }
}

pub enum TxMessageData {
    BroadcastData(BroadcastData),
    AcknowledgedData(AcknowledgedData),
    BurstTransferData(BurstTransferData),
    AdvancedBurstData(AdvancedBurstData),
}

impl TxMessageData {
    /// Helper for profiles to set channel if relevant on external Tx requests
    pub(crate) fn set_channel(&mut self, channel: u8) {
        match self {
            TxMessageData::BroadcastData(bd) => bd.payload.channel_number = channel,
            TxMessageData::AcknowledgedData(ad) => ad.payload.channel_number = channel,
            TxMessageData::BurstTransferData(bt) => {
                bt.payload.channel_sequence.channel_number = channel.into()
            }
            TxMessageData::AdvancedBurstData(ab) => {
                ab.channel_sequence.channel_number = channel.into()
            }
        }
    }
}

impl From<TxMessageData> for TxMessage {
    fn from(msg: TxMessageData) -> TxMessage {
        match msg {
            TxMessageData::BroadcastData(bd) => bd.into(),
            TxMessageData::AcknowledgedData(ad) => ad.into(),
            TxMessageData::BurstTransferData(bt) => bt.into(),
            TxMessageData::AdvancedBurstData(ab) => ab.into(),
        }
    }
}

pub enum TxMessageChannelConfig {
    UnAssignChannel(UnAssignChannel),
    AssignChannel(AssignChannel),
    ChannelId(ChannelId),
    ChannelPeriod(ChannelPeriod),
    SearchTimeout(SearchTimeout),
    ChannelRfFrequency(ChannelRfFrequency),
    SearchWaveform(SearchWaveform),
    AddChannelIdToList(AddChannelIdToList),
    AddEncryptionIdToList(AddEncryptionIdToList),
    ConfigIdList(ConfigIdList),
    ConfigEncryptionIdList(ConfigEncryptionIdList),
    SetChannelTransmitPower(SetChannelTransmitPower),
    LowPrioritySearchTimeout(LowPrioritySearchTimeout),
    SerialNumberSetChannelId(SerialNumberSetChannelId),
    FrequencyAgility(FrequencyAgility),
    ProximitySearch(ProximitySearch),
    ChannelSearchPriority(ChannelSearchPriority),
    ConfigureSelectiveDataUpdates(ConfigureSelectiveDataUpdates),
    EnableSingleChannelEncryption(EnableSingleChannelEncryption),
    ChannelSearchSharing(ChannelSearchSharing),
    // Skip Open / Close as user should use API for that
    RequestMessage(RequestMessage),
}

impl TxMessageChannelConfig {
    /// Helper for profiles to set channel if relevant on external Tx requests
    pub(crate) fn set_channel(&mut self, channel: u8) {
        match self {
            TxMessageChannelConfig::UnAssignChannel(uc) => uc.channel_number = channel,
            TxMessageChannelConfig::AssignChannel(ac) => ac.data.channel_number = channel,
            TxMessageChannelConfig::ChannelId(id) => id.channel_number = channel,
            TxMessageChannelConfig::ChannelPeriod(cp) => cp.channel_number = channel,
            TxMessageChannelConfig::SearchTimeout(st) => st.channel_number = channel,
            TxMessageChannelConfig::ChannelRfFrequency(cr) => cr.channel_number = channel,
            TxMessageChannelConfig::SearchWaveform(sw) => sw.channel_number = channel,
            TxMessageChannelConfig::AddChannelIdToList(ac) => ac.channel_number = channel,
            TxMessageChannelConfig::AddEncryptionIdToList(ae) => ae.channel_number = channel,
            TxMessageChannelConfig::ConfigIdList(cl) => cl.channel_number = channel,
            TxMessageChannelConfig::ConfigEncryptionIdList(ce) => ce.channel_number = channel,
            TxMessageChannelConfig::SetChannelTransmitPower(sc) => sc.channel_number = channel,
            TxMessageChannelConfig::LowPrioritySearchTimeout(lp) => lp.channel_number = channel,
            TxMessageChannelConfig::SerialNumberSetChannelId(sn) => sn.channel_number = channel,
            TxMessageChannelConfig::FrequencyAgility(fa) => fa.channel_number = channel,
            TxMessageChannelConfig::ProximitySearch(ps) => ps.channel_number = channel,
            TxMessageChannelConfig::ChannelSearchPriority(cs) => cs.channel_number = channel,
            TxMessageChannelConfig::ConfigureSelectiveDataUpdates(cs) => {
                cs.channel_number = channel
            }
            TxMessageChannelConfig::EnableSingleChannelEncryption(es) => {
                es.channel_number = channel
            }
            TxMessageChannelConfig::ChannelSearchSharing(cs) => cs.channel_number = channel,
            TxMessageChannelConfig::RequestMessage(rm) => rm.data.channel = channel,
        }
    }
}

impl From<TxMessageChannelConfig> for TxMessage {
    fn from(msg: TxMessageChannelConfig) -> TxMessage {
        match msg {
            TxMessageChannelConfig::UnAssignChannel(uc) => uc.into(),
            TxMessageChannelConfig::AssignChannel(ac) => ac.into(),
            TxMessageChannelConfig::ChannelId(id) => id.into(),
            TxMessageChannelConfig::ChannelPeriod(cp) => cp.into(),
            TxMessageChannelConfig::SearchTimeout(st) => st.into(),
            TxMessageChannelConfig::ChannelRfFrequency(cr) => cr.into(),
            TxMessageChannelConfig::SearchWaveform(sw) => sw.into(),
            TxMessageChannelConfig::AddChannelIdToList(ac) => ac.into(),
            TxMessageChannelConfig::AddEncryptionIdToList(ae) => ae.into(),
            TxMessageChannelConfig::ConfigIdList(cl) => cl.into(),
            TxMessageChannelConfig::ConfigEncryptionIdList(ce) => ce.into(),
            TxMessageChannelConfig::SetChannelTransmitPower(sc) => sc.into(),
            TxMessageChannelConfig::LowPrioritySearchTimeout(lp) => lp.into(),
            TxMessageChannelConfig::SerialNumberSetChannelId(sn) => sn.into(),
            TxMessageChannelConfig::FrequencyAgility(fa) => fa.into(),
            TxMessageChannelConfig::ProximitySearch(ps) => ps.into(),
            TxMessageChannelConfig::ChannelSearchPriority(cs) => cs.into(),
            TxMessageChannelConfig::ConfigureSelectiveDataUpdates(cs) => cs.into(),
            TxMessageChannelConfig::EnableSingleChannelEncryption(es) => es.into(),
            TxMessageChannelConfig::ChannelSearchSharing(cs) => cs.into(),
            TxMessageChannelConfig::RequestMessage(rm) => rm.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Represents a generic ANT radio message
pub struct AntMessage {
    pub header: RxMessageHeader,
    pub message: RxMessage,
    /// XOR of all prior bytes should match this
    pub checksum: u8,
}

// Hack to allow memory channels to recycle, not intended for actual use
impl Default for AntMessage {
    fn default() -> AntMessage {
        AntMessage {
            header: RxMessageHeader {
                sync: RxSyncByte::Read,
                msg_length: 0,
                msg_id: RxMessageId::StartUpMessage,
            },
            message: RxMessage::StartUpMessage(StartUpMessage {
                hardware_reset_line: false,
                watch_dog_reset: false,
                command_reset: false,
                synchronous_reset: false,
                suspend_reset: false,
            }),
            checksum: 0,
        }
    }
}

/// Trait for any TX message type
pub trait TransmitableMessage {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError>;
    fn get_tx_msg_id(&self) -> TxMessageId;
}

macro_rules! AntAutoPackWithExtention {
    ($msg_type:ident, $id:expr, $main_field:ident, $ext_field:ident) => {
        impl TransmitableMessage for $msg_type {
            fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
                let data_len = PackedStructSlice::packed_bytes_size(Some(&self.$main_field))?;
                self.$main_field.pack_to_slice(&mut buf[..data_len])?;

                if let Some(ext) = self.$ext_field {
                    let ext_len = PackedStructSlice::packed_bytes_size(Some(&ext))?;
                    ext.pack_to_slice(&mut buf[data_len..data_len + ext_len])?;
                    return Ok(data_len + ext_len);
                }
                Ok(data_len)
            }
            fn get_tx_msg_id(&self) -> TxMessageId {
                $id
            }
        }
        impl From<$msg_type> for TxMessage {
            fn from(msg: $msg_type) -> TxMessage {
                TxMessage::$msg_type(msg)
            }
        }
    };
}

pub(crate) use AntAutoPackWithExtention;

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum RxSyncByte {
    Write = 0xA4,
    Read = 0xA5,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum TxSyncByte {
    Value = 0xA4,
}

#[derive(PackedStruct, Debug, PartialEq, Clone, Copy)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct RxMessageHeader {
    #[packed_field(bytes = "0", ty = "enum")]
    pub sync: RxSyncByte,
    #[packed_field(bytes = "1")]
    pub msg_length: u8,
    #[packed_field(bytes = "2", ty = "enum")]
    pub msg_id: RxMessageId,
}

#[derive(PackedStruct, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct TxMessageHeader {
    #[packed_field(bytes = "0", ty = "enum")]
    pub sync: TxSyncByte,
    #[packed_field(bytes = "1")]
    pub msg_length: u8,
    #[packed_field(bytes = "2", ty = "enum")]
    pub msg_id: TxMessageId,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum RxMessageId {
    // Notification Messages
    StartUpMessage = 0x6F,
    SerialErrorMessage = 0xAE,
    // Data Messages
    BroadcastData = 0x4E,
    AcknowledgedData = 0x4F,
    BurstTransferData = 0x50,
    AdvancedBurstData = 0x72,
    // Channel Messages
    ChannelEvent = 0x40,
    // ChannelResponse                 = 0x40,
    // Requested Response Messages
    ChannelStatus = 0x52,
    ChannelId = 0x51,
    AntVersion = 0x3E,
    Capabilities = 0x54,
    SerialNumber = 0x61,
    EventBufferConfiguration = 0x74,
    AdvancedBurstCapabilities = 0x78,
    // AdvancedBurstCurrentConfiguration      = 0x78,
    EventFilter = 0x79,
    SelectiveDataUpdateMaskSetting = 0x7B,
    UserNvm = 0x7C,
    EncryptionModeParameters = 0x7D,
    // Extended Data Messages (Legacy)
    // #define EXTENDED_BROADCAST_DATA             0x5D
    // #define EXTENDED_ACKNOWLEDGED_DATA          0x5E
    // #define EXTENDED_BURST_DATA                 0x5F
}

// Impl all the duplicate field names
#[allow(non_upper_case_globals)]
impl RxMessageId {}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum TxMessageId {
    // Config Messages
    UnAssignChannel = 0x41,
    AssignChannel = 0x42,
    ChannelId = 0x51,
    ChannelPeriod = 0x43,
    SearchTimeout = 0x44,
    ChannelRfFrequency = 0x45,
    SetNetworkKey = 0x46,
    TransmitPower = 0x47,
    SearchWaveform = 0x49,
    AddChannelIdToList = 0x59,
    // AddEncryptionIdToList           = 0x59,
    ConfigIdList = 0x5A,
    // ConfigEncryptionIdList          = 0x5A,
    SetChannelTransmitPower = 0x60,
    LowPrioritySearchTimeout = 0x63,
    SerialNumberSetChannelId = 0x65,
    EnableExtRxMessages = 0x66,
    EnableLed = 0x68,
    CrystalEnable = 0x6D,
    LibConfig = 0x6E,
    FrequencyAgility = 0x70,
    ProximitySearch = 0x71,
    ConfigureEventBuffer = 0x74,
    ChannelSearchPriority = 0x75,
    Set128BitNetworkKey = 0x76,
    HighDutySearch = 0x77,
    ConfigureAdvancedBurst = 0x78,
    ConfigureEventFilter = 0x79,
    ConfigureSelectiveDataUpdates = 0x7A,
    SetSelectiveDataUpdateMask = 0x7B,
    // #define CONFIGURE_USER_NVM                  0x7C
    EnableSingleChannelEncryption = 0x7D,
    SetEncryptionKey = 0x7E,
    SetEncryptionInfo = 0x7F,
    ChannelSearchSharing = 0x81,
    LoadStoreEncryptionKeyFromNvm = 0x83,
    // #define SET_USB_DESCRIPTOR_STRING           0xC7
    // Control Messages
    ResetSystem = 0x4A,
    OpenChannel = 0x4B,
    CloseChannel = 0x4C,
    RequestMessage = 0x4D,
    OpenRxScanMode = 0x5B,
    SleepMessage = 0xC5,
    // Data Messages
    BroadcastData = 0x4E,
    AcknowledgedData = 0x4F,
    BurstTransferData = 0x50,
    AdvancedBurstData = 0x72,
    // Test Mode Messages
    CwInit = 0x53,
    CwTest = 0x48,
    // Extended Data Messages (Legacy)
    // #define EXTENDED_BROADCAST_DATA             0x5D
    // #define EXTENDED_ACKNOWLEDGED_DATA          0x5E
    // #define EXTENDED_BURST_DATA                 0x5F
}

// Impl all the duplicate field names
#[allow(non_upper_case_globals)]
impl TxMessageId {
    pub const AddEncryptionIdToList: TxMessageId = TxMessageId::AddChannelIdToList;
    pub const ConfigEncryptionIdList: TxMessageId = TxMessageId::ConfigIdList;
    pub const SetEncryptionInfoEncryptionId: TxMessageId = TxMessageId::SetEncryptionInfo;
    pub const SetEncryptionInfoUserInformationString: TxMessageId = TxMessageId::SetEncryptionInfo;
    pub const SetEncryptionInfoRandomSeed: TxMessageId = TxMessageId::SetEncryptionInfo;
    pub const StoreEncryptionKeyInNvm: TxMessageId = TxMessageId::LoadStoreEncryptionKeyFromNvm;
    pub const LoadEncryptionKeyFromNvm: TxMessageId = TxMessageId::LoadStoreEncryptionKeyFromNvm;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rx_message_header() {
        let packed = RxMessageHeader {
            sync: RxSyncByte::Write,
            msg_length: 1,
            msg_id: RxMessageId::StartUpMessage,
        };
        assert_eq!(packed.pack().unwrap(), [0xA4, 1, 0x6F]);
    }

    #[test]
    fn tx_message_header() {
        let packed = TxMessageHeader {
            sync: TxSyncByte::Value,
            msg_length: 1,
            msg_id: TxMessageId::ChannelId,
        };
        assert_eq!(packed.pack().unwrap(), [0xA4, 1, 0x51]);
    }
}
