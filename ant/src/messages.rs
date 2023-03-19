// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::drivers::DriverError;
pub use crate::fields::*;
use ant_derive::AntTx;
use arrayvec::ArrayVec;
use const_utils::{max, min};
use konst::{option::unwrap_or, primitive::parse_usize, unwrap_ctx};
use packed_struct::prelude::*;
use std::convert::{TryFrom, TryInto};

// TODO make this crash compilation if out of bounds rather than silently correct
// TODO skip this if NVM is enabled
const ADVANCED_BURST_BUFFER_SIZE: usize = min(
    max(
        unwrap_ctx!(parse_usize(unwrap_or!(
            option_env!("ADV_BURST_BUF_SIZE"),
            "64"
        ))),
        24,
    ),
    254,
);
pub(crate) const MAX_MESSAGE_DATA_SIZE: usize = ADVANCED_BURST_BUFFER_SIZE + 1;

// TODO remove "type" suffix
/// All supported RX messages
#[derive(Clone, PartialEq, Debug)]
pub enum RxMessageType {
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
    AdvancedBurstConfiguration(AdvancedBurstCurrentConfiguration),
    EventFilter(EventFilter),
    SelectiveDataUpdateMaskSetting(SelectiveDataUpdateMaskSetting),
    UserNvm(UserNvm),
    EncryptionModeParameters(EncryptionModeParameters),
    // Extended Data Messages (Legacy)
    // #define EXTENDED_BROADCAST_DATA             0x5D
    // #define EXTENDED_ACKNOWLEDGED_DATA          0x5E
    // #define EXTENDED_BURST_DATA                 0x5F
}

pub enum TxMessage {
    AssignChannel(AssignChannel),
    ChannelId(ChannelId),
    ChannelPeriod(ChannelPeriod),
    ChannelRfFrequency(ChannelRfFrequency),
    SearchTimeout(SearchTimeout),
    OpenChannel(OpenChannel),
    CloseChannel(CloseChannel),
}

impl AntTxMessageType for TxMessage {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
        match self {
            TxMessage::AssignChannel(ac) => ac.serialize_message(buf),
            TxMessage::ChannelId(id) => id.serialize_message(buf),
            TxMessage::ChannelPeriod(cp) => cp.serialize_message(buf),
            TxMessage::ChannelRfFrequency(cr) => cr.serialize_message(buf),
            TxMessage::SearchTimeout(st) => st.serialize_message(buf),
            TxMessage::OpenChannel(oc) => oc.serialize_message(buf),
            TxMessage::CloseChannel(cc) => cc.serialize_message(buf),
        }
    }
    fn get_tx_msg_id(&self) -> TxMessageId {
        match self {
            TxMessage::AssignChannel(ac) => ac.get_tx_msg_id(),
            TxMessage::ChannelId(id) => id.get_tx_msg_id(),
            TxMessage::ChannelPeriod(cp) => cp.get_tx_msg_id(),
            TxMessage::ChannelRfFrequency(cr) => cr.get_tx_msg_id(),
            TxMessage::SearchTimeout(st) => st.get_tx_msg_id(),
            TxMessage::OpenChannel(oc) => oc.get_tx_msg_id(),
            TxMessage::CloseChannel(cc) => cc.get_tx_msg_id(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Represents a generic ANT radio message
pub struct AntMessage {
    pub header: RxMessageHeader,
    pub message: RxMessageType,
    /// XOR of all prior bytes should match this
    pub checksum: u8,
}

// TODO remove "ant"
/// Trait for any TX message type
pub trait AntTxMessageType {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError>;
    fn get_tx_msg_id(&self) -> TxMessageId;
}

macro_rules! AntAutoPackWithExtention {
    ($msg_type:ty, $id:expr, $main_field:ident, $ext_field:ident) => {
        impl AntTxMessageType for $msg_type {
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
    };
}

// Message Types

/// Represents a UnAssign Channel Message (0x41)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct UnAssignChannel {
    /// Channel to be unassigned
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
}

impl UnAssignChannel {
    /// Creates a new UnAssign Channel message
    pub fn new(channel_number: u8) -> Self {
        Self { channel_number }
    }
}

/// Mandatory fields for [AssignChannel] messages
#[derive(PackedStruct, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct AssignChannelData {
    /// Channel to be initialized
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    #[packed_field(bits = "12:15")]
    _reserved: ReservedZeroes<packed_bits::Bits4>,
    /// Channel type to be configured
    #[packed_field(bits = "8:11", ty = "enum")]
    pub channel_type: ChannelType,
    /// Which network key to use, set keys via [SetNetworkKey]
    #[packed_field(bytes = "2")]
    pub network_number: u8,
}

/// Represents a Assign Channel message (0x42)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AssignChannel {
    /// Mandatory fields
    pub data: AssignChannelData,
    /// Optional fields
    pub extended_assignment: Option<ExtendedAssignment>,
}
AntAutoPackWithExtention!(
    AssignChannel,
    TxMessageId::AssignChannel,
    data,
    extended_assignment
);

impl AssignChannel {
    /// Creates a new Assign Channel message
    pub fn new(
        channel_number: u8,
        channel_type: ChannelType,
        network_number: u8,
        extended_assignment: Option<ExtendedAssignment>,
    ) -> Self {
        Self {
            data: AssignChannelData {
                channel_number,
                channel_type,
                network_number,
                ..AssignChannelData::default()
            },
            extended_assignment,
        }
    }
}

/// Represents a Channel Id message (0x51)
///
/// This message is both RX and TX capable
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "5")]
pub struct ChannelId {
    /// Channel number to configure or from request
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Device ID of channel
    ///
    /// if this is a slave channel and was wildcarded initiallially this will contain the master's
    /// ID once a connection is formed
    #[packed_field(bytes = "1:2")]
    pub device_number: u16,
    /// Device type
    ///
    /// if this is a slave channel and was wildcarded initiallially this will contain the master's
    /// type once a connection is formed
    #[packed_field(bytes = "3")]
    pub device_type: DeviceType,
    /// Transmission type
    ///
    /// if this is a slave channel and was wildcarded initiallially this will contain the master's
    /// type once a connection is formed
    #[packed_field(bytes = "4")]
    pub transmission_type: TransmissionType,
}

impl ChannelId {
    /// Creates a new ChannelId message
    pub fn new(
        channel_number: u8,
        device_number: u16,
        device_type: DeviceType,
        transmission_type: TransmissionType,
    ) -> Self {
        Self {
            channel_number,
            device_number,
            device_type,
            transmission_type,
        }
    }
}

impl Wildcard for ChannelId {
    /// Set all fields to their wildcard values
    fn wildcard(&mut self) {
        self.device_number = 0;
        self.device_type.wildcard();
        self.transmission_type.wildcard();
    }

    /// Make a new ChannelId message with wildcard values
    fn new_wildcard() -> Self {
        Self {
            channel_number: 0,
            device_number: 0,
            device_type: DeviceType::new_wildcard(),
            transmission_type: TransmissionType::new_wildcard(),
        }
    }
}

/// Represents a Channel Period message (0x43)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct ChannelPeriod {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Period to be used
    ///
    /// 32768 / message frequency = period
    #[packed_field(bytes = "1:2")]
    pub channel_period: u16,
}

impl ChannelPeriod {
    /// Creates a new Channel Period message
    pub fn new(channel_number: u8, channel_period: u16) -> Self {
        Self {
            channel_number,
            channel_period,
        }
    }
}

/// Represents a Search Timeout message (0x44)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct SearchTimeout {
    /// Channel to configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Search timeout to be set
    ///
    /// 2.5s * search_timeout = time searching
    /// 0 - no search
    /// 255 - infinite search
    #[packed_field(bytes = "1")]
    pub search_timeout: u8,
}

impl SearchTimeout {
    /// Creates a new Search Timeout message
    pub fn new(channel_number: u8, search_timeout: u8) -> Self {
        Self {
            channel_number,
            search_timeout,
        }
    }
}

/// Represents a Channel RF Frequency (0x45)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct ChannelRfFrequency {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Frequency for channel to operate at
    ///
    /// 2400 MHz + rf_frequency = operating frequency
    #[packed_field(bytes = "1")]
    pub rf_frequency: u8,
}

impl ChannelRfFrequency {
    /// Creates a new Channel RF Frequency message
    pub fn new(channel_number: u8, rf_frequency: u8) -> Self {
        Self {
            channel_number,
            rf_frequency,
        }
    }
}

/// Size of a default network key
pub const NETWORK_KEY_SIZE: usize = 8;

/// Represents a Set Network Key message (0x46)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "9")]
pub struct SetNetworkKey {
    /// Network number to be used
    ///
    /// Max value is device dependent
    #[packed_field(bytes = "0")]
    pub network_number: u8,
    /// Key to be installed
    ///
    /// To use ANT+ or ANT-FS please go to [thisisant](http://www.thisisant.com/developer/ant-plus/ant-plus-basics/network-keys/) to get the appropriate keys
    #[packed_field(bytes = "1:8")]
    pub network_key: [u8; 8], // AKA NETWORK_KEY_SIZE but PackedStruct doens't like const
}

impl SetNetworkKey {
    /// Creates a new Set Network Key message
    pub fn new(network_number: u8, network_key: [u8; NETWORK_KEY_SIZE]) -> Self {
        Self {
            network_number,
            network_key,
        }
    }
}

/// Represents a Transmit Power message (0x47)
///
/// Same as [SetChannelTransmitPower] but for all channels
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct TransmitPower {
    #[packed_field(bytes = "0")]
    _reserved: ReservedZeroes<packed_bits::Bits8>,
    /// Sets TX power for all channels
    ///
    /// Dbm correlation is chip dependent, please chip and ANT messaging documentation
    #[packed_field(bytes = "1")]
    pub tx_power: u8,
}

impl TransmitPower {
    /// Creates a new Transmit Power message
    pub fn new(tx_power: u8) -> Self {
        Self {
            tx_power,
            ..Self::default()
        }
    }
}

/// Represents a Search Waveform message (0x49)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct SearchWaveform {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Waveform to use
    ///
    /// Recommend values are in [SearchWaveformValue] but you can override with catch all value,
    /// but it is **highly recommended you read the documentation first** before deviating from the
    /// standard values.
    #[packed_field(bytes = "1:2", ty = "enum")]
    pub waveform: EnumCatchAll<SearchWaveformValue>,
}

impl SearchWaveform {
    /// Creates a new Search Waveform message
    pub fn new(channel_number: u8, waveform: EnumCatchAll<SearchWaveformValue>) -> Self {
        Self {
            channel_number,
            waveform,
        }
    }
}

/// Represents a Add Channel ID To List message (0x59)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "6")]
pub struct AddChannelIdToList {
    /// Channel list to be modified
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Device number to be added to the list
    #[packed_field(bytes = "1:2")]
    pub device_number: u16,
    /// Device type to be added to the list
    #[packed_field(bytes = "3")]
    pub device_type: DeviceType,
    /// Transmission Type to be added to the list
    #[packed_field(bytes = "4")]
    pub transmission_type: TransmissionType,
    /// List index to be used
    #[packed_field(bytes = "5")]
    pub list_index: u8,
}

impl AddChannelIdToList {
    /// Creates a new Add Channel ID To List message
    pub fn new(
        channel_number: u8,
        device_number: u16,
        device_type: DeviceType,
        transmission_type: TransmissionType,
        list_index: u8,
    ) -> Self {
        Self {
            channel_number,
            device_number,
            device_type,
            transmission_type,
            list_index,
        }
    }
}

/// Represents a Add Encryption ID To List message (0x59)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "6")]
pub struct AddEncryptionIdToList {
    /// Channel list to be modified
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Encryption ID to be added to the list
    #[packed_field(bytes = "1:4")]
    pub encryption_id: [u8; 4],
    /// List index to be modified
    #[packed_field(bytes = "5")]
    pub list_index: u8,
}

impl AddEncryptionIdToList {
    /// Creates a new Add Encryption ID To List message
    pub fn new(channel_number: u8, encryption_id: [u8; 4], list_index: u8) -> Self {
        Self {
            channel_number,
            encryption_id,
            list_index,
        }
    }
}

/// Represents a Config ID List message (0x5A)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct ConfigIdList {
    /// Channel number's list to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// The size of the list
    #[packed_field(bytes = "1")]
    pub list_size: u8,
    /// Exclusion type
    #[packed_field(bytes = "2", ty = "enum")]
    pub exclude: ListExclusion,
}

impl ConfigIdList {
    /// Creates a new Config ID List message
    pub fn new(channel_number: u8, list_size: u8, exclude: ListExclusion) -> Self {
        Self {
            channel_number,
            list_size,
            exclude,
        }
    }
}

/// Represents a Config Encryption ID List message (0x5A)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct ConfigEncryptionIdList {
    /// Channel's number to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// List size
    #[packed_field(bytes = "1")]
    pub list_size: u8,
    /// List exclusion type
    #[packed_field(bytes = "2", ty = "enum")]
    pub list_type: ListType,
}

impl ConfigEncryptionIdList {
    /// Creates a new Config Encryption ID List message
    pub fn new(channel_number: u8, list_size: u8, list_type: ListType) -> Self {
        Self {
            channel_number,
            list_size,
            list_type,
        }
    }
}

/// Represents a Set Channel Transmit Power message (0x60)
///
/// Same as [TransmitPower] but only for a single channel
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct SetChannelTransmitPower {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Power to be used, please refer to docs for value to dbm conversion as it is chip dependent
    #[packed_field(bytes = "1")]
    pub transmit_power: u8,
}

impl SetChannelTransmitPower {
    /// Creates a new Set Channel Transmit Power message
    pub fn new(channel_number: u8, transmit_power: u8) -> Self {
        Self {
            channel_number,
            transmit_power,
        }
    }
}

/// Represents a Low Priority Search Timeout message (0x63)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct LowPrioritySearchTimeout {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Search timeout in counts of 2.5s
    #[packed_field(bytes = "1")]
    pub search_timeout: u8,
}

impl LowPrioritySearchTimeout {
    /// Creates a Low Priority Search Timeout message
    pub fn new(channel_number: u8, search_timeout: u8) -> Self {
        Self {
            channel_number,
            search_timeout,
        }
    }
}

/// Represents a Serial Number Set Channel Id message (0x65)
///
/// This message is not available in softdevice mode
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct SerialNumberSetChannelId {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Device Type to use
    #[packed_field(bytes = "1")]
    pub device_type_id: DeviceType,
    /// Transmission Type to use
    #[packed_field(bytes = "2")]
    pub transmission_type: TransmissionType,
}

impl SerialNumberSetChannelId {
    /// Creates a new Serial Number Set Channel Id message
    pub fn new(
        channel_number: u8,
        device_type_id: DeviceType,
        transmission_type: TransmissionType,
    ) -> Self {
        Self {
            channel_number,
            device_type_id,
            transmission_type,
        }
    }
}

/// Represents a Enable Ext Rx Messages message (0x66)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct EnableExtRxMessages {
    #[packed_field(bits = "0:14")]
    _reserved: ReservedZeroes<packed_bits::Bits15>,
    /// enable extended messages
    #[packed_field(bits = "15")]
    pub enable: bool,
}

impl EnableExtRxMessages {
    /// Creates a new Enable Ext Rx Messages message
    pub fn new(enable: bool) -> Self {
        Self {
            enable,
            ..Self::default()
        }
    }
}

/// Represents an Enable LED message (0x68)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct EnableLed {
    #[packed_field(bits = "0:14")]
    _reserved: ReservedZeroes<packed_bits::Bits15>,
    #[packed_field(bits = "15")]
    /// Switch to enable/disable
    pub enable: bool,
}

impl EnableLed {
    /// Creates a new Enable LED message
    pub fn new(enable: bool) -> Self {
        Self {
            enable,
            ..Self::default()
        }
    }
}

/// Represents a Crystal Enable message (0x6D)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct CrystalEnable {
    #[packed_field(bytes = "0")]
    _reserved: ReservedZeroes<packed_bits::Bits8>,
}

impl CrystalEnable {
    /// Creates a new Crystal Enable message
    pub fn new() -> Self {
        Self { ..Self::default() }
    }
}

/// Represents a Lib Config message (0x6E)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct LibConfig {
    #[packed_field(bytes = "0")]
    _reserved0: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bits = "8")]
    pub enable_channel_id_output: bool,
    #[packed_field(bits = "9")]
    pub enable_rssi_output: bool,
    #[packed_field(bits = "10")]
    pub enable_rx_timestamp_output: bool,
    #[packed_field(bits = "11:15")]
    _reserved1: ReservedZeroes<packed_bits::Bits5>,
}

impl LibConfig {
    pub fn new(
        enable_channel_id_output: bool,
        enable_rssi_output: bool,
        enable_rx_timestamp_output: bool,
    ) -> Self {
        Self {
            enable_channel_id_output,
            enable_rssi_output,
            enable_rx_timestamp_output,
            ..LibConfig::default()
        }
    }
}

/// Represents a Frequency Agility message (0x70)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "4")]
pub struct FrequencyAgility {
    /// Channel number to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// frequency parameter 1
    #[packed_field(bytes = "1")]
    pub frequency_1: u8,
    /// frequency parameter 2
    #[packed_field(bytes = "2")]
    pub frequency_2: u8,
    /// frequency parameter 3
    #[packed_field(bytes = "3")]
    pub frequency_3: u8,
}

impl Default for FrequencyAgility {
    /// Creates a new Frequency Agility message using default values from docs
    fn default() -> Self {
        FrequencyAgility {
            channel_number: 0,
            frequency_1: 3,
            frequency_2: 39,
            frequency_3: 75,
        }
    }
}

impl FrequencyAgility {
    /// Creates a new Frequency Agility message
    pub fn new(channel_number: u8, frequency_1: u8, frequency_2: u8, frequency_3: u8) -> Self {
        Self {
            channel_number,
            frequency_1,
            frequency_2,
            frequency_3,
        }
    }
}

/// Represents a Proximity Search message (0x71)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct ProximitySearch {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Search threshold to use
    #[packed_field(bytes = "1")]
    pub search_threshold: u8,
}

impl ProximitySearch {
    /// Creates a new Proximity Search message
    pub fn new(channel_number: u8, search_threshold: u8) -> Self {
        Self {
            channel_number,
            search_threshold,
        }
    }
}

/// Represents a Configure Event Buffer message (0x74)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "6")]
pub struct ConfigureEventBuffer {
    #[packed_field(bytes = "0")]
    _reserved: ReservedZeroes<packed_bits::Bits8>,
    /// Defines which events to buffer
    #[packed_field(bytes = "1", ty = "enum")]
    pub config: EventBufferConfig,
    /// Maximum number of bytes to buffer
    #[packed_field(bytes = "2:3")]
    pub size: u16,
    /// Maximum time to buffer events in 10ms counts
    #[packed_field(bytes = "4:5")]
    pub time: u16,
}

impl ConfigureEventBuffer {
    /// Creates a new Configure Event Buffer message
    pub fn new(config: EventBufferConfig, size: u16, time: u16) -> Self {
        Self {
            config,
            size,
            time,
            ..Self::default()
        }
    }
}

/// Represents a Channel Search Priority message (0x75)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct ChannelSearchPriority {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Priority used in searching
    #[packed_field(bytes = "1")]
    pub search_priority: u8,
}

impl ChannelSearchPriority {
    /// Creates a new Channel Search Priority message
    pub fn new(channel_number: u8, search_priority: u8) -> Self {
        Self {
            channel_number,
            search_priority,
        }
    }
}

/// Represents a Set 128 Bit Network Key message (0x76)
#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "17")]
pub struct Set128BitNetworkKey {
    /// Network number to be used
    ///
    /// Max value is device dependent
    #[packed_field(bytes = "0")]
    pub network_number: u8,
    /// Network key to be used
    #[packed_field(bytes = "1:16")]
    pub network_key: [u8; 16],
}

impl Set128BitNetworkKey {
    /// Creates a new Set 128-Bit Network Key message
    pub fn new(network_number: u8, network_key: [u8; 16]) -> Self {
        Self {
            network_number,
            network_key,
        }
    }
}

/// Contains the mandatory fields for HighDutySearch
#[derive(PackedStruct, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct HighDutySearchData {
    #[packed_field(bits = "0:14")]
    _reserved: ReservedZeroes<packed_bits::Bits15>,
    /// bool to turn high duty search on and off
    #[packed_field(bits = "15")]
    pub enable: bool,
}

/// Optional fields for HighDutySearch
#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct HighDutySearchSuppressionCycle {
    #[packed_field(bits = "3:7")]
    _reserved: ReservedZeroes<packed_bits::Bits5>,
    /// high priority search suppression in increments of 250ms, limit is 5 and is full
    /// suppression, 0 is no suppression
    #[packed_field(bits = "0:2")]
    suppression_cycle: u8,
}

impl HighDutySearchSuppressionCycle {
    /// Creates a new HighDutySearchSuppressionCycle
    pub fn new(suppression_cycle: u8) -> Self {
        Self {
            suppression_cycle,
            ..Self::default()
        }
    }
}

impl Default for HighDutySearchSuppressionCycle {
    fn default() -> Self {
        Self::new(3)
    }
}

/// Represents a High Duty Search message (0x77)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HighDutySearch {
    /// Required fields
    data: HighDutySearchData,
    /// Optional fields
    suppression_cycle: Option<HighDutySearchSuppressionCycle>,
}
AntAutoPackWithExtention!(
    HighDutySearch,
    TxMessageId::HighDutySearch,
    data,
    suppression_cycle
);

impl HighDutySearch {
    /// Creates a new High Duty Search message
    pub fn new(enable: bool, suppression_cycle: Option<HighDutySearchSuppressionCycle>) -> Self {
        Self {
            data: HighDutySearchData {
                enable,
                ..HighDutySearchData::default()
            },
            suppression_cycle,
        }
    }
}

/// Represents Configure Advanced Burst required fields
#[derive(PackedStruct, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "9")]
pub struct ConfigureAdvancedBurstData {
    #[packed_field(bits = "0:14")]
    _reserved: ReservedZeroes<packed_bits::Bits15>,
    /// enable/disable advanced burst
    #[packed_field(bits = "15")]
    pub enable: bool,
    /// Maximum size of an advanced burst message
    #[packed_field(bytes = "2", ty = "enum")]
    pub max_packet_length: AdvancedBurstMaxPacketLength,
    /// Field to specify required features
    #[packed_field(bytes = "3:5")]
    pub required_features: SupportedFeatures,
    /// Field to specify optional features
    #[packed_field(bytes = "6:8")]
    pub optional_features: SupportedFeatures,
}

/// Represents a Configure Advanced Burst message (0x78)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConfigureAdvancedBurst {
    /// Required Fields
    pub data: ConfigureAdvancedBurstData,
    /// Optional stall count fields
    pub stall_count: Option<Integer<u16, packed_bits::Bits16>>,
    /// Optional retry count fields
    ///
    /// Note, to use retry count, you must also use stall count
    pub retry_count_extension: Option<Integer<u8, packed_bits::Bits8>>,
}

const CONFIGURE_ADVANCED_BURST_DATA_SIZE: usize = 9;

impl AntTxMessageType for ConfigureAdvancedBurst {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
        let mut len = CONFIGURE_ADVANCED_BURST_DATA_SIZE;
        self.data
            .pack_to_slice(&mut buf[..CONFIGURE_ADVANCED_BURST_DATA_SIZE])?;
        if let Some(data) = self.stall_count {
            buf[CONFIGURE_ADVANCED_BURST_DATA_SIZE..CONFIGURE_ADVANCED_BURST_DATA_SIZE + 2]
                .copy_from_slice(data.to_lsb_bytes()?.as_slice());
            len += 2;
            if let Some(retry_count) = self.retry_count_extension {
                buf[len..len + 1].copy_from_slice(retry_count.to_lsb_bytes()?.as_slice());
                len += 1;
            }
        } else if self.stall_count.is_none() && self.retry_count_extension.is_some() {
            return Err(PackingError::InvalidValue);
        }
        Ok(len)
    }

    fn get_tx_msg_id(&self) -> TxMessageId {
        TxMessageId::ConfigureAdvancedBurst
    }
}

impl ConfigureAdvancedBurst {
    /// Creates a new Configure Advanced Burst message
    pub fn new(
        enable: bool,
        max_packet_length: AdvancedBurstMaxPacketLength,
        required_features: SupportedFeatures,
        optional_features: SupportedFeatures,
        stall_count: Option<Integer<u16, packed_bits::Bits16>>,
        retry_count_extension: Option<Integer<u8, packed_bits::Bits8>>,
    ) -> Self {
        Self {
            data: ConfigureAdvancedBurstData {
                enable,
                max_packet_length,
                required_features,
                optional_features,
                ..ConfigureAdvancedBurstData::default()
            },
            stall_count,
            retry_count_extension,
        }
    }

    pub(crate) fn unpack_from_slice<R, W>(
        buf: &[u8],
    ) -> Result<ConfigureAdvancedBurst, DriverError<R, W>> {
        let data = ConfigureAdvancedBurstData::unpack_from_slice(
            &buf[..CONFIGURE_ADVANCED_BURST_DATA_SIZE],
        )?;
        let buf = &buf[CONFIGURE_ADVANCED_BURST_DATA_SIZE..];

        let mut msg = ConfigureAdvancedBurst {
            data,
            stall_count: None,
            retry_count_extension: None,
        };

        if buf.is_empty() {
            return Ok(msg);
        }

        if buf.len() < 2 {
            return Err(DriverError::BadLength(buf.len(), 2));
        }

        msg.stall_count = Some(Integer::<u16, packed_bits::Bits16>::from_lsb_bytes(
            buf[..2].try_into()?,
        )?);
        let buf = &buf[2..];

        if buf.is_empty() {
            return Ok(msg);
        }

        if buf.len() != 1 {
            return Err(DriverError::BadLength(buf.len(), 1));
        }

        msg.retry_count_extension = Some(Integer::<u8, packed_bits::Bits8>::from_lsb_bytes(
            buf[..1].try_into()?,
        )?);

        Ok(msg)
    }
}

/// Represents a Configure Event Filter message (0x79)
#[derive(PackedStruct, AntTx, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct ConfigureEventFilter {
    #[packed_field(bytes = "0")]
    _reserved0: ReservedZeroes<packed_bits::Bits8>,
    /// filter out rx search time out events
    #[packed_field(bits = "15")]
    pub filter_event_rx_search_timeout: bool,
    /// filter out rx search fail events
    #[packed_field(bits = "14")]
    pub filter_event_rx_fail: bool,
    /// filter out tx events
    #[packed_field(bits = "13")]
    pub filter_event_tx: bool,
    /// filter out event transfers rx failed
    #[packed_field(bits = "12")]
    pub filter_event_transfer_rx_failed: bool,
    /// filter out event transfers tx completed
    #[packed_field(bits = "11")]
    pub filter_event_transfer_tx_completed: bool,
    /// filter out event transfers tx failed
    #[packed_field(bits = "10")]
    pub filter_event_transfer_tx_failed: bool,
    /// filter out event channel closed
    #[packed_field(bits = "9")]
    pub filter_event_channel_closed: bool,
    /// filter out event rx fail go to search
    #[packed_field(bits = "8")]
    pub filter_event_rx_fail_go_to_search: bool,
    /// filter out event channel collision
    #[packed_field(bits = "23")]
    pub filter_event_channel_collision: bool,
    /// filter out event transfer tx start
    #[packed_field(bits = "22")]
    pub filter_event_transfer_tx_start: bool,
    #[packed_field(bits = "16:21")]
    _reserved1: ReservedZeroes<packed_bits::Bits8>,
}

impl ConfigureEventFilter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        filter_event_rx_search_timeout: bool,
        filter_event_rx_fail: bool,
        filter_event_tx: bool,
        filter_event_transfer_rx_failed: bool,
        filter_event_transfer_tx_completed: bool,
        filter_event_transfer_tx_failed: bool,
        filter_event_channel_closed: bool,
        filter_event_rx_fail_go_to_search: bool,
        filter_event_channel_collision: bool,
        filter_event_transfer_tx_start: bool,
    ) -> Self {
        Self {
            filter_event_rx_search_timeout,
            filter_event_rx_fail,
            filter_event_tx,
            filter_event_transfer_rx_failed,
            filter_event_transfer_tx_completed,
            filter_event_transfer_tx_failed,
            filter_event_channel_closed,
            filter_event_rx_fail_go_to_search,
            filter_event_channel_collision,
            filter_event_transfer_tx_start,
            ..Self::default()
        }
    }
}

/// Represents a Configure Selective Data Updates message (0x7A)
#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct ConfigureSelectiveDataUpdates {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    #[packed_field(bytes = "1")]
    pub selected_data: u8,
    // TODO figure out this field and write a test, it has reserved bits that are changeable when
    // invalidating a config
}
// TODO test

impl ConfigureSelectiveDataUpdates {
    pub fn new(channel_number: u8, selected_data: u8) -> Self {
        Self {
            channel_number,
            selected_data,
        }
    }
}

/// Represents a Set Selective Data Update Mask message (0x7B)
#[derive(PackedStruct, AntTx, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "9")]
pub struct SetSelectiveDataUpdateMask {
    /// Mask to updated
    ///
    /// Must be in range [0..MAX_SDU_MASKS-1]
    #[packed_field(bytes = "0")]
    pub sdu_mask_number: u8,
    /// Mask to be set for 8-byte messages
    ///
    /// Bit meanings
    /// 0 - ignore
    /// 1 - compare and send when changed
    #[packed_field(bytes = "1:8")]
    pub sdu_mask: [u8; 8],
}

impl SetSelectiveDataUpdateMask {
    pub fn new(sdu_mask_number: u8, sdu_mask: [u8; 8]) -> Self {
        Self {
            sdu_mask_number,
            sdu_mask,
        }
    }
}

// TODO configure user nvme message

/// Represents a Enable Single Channel Encryption message (0x7D)
#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "4")]
pub struct EnableSingleChannelEncryption {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Encryption mode to be used
    #[packed_field(bytes = "1", ty = "enum")]
    pub encryption_mode: EncryptionMode,
    /// Per version 5.1 of the spec this field has a range of 0
    #[packed_field(bytes = "2")]
    pub volatile_key_index: ReservedZeroes<packed_bits::Bits8>,
    /// Master channel rate / slave tracking channel rate
    #[packed_field(bytes = "3")]
    pub decimation_rate: u8,
}

impl EnableSingleChannelEncryption {
    pub fn new(channel_number: u8, encryption_mode: EncryptionMode, decimation_rate: u8) -> Self {
        Self {
            channel_number,
            encryption_mode,
            decimation_rate,
            ..Self::default()
        }
    }
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "17")]
pub struct SetEncryptionKey {
    // Per version 5.1 of the spec this field has a range of 0
    #[packed_field(bytes = "0")]
    pub volatile_key_index: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bytes = "1:16")]
    pub encryption_key: [u8; 16],
}

impl SetEncryptionKey {
    pub fn new(encryption_key: [u8; 16]) -> Self {
        Self {
            encryption_key,
            ..Self::default()
        }
    }
}

// The spec defines this as a single variable message but variable types are
// basically impossible with the packed_stuct lib so it is easier to just
// implement 3 message types to handle all the cases.
#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "5")]
pub struct SetEncryptionInfoEncryptionId {
    // 0 for encryption id
    #[packed_field(bytes = "0")]
    pub set_parameter: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bytes = "1:4")]
    pub encryption_id: EncryptionId,
}

impl SetEncryptionInfoEncryptionId {
    pub fn new(encryption_id: EncryptionId) -> Self {
        Self {
            encryption_id,
            ..Self::default()
        }
    }
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "20")]
pub struct SetEncryptionInfoUserInformationString {
    // 1 for User Information String
    #[packed_field(bits = "0:6")]
    pub set_parameter0: ReservedZeroes<packed_bits::Bits7>,
    #[packed_field(bits = "7")]
    pub set_parameter1: ReservedOnes<packed_bits::Bits1>,
    #[packed_field(bytes = "1:19")]
    pub user_information_string: UserInformationString,
}

impl SetEncryptionInfoUserInformationString {
    pub fn new(user_information_string: UserInformationString) -> Self {
        Self {
            user_information_string,
            ..Self::default()
        }
    }
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "17")]
pub struct SetEncryptionInfoRandomSeed {
    // 2 for Random Number Seed
    #[packed_field(bits = "0:5")]
    pub set_parameter0: ReservedZeroes<packed_bits::Bits6>,
    #[packed_field(bits = "6")]
    pub set_parameter1: ReservedOnes<packed_bits::Bits1>,
    #[packed_field(bits = "7")]
    pub set_parameter2: ReservedZeroes<packed_bits::Bits1>,
    #[packed_field(bytes = "1:16")]
    pub random_seed: [u8; 16],
}

impl SetEncryptionInfoRandomSeed {
    pub fn new(random_seed: [u8; 16]) -> Self {
        Self {
            random_seed,
            ..Self::default()
        }
    }
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct ChannelSearchSharing {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    #[packed_field(bytes = "1")]
    pub search_sharing_cycles: u8,
}

impl ChannelSearchSharing {
    pub fn new(channel_number: u8, search_sharing_cycles: u8) -> Self {
        Self {
            channel_number,
            search_sharing_cycles,
        }
    }
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct LoadEncryptionKeyFromNvm {
    #[packed_field(bytes = "0")]
    pub operation: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bytes = "1")]
    pub nvm_key_index: u8,
    // 0 per spec v5.1
    #[packed_field(bytes = "2")]
    volatile_key_index: ReservedZeroes<packed_bits::Bits8>,
}

impl LoadEncryptionKeyFromNvm {
    pub fn new(nvm_key_index: u8) -> Self {
        Self {
            nvm_key_index,
            ..Self::default()
        }
    }
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "18")]
pub struct StoreEncryptionKeyInNvm {
    #[packed_field(bits = "0:6")]
    pub operation0: ReservedZeroes<packed_bits::Bits7>,
    #[packed_field(bits = "7")]
    pub operation1: ReservedOnes<packed_bits::Bits1>,
    #[packed_field(bytes = "1")]
    pub nvm_key_index: u8,
    #[packed_field(bytes = "2:17")]
    pub encryption_key: [u8; 16],
}

impl StoreEncryptionKeyInNvm {
    pub fn new(nvm_key_index: u8, encryption_key: [u8; 16]) -> Self {
        Self {
            nvm_key_index,
            encryption_key,
            ..Self::default()
        }
    }
}

// TODO SetUsbDescriptorString

#[derive(PackedStruct, Debug, Clone, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", endian = "lsb", size_bytes = "1")]
pub struct StartUpMessage {
    #[packed_field(bits = "0")]
    pub hardware_reset_line: bool,
    #[packed_field(bits = "1")]
    pub watch_dog_reset: bool,
    #[packed_field(bits = "5")]
    pub command_reset: bool,
    #[packed_field(bits = "6")]
    pub synchronous_reset: bool,
    #[packed_field(bits = "7")]
    pub suspend_reset: bool,
}

impl StartUpMessage {
    /// Helper function to detect special bitfield case of power on reset cause
    // TODO test
    pub fn is_power_on_reset(&self) -> bool {
        !(self.hardware_reset_line
            || self.watch_dog_reset
            || self.command_reset
            || self.synchronous_reset
            || self.suspend_reset)
    }
}

// TODO spec says rest of data contains a copy of the error message, need to validate how this
// works on the usb in the field
// Note this message has a range up to 255
// TODO make a config so users can set TX and RX buffer sizes for embeded devices since only
// users of the USB devices need the full 256 bytes for NVMe
#[derive(PackedStruct, Debug, Clone, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct SerialErrorMessage {
    #[packed_field(bytes = "0", ty = "enum")]
    pub error_number: SerialErrorType,
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct ResetSystem {
    #[packed_field(bytes = "0")]
    filler: ReservedZeroes<packed_bits::Bits8>,
}

impl ResetSystem {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct OpenChannel {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
}

impl OpenChannel {
    pub fn new(channel_number: u8) -> Self {
        Self { channel_number }
    }
}

#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct CloseChannel {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
}

impl CloseChannel {
    pub fn new(channel_number: u8) -> Self {
        Self { channel_number }
    }
}

#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct RequestMessageData {
    #[packed_field(bytes = "0")]
    pub channel: u8,
    #[packed_field(bytes = "1", ty = "enum")]
    pub message_id: RequestableMessageId,
}

// TODO test
#[derive(PackedStruct, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct NvmeRequest {
    #[packed_field(bytes = "0:1")]
    pub addr: u16,
    #[packed_field(bytes = "2")]
    pub size: u8,
}

impl NvmeRequest {
    pub fn new(addr: u16, size: u8) -> Self {
        Self { addr, size }
    }
}

// TODO test
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestMessage {
    pub data: RequestMessageData,
    pub nvme_region: Option<NvmeRequest>,
}
AntAutoPackWithExtention!(
    RequestMessage,
    TxMessageId::RequestMessage,
    data,
    nvme_region
);

impl RequestMessage {
    pub fn new(
        channel: u8,
        message_id: RequestableMessageId,
        nvme_region: Option<NvmeRequest>,
    ) -> Self {
        Self {
            data: RequestMessageData {
                channel,
                message_id,
            },
            nvme_region,
        }
    }
}

// TODO implement serialize and test
// TODO implement new handler
pub struct OpenRxScanMode {
    pub synchronous_channel_packets_only: Option<bool>,
}

#[derive(PackedStruct, AntTx, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct SleepMessage {
    #[packed_field(bytes = "0")]
    filler: ReservedZeroes<packed_bits::Bits8>,
}

impl SleepMessage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(PackedStruct, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "9")]
pub struct BroadcastDataPayload {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    #[packed_field(bytes = "1:8")]
    pub data: [u8; 8],
}

// TODO test TX
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BroadcastData {
    pub payload: BroadcastDataPayload,
    pub extended_info: Option<ExtendedInfo>,
}

impl AntTxMessageType for BroadcastData {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
        // Data payloads have optional RX fields but are ignored on TX
        self.payload
            .pack_to_slice(&mut buf[..BROADCAST_PAYLOAD_SIZE])?;
        Ok(BROADCAST_PAYLOAD_SIZE)
    }
    fn get_tx_msg_id(&self) -> TxMessageId {
        TxMessageId::BroadcastData
    }
}

const BROADCAST_PAYLOAD_SIZE: usize = 9;

impl BroadcastData {
    pub fn new(channel_number: u8, data: [u8; 8]) -> Self {
        Self {
            payload: BroadcastDataPayload {
                channel_number,
                data,
            },
            extended_info: None,
        }
    }

    pub(crate) fn unpack_from_slice<R, W>(data: &[u8]) -> Result<BroadcastData, DriverError<R, W>> {
        Ok(BroadcastData {
            payload: BroadcastDataPayload::unpack_from_slice(&data[..BROADCAST_PAYLOAD_SIZE])?,
            extended_info: ExtendedInfo::unpack_from_slice(&data[BROADCAST_PAYLOAD_SIZE..])?,
        })
    }
}

// Same byte payload, just different name
pub type AcknowledgedDataPayload = BroadcastDataPayload;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AcknowledgedData {
    pub payload: AcknowledgedDataPayload,
    pub extended_info: Option<ExtendedInfo>,
}

// TODO test TX
impl AntTxMessageType for AcknowledgedData {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
        // Data payloads have optional RX fields but are ignored on TX
        self.payload
            .pack_to_slice(&mut buf[..BROADCAST_PAYLOAD_SIZE])?;
        Ok(BROADCAST_PAYLOAD_SIZE)
    }
    fn get_tx_msg_id(&self) -> TxMessageId {
        TxMessageId::AcknowledgedData
    }
}

impl AcknowledgedData {
    pub fn new(channel_number: u8, data: [u8; 8]) -> Self {
        Self {
            payload: BroadcastDataPayload {
                channel_number,
                data,
            },
            extended_info: None,
        }
    }

    pub(crate) fn unpack_from_slice<R, W>(
        data: &[u8],
    ) -> Result<AcknowledgedData, DriverError<R, W>> {
        Ok(AcknowledgedData {
            payload: AcknowledgedDataPayload::unpack_from_slice(&data[..BROADCAST_PAYLOAD_SIZE])?,
            extended_info: ExtendedInfo::unpack_from_slice(&data[BROADCAST_PAYLOAD_SIZE..])?,
        })
    }
}

#[derive(PackedStruct, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "9")]
pub struct BurstTransferDataPayload {
    #[packed_field(bytes = "0")]
    pub channel_sequence: ChannelSequence,
    #[packed_field(bytes = "1:8")]
    pub data: [u8; 8],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BurstTransferData {
    pub payload: BurstTransferDataPayload,
    pub extended_info: Option<ExtendedInfo>,
}

// TODO test TX
impl AntTxMessageType for BurstTransferData {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
        // Data payloads have optional RX fields but are ignored on TX
        self.payload
            .pack_to_slice(&mut buf[..BURSTTRANSFER_PAYLOAD_SIZE])?;
        Ok(BURSTTRANSFER_PAYLOAD_SIZE)
    }
    fn get_tx_msg_id(&self) -> TxMessageId {
        TxMessageId::BurstTransferData
    }
}

const BURSTTRANSFER_PAYLOAD_SIZE: usize = 9;

impl BurstTransferData {
    pub fn new(channel_sequence: ChannelSequence, data: [u8; 8]) -> Self {
        Self {
            payload: BurstTransferDataPayload {
                channel_sequence,
                data,
            },
            extended_info: None,
        }
    }

    pub(crate) fn unpack_from_slice<R, W>(
        data: &[u8],
    ) -> Result<BurstTransferData, DriverError<R, W>> {
        Ok(BurstTransferData {
            payload: BurstTransferDataPayload::unpack_from_slice(
                &data[..BURSTTRANSFER_PAYLOAD_SIZE],
            )?,
            extended_info: ExtendedInfo::unpack_from_slice(&data[BURSTTRANSFER_PAYLOAD_SIZE..])?,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AdvancedBurstData {
    pub channel_sequence: ChannelSequence,
    pub data: ArrayVec<u8, ADVANCED_BURST_BUFFER_SIZE>,
}

impl AdvancedBurstData {
    pub fn new(
        channel_sequence: ChannelSequence,
        data: ArrayVec<u8, ADVANCED_BURST_BUFFER_SIZE>,
    ) -> Self {
        Self {
            channel_sequence,
            data,
        }
    }

    pub(crate) fn unpack_from_slice<R, W>(data: &[u8]) -> Result<Self, DriverError<R, W>> {
        Ok(AdvancedBurstData {
            channel_sequence: ChannelSequence::unpack_from_slice(&data[..1])?,
            data: data[1..].try_into()?,
        })
    }
}

impl AntTxMessageType for AdvancedBurstData {
    // TODO test
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
        let sequence_size = ChannelSequence::packed_bytes_size(None)?;
        let len = sequence_size + self.data.len();

        self.channel_sequence
            .pack_to_slice(&mut buf[..sequence_size])?;
        buf[sequence_size..sequence_size + self.data.len()].copy_from_slice(&self.data);
        Ok(len)
    }
    fn get_tx_msg_id(&self) -> TxMessageId {
        TxMessageId::AdvancedBurstData
    }
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "3")]
pub struct ChannelEventPayload {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    #[packed_field(bits = "8:14")]
    _reserved0: ReservedZeroes<packed_bits::Bits7>,
    #[packed_field(bits = "15")]
    _reserved1: ReservedOnes<packed_bits::Bits1>,
    #[packed_field(bytes = "2", ty = "enum")]
    pub message_code: MessageCode,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChannelEvent {
    pub payload: ChannelEventPayload,
    pub extended_info: Option<ChannelEventExtension>,
}
// TODO test
impl ChannelEvent {
    pub(crate) fn unpack_from_slice<R, W>(data: &[u8]) -> Result<Self, DriverError<R, W>> {
        let payload = ChannelEventPayload::unpack_from_slice(data)?;

        Ok(ChannelEvent {
            payload,
            // TODO extended_info,
            extended_info: None,
        })
    }
}

#[derive(PackedStruct, Debug, Clone, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct ChannelResponse {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    #[packed_field(bytes = "1", ty = "enum")]
    pub message_id: TxMessageId,
    #[packed_field(bytes = "2", ty = "enum")]
    pub message_code: MessageCode,
}

#[derive(PackedStruct, Debug, Clone, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct ChannelStatus {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    #[packed_field(bits = "8:11", ty = "enum")]
    pub channel_type: ChannelType,
    #[packed_field(bits = "12:13")]
    pub network_number: u8,
    #[packed_field(bits = "14:15", ty = "enum")]
    pub channel_state: ChannelState,
}

// TODO test
#[derive(Clone, Debug, PartialEq)]
pub struct AntVersion {
    version: ArrayVec<u8, MAX_MESSAGE_DATA_SIZE>,
}

impl AntVersion {
    pub(crate) fn unpack_from_slice<R, W>(data: &[u8]) -> Result<Self, DriverError<R, W>> {
        Ok(Self {
            version: data.try_into()?,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Capabilities {
    pub base_capabilities: BaseCapabilities,
    pub advanced_options2: Option<AdvancedOptions2>,
    pub max_sensrcore_channels: Option<u8>,
    pub advanced_options3: Option<AdvancedOptions3>,
    pub advanced_options4: Option<AdvancedOptions4>,
}

const BASECAPABILITIES_SIZE: usize = 4;
const ADVANCEDOPTIONS2_SIZE: usize = 1;
const MAX_SENSRCORE_CHANNELS_SIZE: usize = 1;
const ADVANCEDOPTIONS3_SIZE: usize = 1;
const ADVANCEDOPTIONS4_SIZE: usize = 1;

// TODO test
impl Capabilities {
    pub(crate) fn unpack_from_slice<R, W>(data: &[u8]) -> Result<Self, DriverError<R, W>> {
        let base_capabilities =
            BaseCapabilities::unpack_from_slice(&data[..BASECAPABILITIES_SIZE])?;
        let data = &data[BASECAPABILITIES_SIZE..];

        if data.is_empty() {
            return Ok(Capabilities {
                base_capabilities,
                advanced_options2: None,
                max_sensrcore_channels: None,
                advanced_options3: None,
                advanced_options4: None,
            });
        }

        let advanced_options2 =
            AdvancedOptions2::unpack_from_slice(&data[..ADVANCEDOPTIONS2_SIZE])?;
        let data = &data[ADVANCEDOPTIONS2_SIZE..];

        if data.is_empty() {
            return Ok(Capabilities {
                base_capabilities,
                advanced_options2: Some(advanced_options2),
                max_sensrcore_channels: None,
                advanced_options3: None,
                advanced_options4: None,
            });
        }

        let max_sensrcore_channels = data[0];
        let data = &data[MAX_SENSRCORE_CHANNELS_SIZE..];

        if data.is_empty() {
            return Ok(Capabilities {
                base_capabilities,
                advanced_options2: Some(advanced_options2),
                max_sensrcore_channels: Some(max_sensrcore_channels),
                advanced_options3: None,
                advanced_options4: None,
            });
        }

        let advanced_options3 =
            AdvancedOptions3::unpack_from_slice(&data[..ADVANCEDOPTIONS3_SIZE])?;
        let data = &data[ADVANCEDOPTIONS3_SIZE..];

        if data.is_empty() {
            return Ok(Capabilities {
                base_capabilities,
                advanced_options2: Some(advanced_options2),
                max_sensrcore_channels: Some(max_sensrcore_channels),
                advanced_options3: Some(advanced_options3),
                advanced_options4: None,
            });
        }

        let advanced_options4 =
            AdvancedOptions4::unpack_from_slice(&data[..ADVANCEDOPTIONS4_SIZE])?;
        let data = &data[ADVANCEDOPTIONS4_SIZE..];

        if data.is_empty() {
            return Ok(Capabilities {
                base_capabilities,
                advanced_options2: Some(advanced_options2),
                max_sensrcore_channels: Some(max_sensrcore_channels),
                advanced_options3: Some(advanced_options3),
                advanced_options4: Some(advanced_options4),
            });
        }

        let expected_size = BASECAPABILITIES_SIZE
            + ADVANCEDOPTIONS2_SIZE
            + MAX_SENSRCORE_CHANNELS_SIZE
            + ADVANCEDOPTIONS3_SIZE
            + ADVANCEDOPTIONS4_SIZE;
        Err(DriverError::BadLength(
            expected_size + data.len(),
            expected_size,
        ))
    }
}

// TODO test
#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "5")]
pub struct AdvancedBurstCapabilities {
    #[packed_field(bytes = "0")]
    _reserved: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bytes = "1", ty = "enum")]
    pub supported_max_packed_length: AdvancedBurstMaxPacketLength,
    #[packed_field(bytes = "2:4")]
    pub supported_features: SupportedFeatures,
}

#[derive(PackedStruct, Debug, Clone, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "4")]
pub struct SerialNumber {
    #[packed_field(bytes = "0:3")]
    serial_number: [u8; 4],
}

// Reexport under new name even though its the same type to match the docs
// Reserved fields are ignored so any mismatch in fixed fields is ignored on parsing
pub use ConfigureAdvancedBurst as AdvancedBurstCurrentConfiguration;
pub use ConfigureEventBuffer as EventBufferConfiguration;
pub use ConfigureEventFilter as EventFilter;
pub use SetSelectiveDataUpdateMask as SelectiveDataUpdateMaskSetting;

#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct UserNvmHeader {
    #[packed_field(bytes = "0")]
    resered: ReservedZeroes<packed_bits::Bits8>,
}

// TODO conditionally compile this, also magic num
#[derive(Clone, Debug, PartialEq)]
pub struct UserNvm {
    header: UserNvmHeader,
    data: ArrayVec<u8, 255>,
}

impl UserNvm {
    pub(crate) fn unpack_from_slice<R, W>(data: &[u8]) -> Result<UserNvm, DriverError<R, W>> {
        Ok(UserNvm {
            header: UserNvmHeader::unpack_from_slice(&data[0..1])?,
            data: data[1..].try_into()?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EncryptionModeParameters {
    pub requested_encryption_parameter: RequestedEncryptionParameter,
    pub requested_encryption_parameter_data: RequestedEncryptionParameterData,
}

impl EncryptionModeParameters {
    pub(crate) fn unpack_from_slice<R, W>(
        data: &[u8],
    ) -> Result<EncryptionModeParameters, DriverError<R, W>> {
        if data.is_empty() {
            return Err(DriverError::BadLength(0, 1));
        }
        let parameter = RequestedEncryptionParameter::from_primitive(data[0])
            .ok_or(DriverError::InvalidData())?;
        // TODO magic num
        let data = &data[1..];
        let data = match parameter {
            RequestedEncryptionParameter::MaxSupportedEncryptionMode => {
                if data.len() != 1 {
                    return Err(DriverError::BadLength(data.len(), 1));
                }
                let param = match EncryptionMode::from_primitive(data[0]) {
                    Some(x) => x,
                    None => return Err(DriverError::InvalidData()),
                };
                RequestedEncryptionParameterData::MaxSupportedEncryptionMode(param)
            }
            RequestedEncryptionParameter::EncryptionId => {
                RequestedEncryptionParameterData::EncryptionId(EncryptionId::try_from(data)?)
            }
            RequestedEncryptionParameter::UserInformationString => {
                RequestedEncryptionParameterData::UserInformationString(
                    UserInformationString::try_from(data)?,
                )
            }
        };
        Ok(EncryptionModeParameters {
            requested_encryption_parameter: parameter,
            requested_encryption_parameter_data: data,
        })
    }
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct CwInit {
    #[packed_field(bytes = "0")]
    filler: ReservedZeroes<packed_bits::Bits8>,
}

impl CwInit {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(PackedStruct, AntTx, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct CwTest {
    #[packed_field(bytes = "0")]
    filler: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bytes = "1")]
    pub transmit_power: u8,
    #[packed_field(bytes = "2")]
    pub channel_rf_frequency: u8,
}

impl CwTest {
    pub fn new(transmit_power: u8, channel_rf_frequency: u8) -> Self {
        Self {
            transmit_power,
            channel_rf_frequency,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inner::inner;
    use packed_struct::PackingError;

    #[derive(Debug)]
    enum SerialError {}

    #[test]
    fn unassign_channel() {
        let packed = UnAssignChannel::new(5);
        assert_eq!(packed.pack().unwrap(), [5]);
    }

    #[test]
    fn assign_channel() {
        let mut buf: [u8; 5] = [0; 5];
        let packed = AssignChannel::new(1, ChannelType::SharedBidirectionalSlave, 3, None);
        assert_eq!(packed.serialize_message(&mut buf).unwrap(), 3);
        assert_eq!(buf, [1, 0x20, 3, 0, 0]);
        buf = [0; 5];
        let mut ext = ExtendedAssignment::default();
        ext.always_search = true;
        ext.fast_initiation_mode = true;
        let packed = AssignChannel {
            data: AssignChannelData {
                channel_number: 1,
                channel_type: ChannelType::SharedBidirectionalSlave,
                network_number: 3,
                ..AssignChannelData::default()
            },
            extended_assignment: Some(ext),
        };
        assert_eq!(packed.serialize_message(&mut buf).unwrap(), 4);
        assert_eq!(buf, [1, 0x20, 3, 0x11, 0]);
    }

    #[test]
    fn channel_id() {
        let mut packed = ChannelId::new(
            2,
            0xABCD,
            DeviceType::new(3.into(), false),
            TransmissionType::default(),
        );
        assert_eq!(packed.pack().unwrap(), [2, 0xCD, 0xAB, 3, 1]);

        packed.wildcard();
        assert_eq!(packed.pack().unwrap(), [2, 0, 0, 0, 0]);

        let unpacked = ChannelId::unpack(&[1, 0x36, 0x47, 0x85, 5]).unwrap();
        assert_eq!(unpacked.channel_number, 1);
        assert_eq!(unpacked.device_number, 0x4736);
        assert_eq!(unpacked.device_type, DeviceType::new(5.into(), true));
        assert_eq!(
            unpacked.transmission_type,
            TransmissionType::new(
                TransmissionChannelType::IndependentChannel,
                TransmissionGlobalDataPages::GlobalDataPagesUsed,
                0.into(),
            )
        );
    }

    #[test]
    fn channel_period() {
        let packed = ChannelPeriod::new(1, 0xABCD);
        assert_eq!(packed.pack().unwrap(), [1, 0xCD, 0xAB]);
    }

    #[test]
    fn search_timeout() {
        let packed = SearchTimeout::new(1, 0xA);
        assert_eq!(packed.pack().unwrap(), [1, 0xA]);
    }

    #[test]
    fn channel_rf_frequency() {
        let packed = ChannelRfFrequency::new(1, 0xA);
        assert_eq!(packed.pack().unwrap(), [1, 0xA]);
    }

    #[test]
    fn set_network_key() {
        let packed = SetNetworkKey::new(1, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]);
        assert_eq!(
            packed.pack().unwrap(),
            [1, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]
        );
    }

    #[test]
    fn transmit_power() {
        let packed = TransmitPower::new(0x55);
        assert_eq!(packed.pack().unwrap(), [0, 0x55]);
    }

    #[test]
    fn search_waveform() {
        let packed = SearchWaveform::new(1, EnumCatchAll::Enum(SearchWaveformValue::Fast));
        assert_eq!(packed.pack().unwrap(), [1, 97, 0x00]);

        let packed = SearchWaveform::new(1, EnumCatchAll::CatchAll(0xFFAA));
        assert_eq!(packed.pack().unwrap(), [1, 0xAA, 0xFF]);
    }

    #[test]
    fn add_channel_id_to_list() {
        let packed = AddChannelIdToList::new(
            3,
            0xABCD,
            DeviceType::new(0x7F.into(), false),
            TransmissionType::default(),
            2,
        );
        assert_eq!(packed.pack().unwrap(), [3, 0xCD, 0xAB, 0x7F, 1, 2]);
    }

    #[test]
    fn add_encryption_id_to_list() {
        let packed = AddEncryptionIdToList::new(3, [0xAA, 0xBB, 0xCC, 0xDD], 2);
        assert_eq!(packed.pack().unwrap(), [3, 0xAA, 0xBB, 0xCC, 0xDD, 2]);
    }

    #[test]
    fn config_id_list() {
        let packed = ConfigIdList::new(1, 2, ListExclusion::Include);
        assert_eq!(packed.pack().unwrap(), [1, 2, 0]);
    }

    #[test]
    fn config_encryption_id_list() {
        let packed = ConfigEncryptionIdList::new(1, 2, ListType::Whitelist);
        assert_eq!(packed.pack().unwrap(), [1, 2, 0]);
    }

    #[test]
    fn set_channel_transmit_power() {
        let packed = SetChannelTransmitPower::new(1, 2);
        assert_eq!(packed.pack().unwrap(), [1, 2]);
    }

    #[test]
    fn low_priority_search_timeout() {
        let packed = LowPrioritySearchTimeout::new(1, 2);
        assert_eq!(packed.pack().unwrap(), [1, 2]);
    }

    #[test]
    fn serial_number_set_channel_id() {
        let packed =
            SerialNumberSetChannelId::new(2, DeviceType::default(), TransmissionType::default());
        assert_eq!(packed.pack().unwrap(), [2, 0, 1]);
    }

    #[test]
    fn enable_ext_rx_messages() {
        let packed = EnableExtRxMessages::new(true);
        assert_eq!(packed.pack().unwrap(), [0, 1]);
    }

    #[test]
    fn enable_led() {
        let packed = EnableLed::new(false);
        assert_eq!(packed.pack().unwrap(), [0, 0]);
    }

    #[test]
    fn crystal_enable() {
        let packed = CrystalEnable::new();
        assert_eq!(packed.pack().unwrap(), [0]);
    }

    #[test]
    fn lib_config() {
        let packed = LibConfig::new(true, false, true);
        assert_eq!(packed.pack().unwrap(), [0, 0xA0]);
    }

    #[test]
    fn frequency_agility() {
        let packed = FrequencyAgility::new(0, 2, 4, 8);
        assert_eq!(packed.pack().unwrap(), [0, 2, 4, 8]);
    }

    #[test]
    fn proximity_search() {
        let packed = ProximitySearch::new(0, 8);
        assert_eq!(packed.pack().unwrap(), [0, 8]);
    }

    #[test]
    fn configure_event_buffer() {
        let packed = ConfigureEventBuffer::new(EventBufferConfig::BufferAllEvents, 2, 8);
        assert_eq!(packed.pack().unwrap(), [0, 1, 2, 0, 8, 0]);
    }

    #[test]
    fn channel_search_priority() {
        let packed = ChannelSearchPriority::new(0, 8);
        assert_eq!(packed.pack().unwrap(), [0, 8]);
    }

    #[test]
    fn set_128_bit_network_key() {
        let packed = Set128BitNetworkKey::new(
            0,
            [
                0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10,
            ],
        );
        assert_eq!(
            packed.pack().unwrap(),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
    }

    #[test]
    fn high_duty_search() {
        let packed = HighDutySearch::new(true, None);
        assert_eq!(packed.data.pack().unwrap(), [0, 1]);
        // TODO test with suppression cycle
    }

    #[test]
    fn configure_advanced_burst() {
        let mut buf: [u8; MAX_MESSAGE_DATA_SIZE] = [0; MAX_MESSAGE_DATA_SIZE];
        let packed = ConfigureAdvancedBurst::new(
            true,
            AdvancedBurstMaxPacketLength::Max24Byte,
            SupportedFeatures::new(false),
            SupportedFeatures::new(true),
            None,
            None,
        );
        let size = packed.serialize_message(&mut buf[..]).unwrap();
        assert_eq!(buf[..size], [0, 1, 0x03, 0, 0, 0, 1, 0, 0]);
        // TODO test optional fields
        // test RX
        let unpacked = AdvancedBurstCurrentConfiguration::unpack_from_slice::<
            SerialError,
            SerialError,
        >(&[1, 1, 2, 1, 0, 0, 0, 0, 0])
        .unwrap();

        assert_eq!(unpacked.data.enable, true);
        assert_eq!(
            unpacked.data.max_packet_length,
            AdvancedBurstMaxPacketLength::Max16Byte
        );
        assert_eq!(
            unpacked
                .data
                .required_features
                .adv_burst_frequency_hop_enabled,
            true
        );
        assert_eq!(
            unpacked
                .data
                .optional_features
                .adv_burst_frequency_hop_enabled,
            false
        );
        // TODO optional fields
    }

    #[test]
    fn configure_event_filter() {
        let packed = ConfigureEventFilter::new(
            true, false, false, false, false, false, false, true, true, true,
        );
        assert_eq!(packed.pack().unwrap(), [0, 0x81, 0x3]);
    }

    #[test]
    fn set_selective_data_update_mask() {
        let packed = SetSelectiveDataUpdateMask::new(0, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(packed.pack().unwrap(), [0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn enable_single_channel_encryption() {
        let packed = EnableSingleChannelEncryption::new(0, EncryptionMode::Enable, 0x7F);
        assert_eq!(packed.pack().unwrap(), [0, 1, 0, 0x7F]);
    }

    #[test]
    fn set_encryption_key() {
        let packed = SetEncryptionKey::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        assert_eq!(
            packed.pack().unwrap(),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
    }

    #[test]
    fn set_encryption_info_encryption_id() {
        let packed = SetEncryptionInfoEncryptionId::new([3, 4, 5, 6]);
        assert_eq!(packed.pack().unwrap(), [0, 3, 4, 5, 6]);
    }

    #[test]
    fn set_encryption_info_user_information_string() {
        let packed = SetEncryptionInfoUserInformationString::new([
            2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        ]);
        assert_eq!(
            packed.pack().unwrap(),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
        );
    }

    #[test]
    fn set_encryption_info_random_seed() {
        let packed = SetEncryptionInfoRandomSeed::new([
            3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        ]);
        assert_eq!(
            packed.pack().unwrap(),
            [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18]
        );
    }

    #[test]
    fn channel_search_sharing() {
        let packed = ChannelSearchSharing::new(0, 1);
        assert_eq!(packed.pack().unwrap(), [0, 1]);
    }

    #[test]
    fn load_encryption_key_from_nvm() {
        let packed = LoadEncryptionKeyFromNvm::new(1);
        assert_eq!(packed.pack().unwrap(), [0, 1, 0]);
    }

    #[test]
    fn store_encryption_key_in_nvm() {
        let packed = StoreEncryptionKeyInNvm::new(
            2,
            [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18],
        );
        assert_eq!(
            packed.pack().unwrap(),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18]
        );
    }

    #[test]
    fn startup_message() {
        let unpacked = StartUpMessage::unpack(&[0x02]).unwrap();
        assert_eq!(unpacked.watch_dog_reset, true);
    }

    #[test]
    fn serial_error_message() {
        let unpacked = SerialErrorMessage::unpack(&[0x02]).unwrap();
        assert_eq!(
            unpacked.error_number,
            SerialErrorType::IncorrectChecksumByte
        );
    }

    #[test]
    fn reset_system() {
        let packed = ResetSystem::new();
        assert_eq!(packed.pack().unwrap(), [0]);
    }

    #[test]
    fn open_channel() {
        let packed = OpenChannel::new(0);
        assert_eq!(packed.pack().unwrap(), [0]);
    }

    #[test]
    fn close_channel() {
        let packed = CloseChannel::new(0);
        assert_eq!(packed.pack().unwrap(), [0]);
    }

    #[test]
    fn sleep_message() {
        let packed = SleepMessage::new();
        assert_eq!(packed.pack().unwrap(), [0]);
    }

    #[test]
    fn broadcast_data() {
        let packed = BroadcastData::unpack_from_slice::<SerialError, SerialError>(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8,
        ])
        .unwrap();
        assert_eq!(packed.payload.channel_number, 0);
        assert_eq!(packed.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(packed.extended_info, None);
        let packed = BroadcastData::unpack_from_slice::<SerialError, SerialError>(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 0x20, 0xBB, 0xAA,
        ])
        .unwrap();
        assert_eq!(packed.payload.channel_number, 0);
        assert_eq!(packed.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        let ext_info = packed.extended_info.unwrap();
        assert_eq!(ext_info.flag_byte.channel_id_output, false);
        assert_eq!(ext_info.flag_byte.rssi_output, false);
        assert_eq!(ext_info.flag_byte.timestamp_output, true);
        assert_eq!(ext_info.timestamp_output.unwrap().rx_timestamp, 0xAABB);

        // TODO TX
    }

    #[test]
    fn acknowledged_data() {
        let packed = AcknowledgedData::unpack_from_slice::<SerialError, SerialError>(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8,
        ])
        .unwrap();
        assert_eq!(packed.payload.channel_number, 0);
        assert_eq!(packed.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(packed.extended_info, None);
        let packed = AcknowledgedData::unpack_from_slice::<SerialError, SerialError>(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 0x20, 0xBB, 0xAA,
        ])
        .unwrap();
        assert_eq!(packed.payload.channel_number, 0);
        assert_eq!(packed.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        let ext_info = packed.extended_info.unwrap();
        assert_eq!(ext_info.flag_byte.channel_id_output, false);
        assert_eq!(ext_info.flag_byte.rssi_output, false);
        assert_eq!(ext_info.flag_byte.timestamp_output, true);
        assert_eq!(ext_info.timestamp_output.unwrap().rx_timestamp, 0xAABB);

        // TODO TX
    }

    #[test]
    fn burst_transfer_data() {
        let packed = BurstTransferData::unpack_from_slice::<SerialError, SerialError>(&[
            0x21, 1, 2, 3, 4, 5, 6, 7, 8,
        ])
        .unwrap();
        assert_eq!(packed.payload.channel_sequence.channel_number, 1.into());
        assert_eq!(packed.payload.channel_sequence.sequence_number, 1.into());
        assert_eq!(packed.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(packed.extended_info, None);
        let packed = BurstTransferData::unpack_from_slice::<SerialError, SerialError>(&[
            0x20, 1, 2, 3, 4, 5, 6, 7, 8, 0x20, 0xBB, 0xAA,
        ])
        .unwrap();
        assert_eq!(packed.payload.channel_sequence.channel_number, 0.into());
        assert_eq!(packed.payload.channel_sequence.sequence_number, 1.into());
        assert_eq!(packed.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        let ext_info = packed.extended_info.unwrap();
        assert_eq!(ext_info.flag_byte.channel_id_output, false);
        assert_eq!(ext_info.flag_byte.rssi_output, false);
        assert_eq!(ext_info.flag_byte.timestamp_output, true);
        assert_eq!(ext_info.timestamp_output.unwrap().rx_timestamp, 0xAABB);

        // TODO TX
    }

    #[test]
    fn channel_response() -> Result<(), PackingError> {
        let unpacked = ChannelResponse::unpack(&[1, 0x6E, 0x00])?;
        assert_eq!(unpacked.channel_number, 1);
        assert_eq!(unpacked.message_id, TxMessageId::LibConfig);
        assert_eq!(unpacked.message_code, MessageCode::ResponseNoError);
        Ok(())
    }

    #[test]
    fn channel_status() {
        let unpacked = ChannelStatus::unpack(&[1, 0x36]).unwrap();
        assert_eq!(unpacked.channel_number, 1);
        assert_eq!(
            unpacked.channel_type,
            ChannelType::SharedBidirectionalMaster
        );
        assert_eq!(unpacked.network_number, 1);
        assert_eq!(unpacked.channel_state, ChannelState::Searching);
    }

    #[test]
    fn serial_number() {
        let unpacked = SerialNumber::unpack(&[0xAA, 0xBB, 0xCC, 0xDD]).unwrap();
        assert_eq!(unpacked.serial_number, [0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn event_buffer_configuration() {
        let unpacked = EventBufferConfiguration::unpack(&[0, 1, 0xAA, 0xBB, 0xCC, 0xDD]).unwrap();
        assert_eq!(unpacked.config, EventBufferConfig::BufferAllEvents);
        assert_eq!(unpacked.size, 0xBBAA);
        assert_eq!(unpacked.time, 0xDDCC);
    }

    #[test]
    fn encryption_mode_parameters() {
        let unpacked =
            EncryptionModeParameters::unpack_from_slice::<SerialError, SerialError>(&[0, 1])
                .unwrap();
        assert_eq!(
            unpacked.requested_encryption_parameter,
            RequestedEncryptionParameter::MaxSupportedEncryptionMode
        );
        let mode = inner!(unpacked.requested_encryption_parameter_data,
            if RequestedEncryptionParameterData::MaxSupportedEncryptionMode);
        assert_eq!(mode, EncryptionMode::Enable);
        let unpacked = EncryptionModeParameters::unpack_from_slice::<SerialError, SerialError>(&[
            1, 0xAA, 0xBB, 0xCC, 0xDD,
        ])
        .unwrap();
        assert_eq!(
            unpacked.requested_encryption_parameter,
            RequestedEncryptionParameter::EncryptionId
        );
        let id = inner!(unpacked.requested_encryption_parameter_data,
            if RequestedEncryptionParameterData::EncryptionId);
        assert_eq!(id, [0xAA, 0xBB, 0xCC, 0xDD]);
        let data = [
            2, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
            0xFF, 0x01, 0x02, 0x03, 0x04,
        ];
        let unpacked =
            EncryptionModeParameters::unpack_from_slice::<SerialError, SerialError>(&data).unwrap();
        assert_eq!(
            unpacked.requested_encryption_parameter,
            RequestedEncryptionParameter::UserInformationString
        );
        let id = inner!(unpacked.requested_encryption_parameter_data,
            if RequestedEncryptionParameterData::UserInformationString);
        assert_eq!(id, &data[1..]);
    }

    #[test]
    fn user_nvm() {
        let unpacked =
            UserNvm::unpack_from_slice::<SerialError, SerialError>(&[0, 1, 2, 3, 4]).unwrap();
        assert_eq!(unpacked.data.len(), 4);
        assert_eq!(unpacked.data.as_slice(), &[1, 2, 3, 4]);
        let unpacked =
            UserNvm::unpack_from_slice::<SerialError, SerialError>(&[0, 1, 2, 3, 4, 5, 6]).unwrap();
        assert_eq!(unpacked.data.len(), 6);
        assert_eq!(unpacked.data.as_slice(), &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn ant_version() -> Result<(), DriverError<SerialError, SerialError>> {
        let input = [0x64, 0x65, 0x61, 0x64, 0x62, 0x65, 0x65, 0x66];
        let unpacked = AntVersion::unpack_from_slice::<SerialError, SerialError>(&input)?;
        assert_eq!(unpacked.version.as_slice(), input);
        Ok(())
    }

    #[test]
    fn cw_init() -> Result<(), PackingError> {
        let packed = CwInit::new();
        assert_eq!(packed.pack()?, [0]);
        Ok(())
    }

    #[test]
    fn cw_test() {
        let packed = CwTest::new(1, 2);
        assert_eq!(packed.pack().unwrap(), [0, 1, 2]);
    }

    #[test]
    fn advanced_burst_data() {
        let unpacked = AdvancedBurstData::unpack_from_slice::<SerialError, SerialError>(&[
            10, 1, 2, 3, 4, 5, 6, 7, 8,
        ])
        .unwrap();
        let mut buf = ArrayVec::<u8, ADVANCED_BURST_BUFFER_SIZE>::new();
        [1, 2, 3, 4, 5, 6, 7, 8].iter().for_each(|x| buf.push(*x));
        assert_eq!(
            unpacked,
            AdvancedBurstData {
                channel_sequence: ChannelSequence {
                    channel_number: 10.into(),
                    sequence_number: 0.into(),
                },
                data: buf,
            }
        );
        // TODO TX
    }
}
