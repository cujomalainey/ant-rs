// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use packed_struct::prelude::*;

use crate::drivers::DriverError;

#[derive(PrimitiveEnum_u8, PartialEq, Copy, Clone, Debug)]
pub enum TransmissionChannelType {
    Reserved = 0b00,
    IndependentChannel = 0b01,
    SharedChannel1ByteAddress = 0b10,
    SharedChannel2ByteAddress = 0b11,
}

impl Default for TransmissionChannelType {
    fn default() -> Self {
        TransmissionChannelType::IndependentChannel
    }
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum TransmissionGlobalDataPages {
    GlobalDataPagesNotUsed = 0,
    GlobalDataPagesUsed = 1,
}

impl Default for TransmissionGlobalDataPages {
    fn default() -> Self {
        TransmissionGlobalDataPages::GlobalDataPagesNotUsed
    }
}

#[derive(PackedStruct, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct TransmissionType {
    #[packed_field(bits = "0:1", ty = "enum")]
    pub transmission_channel_type: TransmissionChannelType,
    #[packed_field(bits = "2", ty = "enum")]
    pub global_datapages_used: TransmissionGlobalDataPages,
    #[packed_field(bits = "3")]
    _reserved: ReservedZeroes<packed_bits::Bits1>,
    #[packed_field(bits = "4:7")]
    pub device_number_extension: Integer<u8, packed_bits::Bits4>,
}

impl TransmissionType {
    pub fn new(
        transmission_channel_type: TransmissionChannelType,
        global_datapages_used: TransmissionGlobalDataPages,
        device_number_extension: Integer<u8, packed_bits::Bits4>,
    ) -> Self {
        Self {
            transmission_channel_type,
            global_datapages_used,
            device_number_extension,
            ..TransmissionType::default()
        }
    }
}

pub trait Wildcard {
    fn wildcard(&mut self);
    fn new_wildcard() -> Self;
}

impl Wildcard for TransmissionType {
    fn wildcard(&mut self) {
        self.transmission_channel_type = TransmissionChannelType::Reserved;
        self.global_datapages_used = TransmissionGlobalDataPages::GlobalDataPagesNotUsed;
        self.device_number_extension = 0.into();
    }

    fn new_wildcard() -> Self {
        Self {
            transmission_channel_type: TransmissionChannelType::Reserved,
            global_datapages_used: TransmissionGlobalDataPages::GlobalDataPagesNotUsed,
            device_number_extension: 0.into(),
            ..TransmissionType::default()
        }
    }
}

#[derive(PackedStruct, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct DeviceType {
    #[packed_field(bits = "0:6")]
    pub device_type_id: Integer<u8, packed_bits::Bits7>,
    #[packed_field(bits = "7")]
    pub pairing_request: bool,
}

impl DeviceType {
    pub fn new(device_type_id: Integer<u8, packed_bits::Bits7>, pairing_request: bool) -> Self {
        Self {
            device_type_id,
            pairing_request,
        }
    }
}

impl Wildcard for DeviceType {
    fn wildcard(&mut self) {
        self.pairing_request = false;
        self.device_type_id = 0.into();
    }

    fn new_wildcard() -> Self {
        Self {
            pairing_request: false,
            device_type_id: 0.into(),
        }
    }
}

impl Default for ListExclusion {
    fn default() -> Self {
        ListExclusion::Include
    }
}

#[derive(PrimitiveEnum_u16, Clone, Copy, PartialEq, Debug)]
pub enum SearchWaveformValue {
    Standard = 316,
    Fast = 97,
}

impl Default for SearchWaveformValue {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum EventBufferConfig {
    BufferLowPriorityEvents = 0,
    BufferAllEvents = 1,
}

impl Default for EventBufferConfig {
    fn default() -> Self {
        EventBufferConfig::BufferLowPriorityEvents
    }
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum AdvancedBurstMaxPacketLength {
    Max8Byte = 0x01,
    Max16Byte = 0x02,
    Max24Byte = 0x03,
}

impl Default for AdvancedBurstMaxPacketLength {
    fn default() -> Self {
        AdvancedBurstMaxPacketLength::Max8Byte
    }
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum ListExclusion {
    Include = 0,
    Exclude = 1,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum EncryptionMode {
    Disable = 0x00,
    Enable = 0x01,
    EnabledAndIncludeUserInformationString = 0x02,
}

impl Default for EncryptionMode {
    fn default() -> Self {
        EncryptionMode::Disable
    }
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum RequestedEncryptionParameter {
    MaxSupportedEncryptionMode = 0,
    EncryptionId = 1,
    UserInformationString = 2,
}

pub type EncryptionId = [u8; 4];
pub type UserInformationString = [u8; 19];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RequestedEncryptionParameterData {
    MaxSupportedEncryptionMode(EncryptionMode),
    EncryptionId(EncryptionId),
    UserInformationString(UserInformationString),
}

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

// Note, this is bit shifted 4 bits relative to the offical doc because the field would overlap in
// the channel status message. The result is the same just a minor mismatch compared to official
// docs
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum ChannelType {
    BidirectionalSlave = 0,
    BidirectionalMaster = 1,
    SharedBidirectionalSlave = 2,
    SharedBidirectionalMaster = 3,
    SharedReceiveOnly = 4,
    MasterTransmitOnly = 5,
}

impl Default for ChannelType {
    fn default() -> Self {
        ChannelType::BidirectionalSlave
    }
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum SerialErrorType {
    IncorrectSyncByte = 0x00,
    IncorrectChecksumByte = 0x02,
    IncorrectMessageLength = 0x03,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum MessageCode {
    ResponseNoError = 0x00,
    EventRxSearchTimeout = 0x01,
    EventRxFail = 0x02,
    EventTx = 0x03,
    EventTransferRxFailed = 0x04,
    EventTransferTxCompleted = 0x05,
    EventTransferTxFailed = 0x06,
    EventChannelClosed = 0x07,
    EventRxFailGoToSearch = 0x08,
    EventChannelCollision = 0x09,
    EventTransferTxStart = 0x0A,
    EventTransferNextDataBlock = 0x11,
    ChannelInWrongState = 0x15,
    ChannelNotOpened = 0x16,
    ChannelIdNotSet = 0x18,
    CloseAllChannels = 0x19,
    TransferInProgress = 0x1F,
    TransferSequenceNumberError = 0x20,
    TransferInError = 0x21,
    MessageSizeExceedsLimit = 0x27,
    InvalidMessage = 0x28,
    InvalidNetworkNumber = 0x29,
    InvalidListId = 0x30,
    InvalidScanTxChannel = 0x31,
    InvalidParameterProvided = 0x32,
    EventSerialQueOverflow = 0x34,
    EventQueOverflow = 0x35,
    EncryptNegotiationSuccess = 0x38,
    EncryptNegotiationFail = 0x39,
    NvmFullError = 0x40,
    NvmWriteError = 0x41,
    UsbStringWriteFail = 0x70,
    MesgSerialErrorId = 0xAE, // TODO verify how this behaves with "data portion"
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum ChannelState {
    UnAssigned = 0,
    Assigned = 1,
    Searching = 2,
    Tracking = 3,
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
    // AdvancedBurstConfiguration      = 0x78,
    EventFilter = 0x79,
    SelectiveDataUpdateMaskSetting = 0x7B,
    UserNvm = 0x7C,
    EncryptionModeParameters = 0x7D,
    // Extended Data Messages (Legacy)
    // #define EXTENDED_BROADCAST_DATA             0x5D
    // #define EXTENDED_ACKNOWLEDGED_DATA          0x5E
    // #define EXTENDED_BURST_DATA                 0x5F
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum RequestableMessageId {
    ChannelStatus = 0x52,
    ChannelId = 0x51,
    AntVersion = 0x3E,
    Capabilities = 0x54,
    SerialNumber = 0x61,
    EventBufferConfiguration = 0x74,
    AdvancedBurstCapabilities = 0x78,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum ListType {
    Whitelist = 0,
    Blacklist = 1,
}

impl Default for ListType {
    fn default() -> Self {
        ListType::Whitelist
    }
}

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

const CHANNEL_ID_OUTPUT_SIZE: usize = 4;

#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "4")]
pub struct ChannelIdOutput {
    #[packed_field(bytes = "0:1")]
    pub device_number: u16,
    #[packed_field(bytes = "2")]
    pub device_type: DeviceType,
    #[packed_field(bytes = "3")]
    pub transmission_type: TransmissionType,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Debug)]
pub enum RssiMeasurementType {
    Agc = 0x10,
    Dbm = 0x20,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RssiOutput {
    pub measurement_type: RssiMeasurementType,
    pub measurement_value: RssiMeasurementValue,
}

impl RssiOutput {
    pub(crate) fn unpack_from_slice<R, W>(data: &[u8]) -> Result<RssiOutput, DriverError<R, W>> {
        let measurement_type =
            RssiMeasurementType::from_primitive(data[0]).ok_or(DriverError::InvalidData())?;
        let measurement_value = match measurement_type {
            RssiMeasurementType::Agc => {
                RssiMeasurementValue::Agc(MeasurementValueAgc::unpack_from_slice(&data[1..])?)
            }
            RssiMeasurementType::Dbm => {
                RssiMeasurementValue::Dbm(MeasurementValueDbm::unpack_from_slice(&data[1..])?)
            }
        };
        Ok(RssiOutput {
            measurement_type,
            measurement_value,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RssiMeasurementValue {
    Dbm(MeasurementValueDbm),
    Agc(MeasurementValueAgc),
}

const RSSI_OUTPUT_DBM_SIZE: usize = 3;

#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct MeasurementValueDbm {
    #[packed_field(bytes = "0")]
    pub rssi_value: i8,
    #[packed_field(bytes = "1")]
    pub threshold_configuration_value: i8,
}

const RSSI_OUTPUT_AGC_SIZE: usize = 4;

// https://www.thisisant.com/forum/viewthread/4280/
#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct MeasurementValueAgc {
    #[packed_field(bytes = "0")]
    pub threshold_offset: i8,
    #[packed_field(bytes = "1:2")]
    pub register: u16,
}

const TIMESTAMP_OUTPUT_SIZE: usize = 2;

#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct TimestampOutput {
    #[packed_field(bytes = "0:1")]
    pub rx_timestamp: u16,
}

const FLAG_BYTE_SIZE: usize = 1;

#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct FlagByte {
    #[packed_field(bits = "7")]
    pub channel_id_output: bool,
    #[packed_field(bits = "6")]
    pub rssi_output: bool,
    #[packed_field(bits = "5")]
    pub timestamp_output: bool,
    #[packed_field(bits = "0:4")]
    _reserved: ReservedZeroes<packed_bits::Bits5>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtendedInfo {
    pub flag_byte: FlagByte,
    pub channel_id_output: Option<ChannelIdOutput>,
    pub rssi_output: Option<RssiOutput>,
    pub timestamp_output: Option<TimestampOutput>,
}

impl ExtendedInfo {
    pub(crate) fn unpack_from_slice<R, W>(
        data: &[u8],
    ) -> Result<Option<ExtendedInfo>, DriverError<R, W>> {
        if data.is_empty() {
            return Ok(None);
        }

        let flag_byte = FlagByte::unpack_from_slice(&data[..FLAG_BYTE_SIZE])?;

        let mut extended_info = ExtendedInfo {
            flag_byte,
            channel_id_output: None,
            rssi_output: None,
            timestamp_output: None,
        };

        let mut expected_size = 0;

        let data = &data[FLAG_BYTE_SIZE..];

        let data = if flag_byte.channel_id_output {
            let msg_data = if data.len() > CHANNEL_ID_OUTPUT_SIZE {
                &data[..CHANNEL_ID_OUTPUT_SIZE]
            } else {
                data
            };

            extended_info.channel_id_output = Some(ChannelIdOutput::unpack_from_slice(msg_data)?);
            expected_size += CHANNEL_ID_OUTPUT_SIZE;

            &data[CHANNEL_ID_OUTPUT_SIZE..]
        } else {
            data
        };

        let data = if flag_byte.rssi_output {
            // Hack to handle https://www.thisisant.com/forum/viewthread/4280/
            let format =
                RssiMeasurementType::from_primitive(data[0]).ok_or(DriverError::InvalidData())?;
            let slice_size = match format {
                RssiMeasurementType::Agc => RSSI_OUTPUT_AGC_SIZE,
                RssiMeasurementType::Dbm => RSSI_OUTPUT_DBM_SIZE,
            };
            let msg_data = if data.len() > slice_size {
                &data[..slice_size]
            } else {
                data
            };

            extended_info.rssi_output = Some(RssiOutput::unpack_from_slice(msg_data)?);
            expected_size += slice_size;

            &data[slice_size..]
        } else {
            data
        };

        let data = if flag_byte.timestamp_output {
            let msg_data = if data.len() > TIMESTAMP_OUTPUT_SIZE {
                &data[..TIMESTAMP_OUTPUT_SIZE]
            } else {
                data
            };

            extended_info.timestamp_output = Some(TimestampOutput::unpack_from_slice(msg_data)?);
            expected_size += TIMESTAMP_OUTPUT_SIZE;

            &data[TIMESTAMP_OUTPUT_SIZE..]
        } else {
            data
        };

        if !data.is_empty() {
            return Err(DriverError::BadLength(
                expected_size + data.len(),
                expected_size,
            ));
        }

        Ok(Some(extended_info))
    }
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

#[derive(PackedStruct, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct ChannelSequence {
    #[packed_field(bits = "7:5")]
    pub sequence_number: Integer<u8, packed_bits::Bits3>,
    #[packed_field(bits = "4:0")]
    pub channel_number: Integer<u8, packed_bits::Bits5>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ChannelEventExtension {
    EncryptNegotiationSuccess(EncryptionId, Option<UserInformationString>),
    EncryptNegotiationFail(EncryptionId),
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "4")]
pub struct BaseCapabilities {
    #[packed_field(bytes = "0")]
    pub max_ant_channels: u8,
    #[packed_field(bytes = "1")]
    pub max_networks: u8,
    #[packed_field(bytes = "2")]
    pub standard_options: StandardOptions,
    #[packed_field(bytes = "3")]
    pub advanced_options: AdvancedOptions,
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct StandardOptions {
    #[packed_field(bits = "0")]
    pub no_recieve_channels: bool,
    #[packed_field(bits = "1")]
    pub no_transmit_channels: bool,
    #[packed_field(bits = "2")]
    pub no_recieve_messages: bool,
    #[packed_field(bits = "3")]
    pub no_transmit_messages: bool,
    #[packed_field(bits = "4")]
    pub no_acked_messages: bool,
    #[packed_field(bits = "5")]
    pub no_burst_messages: bool,
    #[packed_field(bits = "6:7")]
    _reserved: ReservedZeroes<packed_bits::Bits2>,
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct AdvancedOptions {
    #[packed_field(bits = "0")]
    _reserved: ReservedZeroes<packed_bits::Bits1>,
    #[packed_field(bits = "1")]
    pub network_enabled: bool,
    #[packed_field(bits = "2")]
    _reserved1: ReservedZeroes<packed_bits::Bits1>,
    #[packed_field(bits = "3")]
    pub serial_number_enabled: bool,
    #[packed_field(bits = "4")]
    pub per_channel_tx_power_enabled: bool,
    #[packed_field(bits = "5")]
    pub low_priority_search_enabled: bool,
    #[packed_field(bits = "6")]
    pub script_enabled: bool,
    #[packed_field(bits = "7")]
    pub search_list_enabled: bool,
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct AdvancedOptions2 {
    #[packed_field(bits = "0")]
    pub led_enabled: bool,
    #[packed_field(bits = "1")]
    pub ext_message_enabled: bool,
    #[packed_field(bits = "2")]
    pub scan_mode_enabled: bool,
    #[packed_field(bits = "3")]
    _reserved: ReservedZeroes<packed_bits::Bits1>,
    #[packed_field(bits = "4")]
    pub prox_search_enabled: bool,
    #[packed_field(bits = "5")]
    pub ext_assign_enabled: bool,
    #[packed_field(bits = "6")]
    pub fs_antfs_enabled: bool,
    #[packed_field(bits = "7")]
    pub fit1_enabled: bool,
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct AdvancedOptions3 {
    #[packed_field(bits = "0")]
    pub advanced_burst_enabled: bool,
    #[packed_field(bits = "1")]
    pub event_buffering_enabled: bool,
    #[packed_field(bits = "2")]
    pub event_filtering_enabled: bool,
    #[packed_field(bits = "3")]
    pub high_duty_search_enabled: bool,
    #[packed_field(bits = "4")]
    pub search_sharing_enabled: bool,
    #[packed_field(bits = "5")]
    _reserved: ReservedZeroes<packed_bits::Bits1>,
    #[packed_field(bits = "6")]
    pub selective_data_updates_enabled: bool,
    #[packed_field(bits = "7")]
    pub encrypted_channel_enabled: bool,
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct AdvancedOptions4 {
    #[packed_field(bits = "0")]
    pub rfactive_notification_enabled: bool,
    #[packed_field(bits = "1:7")]
    _reserved: ReservedZeroes<packed_bits::Bits7>,
}

#[derive(PackedStruct, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct SupportedFeatures {
    #[packed_field(bits = "0:6")]
    _reserved: ReservedZeroes<packed_bits::Bits7>,
    #[packed_field(bits = "7")]
    pub adv_burst_frequency_hop_enabled: bool,
    #[packed_field(bits = "8:23")]
    _reserved1: ReservedZeroes<packed_bits::Bits16>,
}

impl SupportedFeatures {
    pub fn new(adv_burst_frequency_hop_enabled: bool) -> Self {
        Self {
            adv_burst_frequency_hop_enabled,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use packed_struct::PackedStruct;

    // Needed for generics
    #[derive(Debug, PartialEq)]
    enum TestErrors {}

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

    #[test]
    fn flag_byte() {
        let unpacked = FlagByte::unpack(&[0x20]).unwrap();
        assert_eq!(unpacked.channel_id_output, false);
        assert_eq!(unpacked.rssi_output, false);
        assert_eq!(unpacked.timestamp_output, true);
    }

    #[test]
    fn channel_id_output() {
        let unpacked = ChannelIdOutput::unpack(&[0xAA, 0xBB, 0xCC, 0xDD]).unwrap();
        assert_eq!(unpacked.device_number, 0xBBAA);
        assert_eq!(unpacked.device_type.pairing_request, true);
        assert_eq!(unpacked.device_type.device_type_id, 0x4C.into());
        assert_eq!(
            unpacked.transmission_type.transmission_channel_type,
            TransmissionChannelType::IndependentChannel
        );
        assert_eq!(
            unpacked.transmission_type.global_datapages_used,
            TransmissionGlobalDataPages::GlobalDataPagesUsed
        );
        assert_eq!(
            unpacked.transmission_type.device_number_extension,
            0xD.into()
        );
    }

    #[test]
    fn rssi_output() {
        let unpacked = RssiOutput::unpack_from_slice::<TestErrors, TestErrors>(&[
            0x20, 0b10110000, 0b10111010,
        ])
        .unwrap();
        assert_eq!(unpacked.measurement_type, RssiMeasurementType::Dbm);
        let unpacked = match unpacked.measurement_value {
            RssiMeasurementValue::Dbm(e) => e,
            RssiMeasurementValue::Agc(_) => panic!("Incorrect enum"),
        };
        assert_eq!(unpacked.rssi_value, -80);
        assert_eq!(unpacked.threshold_configuration_value, -70);
    }

    #[test]
    fn timestamp_output() {
        let unpacked = TimestampOutput::unpack(&[0xAA, 0xBB]).unwrap();
        assert_eq!(unpacked.rx_timestamp, 0xBBAA);
    }

    #[test]
    fn channel_sequence() {
        let unpacked = ChannelSequence::unpack(&[0x3F]).unwrap();
        assert_eq!(unpacked.sequence_number, 0x1.into());
        assert_eq!(unpacked.channel_number, 0x1F.into());
    }

    #[test]
    fn extended_info() {
        let unpacked = ExtendedInfo::unpack_from_slice::<TestErrors, TestErrors>(&[]).unwrap();
        assert_eq!(unpacked, None);

        let unpacked =
            ExtendedInfo::unpack_from_slice::<TestErrors, TestErrors>(&[0x40, 0x20, 0xCE, 0x80])
                .unwrap()
                .unwrap();
        assert_eq!(unpacked.flag_byte.channel_id_output, false);
        assert_eq!(unpacked.flag_byte.rssi_output, true);
        assert_eq!(unpacked.flag_byte.timestamp_output, false);
        assert_eq!(unpacked.channel_id_output.is_none(), true);
        assert_eq!(unpacked.timestamp_output.is_none(), true);

        let rssi = unpacked.rssi_output.unwrap();
        assert_eq!(rssi.measurement_type, RssiMeasurementType::Dbm);
        let rssi = match rssi.measurement_value {
            RssiMeasurementValue::Dbm(e) => e,
            RssiMeasurementValue::Agc(_) => panic!("Incorrect enum"),
        };
        assert_eq!(rssi.rssi_value, -50);
        assert_eq!(rssi.threshold_configuration_value, -128);

        let unpacked = ExtendedInfo::unpack_from_slice::<TestErrors, TestErrors>(&[
            0x60, 0x10, 0xCE, 0x80, 0x60, 0xAA, 0xBB,
        ])
        .unwrap()
        .unwrap();
        assert_eq!(unpacked.flag_byte.channel_id_output, false);
        assert_eq!(unpacked.flag_byte.rssi_output, true);
        assert_eq!(unpacked.flag_byte.timestamp_output, true);
        assert_eq!(unpacked.channel_id_output.is_none(), true);

        let rssi = unpacked.rssi_output.unwrap();
        assert_eq!(rssi.measurement_type, RssiMeasurementType::Agc);
        let rssi = match rssi.measurement_value {
            RssiMeasurementValue::Dbm(_) => panic!("Incorrect enum"),
            RssiMeasurementValue::Agc(e) => e,
        };
        assert_eq!(rssi.threshold_offset, -50);
        assert_eq!(rssi.register, 0x6080);

        let timestamp = unpacked.timestamp_output.unwrap();
        assert_eq!(timestamp.rx_timestamp, 0xBBAA);
    }

    #[test]
    fn supported_features() {
        let unpacked = SupportedFeatures::unpack_from_slice(&[0x1, 0, 0]).unwrap();
        assert_eq!(unpacked.adv_burst_frequency_hop_enabled, true);
    }

    #[test]
    fn base_capabilities() {
        let unpacked = BaseCapabilities::unpack_from_slice(&[15, 4, 0x15, 0x52]).unwrap();
        assert_eq!(unpacked.max_ant_channels, 15);
        assert_eq!(unpacked.max_networks, 4);
        assert_eq!(unpacked.standard_options.no_recieve_channels, true);
        assert_eq!(unpacked.standard_options.no_transmit_channels, false);
        assert_eq!(unpacked.standard_options.no_recieve_messages, true);
        assert_eq!(unpacked.standard_options.no_transmit_messages, false);
        assert_eq!(unpacked.standard_options.no_acked_messages, true);
        assert_eq!(unpacked.standard_options.no_burst_messages, false);
        assert_eq!(unpacked.advanced_options.network_enabled, true);
        assert_eq!(unpacked.advanced_options.serial_number_enabled, false);
        assert_eq!(unpacked.advanced_options.per_channel_tx_power_enabled, true);
        assert_eq!(unpacked.advanced_options.low_priority_search_enabled, false);
        assert_eq!(unpacked.advanced_options.script_enabled, true);
        assert_eq!(unpacked.advanced_options.search_list_enabled, false);
    }

    #[test]
    fn standard_options() {
        let unpacked = StandardOptions::unpack_from_slice(&[0x2A]).unwrap();
        assert_eq!(unpacked.no_recieve_channels, false);
        assert_eq!(unpacked.no_transmit_channels, true);
        assert_eq!(unpacked.no_recieve_messages, false);
        assert_eq!(unpacked.no_transmit_messages, true);
        assert_eq!(unpacked.no_acked_messages, false);
        assert_eq!(unpacked.no_burst_messages, true);
    }

    #[test]
    fn advanced_options() {
        let unpacked = AdvancedOptions::unpack_from_slice(&[0xA8]).unwrap();
        assert_eq!(unpacked.network_enabled, false);
        assert_eq!(unpacked.serial_number_enabled, true);
        assert_eq!(unpacked.per_channel_tx_power_enabled, false);
        assert_eq!(unpacked.low_priority_search_enabled, true);
        assert_eq!(unpacked.script_enabled, false);
        assert_eq!(unpacked.search_list_enabled, true);
    }

    #[test]
    fn advanced_options_2() {
        let unpacked = AdvancedOptions2::unpack_from_slice(&[0xC7]).unwrap();
        assert_eq!(unpacked.led_enabled, true);
        assert_eq!(unpacked.ext_message_enabled, true);
        assert_eq!(unpacked.scan_mode_enabled, true);
        assert_eq!(unpacked.prox_search_enabled, false);
        assert_eq!(unpacked.ext_assign_enabled, false);
        assert_eq!(unpacked.fs_antfs_enabled, true);
        assert_eq!(unpacked.fit1_enabled, true);
    }

    #[test]
    fn advanced_options_3() -> Result<(), packed_struct::PackingError> {
        let unpacked = AdvancedOptions3::unpack_from_slice(&[0x53])?;
        assert_eq!(unpacked.advanced_burst_enabled, true);
        assert_eq!(unpacked.event_buffering_enabled, true);
        assert_eq!(unpacked.event_filtering_enabled, false);
        assert_eq!(unpacked.high_duty_search_enabled, false);
        assert_eq!(unpacked.search_sharing_enabled, true);
        assert_eq!(unpacked.selective_data_updates_enabled, true);
        assert_eq!(unpacked.encrypted_channel_enabled, false);
        Ok(())
    }

    #[test]
    fn advanced_options_4() {
        let unpacked = AdvancedOptions4::unpack_from_slice(&[0x1]).unwrap();
        assert_eq!(unpacked.rfactive_notification_enabled, true);
    }
}
