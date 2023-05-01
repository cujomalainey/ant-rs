// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::messages::{AntAutoPackWithExtention, TransmitableMessage, TxMessage, TxMessageId};
use ant_derive::AntTx;
use derive_new::new;
use packed_struct::prelude::*;

// Re-export reused types
pub use crate::messages::requested_response::{EncryptionId, UserInformationString};

/// Represents a UnAssign Channel Message (0x41)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct UnAssignChannel {
    /// Channel to be unassigned
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
}

// Note, this is bit shifted 4 bits relative to the offical doc because the field would overlap in
// the channel status message. The result is the same just a minor mismatch compared to official
// docs
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq, Default)]
pub enum ChannelType {
    #[default]
    BidirectionalSlave = 0,
    BidirectionalMaster = 1,
    SharedBidirectionalSlave = 2,
    SharedBidirectionalMaster = 3,
    SharedReceiveOnly = 4,
    MasterTransmitOnly = 5,
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

#[derive(PackedStruct, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct ExtendedAssignment {
    #[packed_field(bits = "0")]
    pub always_search: bool,
    #[packed_field(bits = "1")]
    pub ignore_transmission_type: bool,
    #[packed_field(bits = "2")]
    pub frequency_agility: bool,
    #[packed_field(bits = "3")]
    pub auto_shared_slave: bool,
    #[packed_field(bits = "4")]
    pub fast_initiation_mode: bool,
    #[packed_field(bits = "5")]
    pub async_tx_mode: bool,
    #[packed_field(bits = "6:7")]
    _reserved: ReservedZeroes<packed_bits::Bits2>,
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
    /// Constructs a new `AssignChannel`.
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

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug, Default)]
pub enum TransmissionChannelType {
    Reserved = 0b00,
    #[default]
    IndependentChannel = 0b01,
    SharedChannel1ByteAddress = 0b10,
    SharedChannel2ByteAddress = 0b11,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug, Default)]
pub enum TransmissionGlobalDataPages {
    #[default]
    GlobalDataPagesNotUsed = 0,
    GlobalDataPagesUsed = 1,
}

#[derive(PackedStruct, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct TransmissionType {
    #[packed_field(bits = "0:1", ty = "enum")]
    pub transmission_channel_type: TransmissionChannelType,
    #[packed_field(bits = "2", ty = "enum")]
    pub global_datapages_used: TransmissionGlobalDataPages,
    #[new(default)]
    #[packed_field(bits = "3")]
    _reserved: ReservedZeroes<packed_bits::Bits1>,
    // TODO alias this type when https://github.com/hashmismatch/packed_struct.rs/issues/86 is
    // resolved
    #[packed_field(bits = "4:7")]
    pub device_number_extension: Integer<u8, packed_bits::Bits4>,
}

impl TransmissionType {
    /// Modifies the type into a wildcarded value.
    pub fn wildcard(&mut self) {
        self.transmission_channel_type = TransmissionChannelType::Reserved;
        self.global_datapages_used = TransmissionGlobalDataPages::GlobalDataPagesNotUsed;
        self.device_number_extension = 0.into();
    }

    /// Constructs a new `TransmissionType` with wildcarded values.
    pub fn new_wildcard() -> Self {
        Self {
            transmission_channel_type: TransmissionChannelType::Reserved,
            global_datapages_used: TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
            device_number_extension: 0.into(),
            ..TransmissionType::default()
        }
    }
}

#[derive(PackedStruct, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct DeviceType {
    #[packed_field(bits = "0:6")]
    pub device_type_id: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "7")]
    pub pairing_request: bool,
}

impl DeviceType {
    /// Modifies the type into a wildcarded value.
    pub fn wildcard(&mut self) {
        self.pairing_request = false;
        self.device_type_id = 0.into();
    }

    /// Constructs a new `DeviceType` with wildcarded values.
    pub fn new_wildcard() -> Self {
        Self {
            pairing_request: false,
            device_type_id: 0.into(),
        }
    }
}

/// Represents a Channel Id message (0x51)
///
/// This message is both RX and TX capable
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
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
    /// Set all fields to their wildcard values
    pub fn wildcard(&mut self) {
        self.device_number = 0;
        self.device_type.wildcard();
        self.transmission_type.wildcard();
    }

    /// Constructs a new `ChannelId` with wildcarded values.
    pub fn new_wildcard(channel: u8) -> Self {
        Self {
            channel_number: channel,
            device_number: 0,
            device_type: DeviceType::new_wildcard(),
            transmission_type: TransmissionType::new_wildcard(),
        }
    }
}

/// Represents a Channel Period message (0x43)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
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

/// Represents a Search Timeout message (0x44)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
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

/// Represents a Channel RF Frequency (0x45)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
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

/// Represents a Set Network Key message (0x46)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
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
    /// Size of a default network key
    pub const NETWORK_KEY_SIZE: usize = 8;
}

/// Represents a Transmit Power message (0x47)
///
/// Same as [SetChannelTransmitPower] but for all channels
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct TransmitPower {
    #[new(default)]
    #[packed_field(bytes = "0")]
    _reserved: ReservedZeroes<packed_bits::Bits8>,
    /// Sets TX power for all channels
    ///
    /// Dbm correlation is chip dependent, please chip and ANT messaging documentation
    #[packed_field(bytes = "1")]
    pub tx_power: u8,
}

#[derive(PrimitiveEnum_u16, Clone, Copy, PartialEq, Debug, Default)]
pub enum SearchWaveformValue {
    #[default]
    Standard = 316,
    Fast = 97,
}

/// Represents a Search Waveform message (0x49)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, PartialEq)]
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

/// Represents a Add Channel ID To List message (0x59)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
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

/// Represents a Add Encryption ID To List message (0x59)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
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

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug, Default)]
pub enum ListExclusion {
    #[default]
    Include = 0,
    Exclude = 1,
}

/// Represents a Config ID List message (0x5A)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
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

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug, Default)]
pub enum ListType {
    #[default]
    Whitelist = 0,
    Blacklist = 1,
}

/// Represents a Config Encryption ID List message (0x5A)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
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

/// Represents a Set Channel Transmit Power message (0x60)
///
/// Same as [TransmitPower] but only for a single channel
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct SetChannelTransmitPower {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Power to be used, please refer to docs for value to dbm conversion as it is chip dependent
    #[packed_field(bytes = "1")]
    pub transmit_power: u8,
}

/// Represents a Low Priority Search Timeout message (0x63)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct LowPrioritySearchTimeout {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Search timeout in counts of 2.5s
    #[packed_field(bytes = "1")]
    pub search_timeout: u8,
}

/// Represents a Serial Number Set Channel Id message (0x65)
///
/// This message is not available in softdevice mode
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
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

/// Represents a Enable Ext Rx Messages message (0x66)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct EnableExtRxMessages {
    #[new(default)]
    #[packed_field(bits = "0:14")]
    _reserved: ReservedZeroes<packed_bits::Bits15>,
    /// enable extended messages
    #[packed_field(bits = "15")]
    pub enable: bool,
}

/// Represents an Enable LED message (0x68)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct EnableLed {
    #[new(default)]
    #[packed_field(bits = "0:14")]
    _reserved: ReservedZeroes<packed_bits::Bits15>,
    #[packed_field(bits = "15")]
    /// Switch to enable/disable
    pub enable: bool,
}

/// Represents a Crystal Enable message (0x6D)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct CrystalEnable {
    #[new(default)]
    #[packed_field(bytes = "0")]
    _reserved: ReservedZeroes<packed_bits::Bits8>,
}

/// Represents a Lib Config message (0x6E)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct LibConfig {
    #[new(default)]
    #[packed_field(bytes = "0")]
    _reserved0: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bits = "8")]
    pub enable_channel_id_output: bool,
    #[packed_field(bits = "9")]
    pub enable_rssi_output: bool,
    #[packed_field(bits = "10")]
    pub enable_rx_timestamp_output: bool,
    #[new(default)]
    #[packed_field(bits = "11:15")]
    _reserved1: ReservedZeroes<packed_bits::Bits5>,
}

/// Represents a Frequency Agility message (0x70)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, PartialEq)]
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

/// Represents a Proximity Search message (0x71)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct ProximitySearch {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Search threshold to use
    #[packed_field(bytes = "1")]
    pub search_threshold: u8,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug, Default)]
pub enum EventBufferConfig {
    #[default]
    BufferLowPriorityEvents = 0,
    BufferAllEvents = 1,
}

/// Represents a Configure Event Buffer message (0x74)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "6")]
pub struct ConfigureEventBuffer {
    #[new(default)]
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

/// Represents a Channel Search Priority message (0x75)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct ChannelSearchPriority {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Priority used in searching
    #[packed_field(bytes = "1")]
    pub search_priority: u8,
}

/// Represents a Set 128 Bit Network Key message (0x76)
#[derive(PackedStruct, AntTx, new, Clone, Copy, Debug, Default, PartialEq)]
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
#[derive(PackedStruct, new, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct HighDutySearchSuppressionCycle {
    #[new(default)]
    #[packed_field(bits = "3:7")]
    _reserved: ReservedZeroes<packed_bits::Bits5>,
    /// high priority search suppression in increments of 250ms, limit is 5 and is full
    /// suppression, 0 is no suppression
    #[packed_field(bits = "0:2")]
    suppression_cycle: u8,
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
    /// Constructs a new `HighDutySearch`.
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

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug, Default)]
pub enum AdvancedBurstMaxPacketLength {
    #[default]
    Max8Byte = 0x01,
    Max16Byte = 0x02,
    Max24Byte = 0x03,
}

#[derive(PackedStruct, new, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct SupportedFeatures {
    #[new(default)]
    #[packed_field(bits = "0:6")]
    _reserved: ReservedZeroes<packed_bits::Bits7>,
    #[packed_field(bits = "7")]
    pub adv_burst_frequency_hop_enabled: bool,
    #[new(default)]
    #[packed_field(bits = "8:23")]
    _reserved1: ReservedZeroes<packed_bits::Bits16>,
}

/// Represents Configure Advanced Burst required fields
#[derive(PackedStruct, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "9")]
pub struct ConfigureAdvancedBurstData {
    #[new(default)]
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

impl ConfigureAdvancedBurstData {
    const PACKING_SIZE: usize = 9;
}

/// Represents a Configure Advanced Burst message (0x78)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConfigureAdvancedBurst {
    /// Required Fields
    pub data: ConfigureAdvancedBurstData,
    /// Optional stall count fields
    // TODO why is this no just raw u8 and u16?
    pub stall_count: Option<Integer<u16, packed_bits::Bits16>>,
    /// Optional retry count fields
    ///
    /// Note, to use retry count, you must also use stall count
    pub retry_count_extension: Option<Integer<u8, packed_bits::Bits8>>,
}

impl ConfigureAdvancedBurst {
    const STALL_COUNT_SIZE: usize = 2;
    const RETRY_COUNT_EXTENSION_SIZE: usize = 1;
}

impl TransmitableMessage for ConfigureAdvancedBurst {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
        let mut len = ConfigureAdvancedBurstData::PACKING_SIZE;
        let (data_buf, buf) = buf.split_at_mut(len);
        self.data.pack_to_slice(data_buf)?;
        if let Some(data) = self.stall_count {
            let (stall_buf, buf) = buf.split_at_mut(ConfigureAdvancedBurst::STALL_COUNT_SIZE);
            len += ConfigureAdvancedBurst::STALL_COUNT_SIZE;
            stall_buf.copy_from_slice(data.to_lsb_bytes()?.as_slice());
            let retry_buf = &mut buf[..ConfigureAdvancedBurst::RETRY_COUNT_EXTENSION_SIZE];
            if let Some(retry_count) = self.retry_count_extension {
                retry_buf.copy_from_slice(retry_count.to_lsb_bytes()?.as_slice());
                len += ConfigureAdvancedBurst::RETRY_COUNT_EXTENSION_SIZE;
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
    /// Constructs a new `ConfigureAdvancedBurst`.
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

    pub(crate) fn unpack_from_slice(buf: &[u8]) -> Result<ConfigureAdvancedBurst, PackingError> {
        let data_buf = buf
            .get(..ConfigureAdvancedBurstData::PACKING_SIZE)
            .ok_or(PackingError::BufferTooSmall)?;
        let buf = buf.get(ConfigureAdvancedBurstData::PACKING_SIZE..);
        let data = ConfigureAdvancedBurstData::unpack_from_slice(data_buf)?;

        let mut msg = ConfigureAdvancedBurst {
            data,
            stall_count: None,
            retry_count_extension: None,
        };

        let buf = match buf {
            Some(x) => x,
            None => return Ok(msg),
        };

        if buf.len() < 2 {
            return Err(PackingError::BufferSizeMismatch {
                expected: 2,
                actual: buf.len(),
            });
        }

        let (stall_buf, buf) = buf.split_at(2);
        let stall_count_bytes = match stall_buf.try_into() {
            Ok(x) => x,
            Err(_) => return Err(PackingError::SliceIndexingError { slice_len: 2 }),
        };
        msg.stall_count = Some(Integer::<u16, packed_bits::Bits16>::from_lsb_bytes(
            stall_count_bytes,
        )?);

        if buf.is_empty() {
            return Ok(msg);
        }

        if buf.len() != 1 {
            return Err(PackingError::BufferSizeMismatch {
                expected: 1,
                actual: buf.len(),
            });
        }

        let retry_buf = &buf[..1];

        let retry_count_extension_bytes = match retry_buf.try_into() {
            Ok(x) => x,
            Err(_) => return Err(PackingError::SliceIndexingError { slice_len: 1 }),
        };
        msg.retry_count_extension = Some(Integer::<u8, packed_bits::Bits8>::from_lsb_bytes(
            retry_count_extension_bytes,
        )?);

        Ok(msg)
    }
}

/// Represents a Configure Event Filter message (0x79)
#[allow(clippy::too_many_arguments)]
#[derive(PackedStruct, AntTx, new, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct ConfigureEventFilter {
    #[new(default)]
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
    #[new(default)]
    #[packed_field(bits = "16:21")]
    _reserved1: ReservedZeroes<packed_bits::Bits8>,
}

/// Represents a Configure Selective Data Updates message (0x7A)
#[derive(PackedStruct, AntTx, new, Debug, Default, PartialEq)]
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

/// Represents a Set Selective Data Update Mask message (0x7B)
#[derive(PackedStruct, AntTx, new, Clone, Debug, Default, PartialEq)]
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

// TODO configure user nvme message

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug, Default)]
pub enum EncryptionMode {
    #[default]
    Disable = 0x00,
    Enable = 0x01,
    EnabledAndIncludeUserInformationString = 0x02,
}

/// Represents a Enable Single Channel Encryption message (0x7D)
#[derive(PackedStruct, AntTx, new, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "4")]
pub struct EnableSingleChannelEncryption {
    /// Channel to be configured
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    /// Encryption mode to be used
    #[packed_field(bytes = "1", ty = "enum")]
    pub encryption_mode: EncryptionMode,
    /// Per version 5.1 of the spec this field has a range of 0
    #[new(default)]
    #[packed_field(bytes = "2")]
    pub volatile_key_index: ReservedZeroes<packed_bits::Bits8>,
    /// Master channel rate / slave tracking channel rate
    #[packed_field(bytes = "3")]
    pub decimation_rate: u8,
}

#[derive(PackedStruct, AntTx, new, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "17")]
pub struct SetEncryptionKey {
    /// Per version 5.1 of the spec this field has a range of 0
    #[new(default)]
    #[packed_field(bytes = "0")]
    pub volatile_key_index: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bytes = "1:16")]
    pub encryption_key: [u8; 16],
}

// The spec defines this as a single variable message but variable types are
// basically impossible with the packed_stuct lib so it is easier to just
// implement 3 message types to handle all the cases.
#[derive(PackedStruct, AntTx, new, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "5")]
pub struct SetEncryptionInfoEncryptionId {
    // 0 for encryption id
    #[new(default)]
    #[packed_field(bytes = "0")]
    pub set_parameter: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bytes = "1:4")]
    pub encryption_id: EncryptionId,
}

#[derive(PackedStruct, AntTx, new, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "20")]
pub struct SetEncryptionInfoUserInformationString {
    // 1 for User Information String
    #[new(default)]
    #[packed_field(bits = "0:6")]
    pub set_parameter0: ReservedZeroes<packed_bits::Bits7>,
    #[new(default)]
    #[packed_field(bits = "7")]
    pub set_parameter1: ReservedOnes<packed_bits::Bits1>,
    #[packed_field(bytes = "1:19")]
    pub user_information_string: UserInformationString,
}

#[derive(PackedStruct, AntTx, new, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "17")]
pub struct SetEncryptionInfoRandomSeed {
    // 2 for Random Number Seed
    #[new(default)]
    #[packed_field(bits = "0:5")]
    pub set_parameter0: ReservedZeroes<packed_bits::Bits6>,
    #[new(default)]
    #[packed_field(bits = "6")]
    pub set_parameter1: ReservedOnes<packed_bits::Bits1>,
    #[new(default)]
    #[packed_field(bits = "7")]
    pub set_parameter2: ReservedZeroes<packed_bits::Bits1>,
    #[packed_field(bytes = "1:16")]
    pub random_seed: [u8; 16],
}

#[derive(PackedStruct, AntTx, new, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct ChannelSearchSharing {
    #[packed_field(bytes = "0")]
    pub channel_number: u8,
    #[packed_field(bytes = "1")]
    pub search_sharing_cycles: u8,
}

#[derive(PackedStruct, AntTx, new, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct LoadEncryptionKeyFromNvm {
    #[new(default)]
    #[packed_field(bytes = "0")]
    pub operation: ReservedZeroes<packed_bits::Bits8>,
    #[packed_field(bytes = "1")]
    pub nvm_key_index: u8,
    // 0 per spec v5.1
    #[new(default)]
    #[packed_field(bytes = "2")]
    volatile_key_index: ReservedZeroes<packed_bits::Bits8>,
}

#[derive(PackedStruct, AntTx, new, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "18")]
pub struct StoreEncryptionKeyInNvm {
    #[new(default)]
    #[packed_field(bits = "0:6")]
    pub operation0: ReservedZeroes<packed_bits::Bits7>,
    #[new(default)]
    #[packed_field(bits = "7")]
    pub operation1: ReservedOnes<packed_bits::Bits1>,
    #[packed_field(bytes = "1")]
    pub nvm_key_index: u8,
    #[packed_field(bytes = "2:17")]
    pub encryption_key: [u8; 16],
}

// TODO SetUsbDescriptorString

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transmission_type() {
        let unpacked = TransmissionType::unpack(&[0b11110110]).unwrap();
        assert_eq!(
            unpacked.transmission_channel_type,
            TransmissionChannelType::SharedChannel1ByteAddress
        );
        assert_eq!(
            unpacked.global_datapages_used,
            TransmissionGlobalDataPages::GlobalDataPagesUsed
        );
        assert_eq!(*unpacked.device_number_extension, 0b1111);

        let mut packed = TransmissionType::new(
            TransmissionChannelType::IndependentChannel,
            TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
            0b1001.into(),
        );

        assert_eq!(packed.pack().unwrap(), [0b1001_0001]);

        packed.wildcard();
        assert_eq!(packed.pack().unwrap(), [0x0]);
    }

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
        use crate::messages::MAX_MESSAGE_DATA_SIZE;
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
}
