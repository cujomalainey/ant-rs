// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::messages::MAX_MESSAGE_DATA_SIZE;
use arrayvec::ArrayVec;
use packed_struct::prelude::*;

// Rexport reused types so they exist in all expected namespaces based on the datasheet
pub use crate::messages::config::{
    AdvancedBurstMaxPacketLength, ChannelId, ChannelType, DeviceType, EncryptionMode,
    SupportedFeatures, TransmissionChannelType, TransmissionGlobalDataPages, TransmissionType,
};

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum ChannelState {
    UnAssigned = 0,
    Assigned = 1,
    Searching = 2,
    Tracking = 3,
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

#[derive(Clone, Debug, PartialEq)]
pub struct AntVersion {
    version: ArrayVec<u8, MAX_MESSAGE_DATA_SIZE>,
}

impl AntVersion {
    pub(crate) fn unpack_from_slice(data: &[u8]) -> Result<Self, PackingError> {
        let data_bytes = match data.try_into() {
            Ok(x) => x,
            Err(_) => {
                return Err(PackingError::SliceIndexingError {
                    slice_len: data.len(),
                })
            }
        };
        Ok(Self {
            version: data_bytes,
        })
    }
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

impl BaseCapabilities {
    const PACKING_SIZE: usize = 4;
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct StandardOptions {
    #[packed_field(bits = "0")]
    pub no_receive_channels: bool,
    #[packed_field(bits = "1")]
    pub no_transmit_channels: bool,
    #[packed_field(bits = "2")]
    pub no_receive_messages: bool,
    #[packed_field(bits = "3")]
    pub no_transmit_messages: bool,
    #[packed_field(bits = "4")]
    pub no_acked_messages: bool,
    #[packed_field(bits = "5")]
    pub no_burst_messages: bool,
    #[packed_field(bits = "6:7")]
    _reserved: ReservedZeroes<packed_bits::Bits<2>>,
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct AdvancedOptions {
    #[packed_field(bits = "0")]
    _reserved: ReservedZeroes<packed_bits::Bits<1>>,
    #[packed_field(bits = "1")]
    pub network_enabled: bool,
    #[packed_field(bits = "2")]
    _reserved1: ReservedZeroes<packed_bits::Bits<1>>,
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
    _reserved: ReservedZeroes<packed_bits::Bits<1>>,
    #[packed_field(bits = "4")]
    pub prox_search_enabled: bool,
    #[packed_field(bits = "5")]
    pub ext_assign_enabled: bool,
    #[packed_field(bits = "6")]
    pub fs_antfs_enabled: bool,
    #[packed_field(bits = "7")]
    pub fit1_enabled: bool,
}

impl AdvancedOptions2 {
    const PACKING_SIZE: usize = 1;
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
    _reserved: ReservedZeroes<packed_bits::Bits<1>>,
    #[packed_field(bits = "6")]
    pub selective_data_updates_enabled: bool,
    #[packed_field(bits = "7")]
    pub encrypted_channel_enabled: bool,
}

impl AdvancedOptions3 {
    const PACKING_SIZE: usize = 1;
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct AdvancedOptions4 {
    #[packed_field(bits = "0")]
    pub rfactive_notification_enabled: bool,
    #[packed_field(bits = "1:7")]
    _reserved: ReservedZeroes<packed_bits::Bits<7>>,
}

impl AdvancedOptions4 {
    const PACKING_SIZE: usize = 1;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Capabilities {
    pub base_capabilities: BaseCapabilities,
    pub advanced_options2: Option<AdvancedOptions2>,
    pub max_sensrcore_channels: Option<u8>,
    pub advanced_options3: Option<AdvancedOptions3>,
    pub advanced_options4: Option<AdvancedOptions4>,
}

impl Capabilities {
    const MAX_SENSRCORE_CHANNELS_SIZE: usize = 1;

    pub(crate) fn unpack_from_slice(data: &[u8]) -> Result<Self, PackingError> {
        let base_buf =
            data.get(..BaseCapabilities::PACKING_SIZE)
                .ok_or(PackingError::BufferSizeMismatch {
                    expected: BaseCapabilities::PACKING_SIZE,
                    actual: data.len(),
                })?;
        let data = data
            .get(BaseCapabilities::PACKING_SIZE..)
            .ok_or(PackingError::BufferTooSmall)?;
        let base_capabilities = BaseCapabilities::unpack_from_slice(base_buf)?;

        if data.is_empty() {
            return Ok(Capabilities {
                base_capabilities,
                advanced_options2: None,
                max_sensrcore_channels: None,
                advanced_options3: None,
                advanced_options4: None,
            });
        }

        let adv2_buf =
            data.get(..AdvancedOptions2::PACKING_SIZE)
                .ok_or(PackingError::BufferSizeMismatch {
                    actual: data.len(),
                    expected: AdvancedOptions2::PACKING_SIZE,
                })?;
        let data =
            data.get(AdvancedOptions2::PACKING_SIZE..)
                .ok_or(PackingError::BufferSizeMismatch {
                    actual: data.len(),
                    expected: AdvancedOptions2::PACKING_SIZE,
                })?;
        let advanced_options2 = AdvancedOptions2::unpack_from_slice(adv2_buf)?;

        if data.is_empty() {
            return Ok(Capabilities {
                base_capabilities,
                advanced_options2: Some(advanced_options2),
                max_sensrcore_channels: None,
                advanced_options3: None,
                advanced_options4: None,
            });
        }

        let max_sensrcore_channels = data.first().ok_or(PackingError::BufferSizeMismatch {
            actual: data.len(),
            expected: 1,
        })?;
        let data = data.get(Self::MAX_SENSRCORE_CHANNELS_SIZE..).ok_or(
            PackingError::BufferSizeMismatch {
                actual: data.len(),
                expected: 1,
            },
        )?;

        if data.is_empty() {
            return Ok(Capabilities {
                base_capabilities,
                advanced_options2: Some(advanced_options2),
                max_sensrcore_channels: Some(*max_sensrcore_channels),
                advanced_options3: None,
                advanced_options4: None,
            });
        }

        let adv3_buf =
            data.get(..AdvancedOptions3::PACKING_SIZE)
                .ok_or(PackingError::BufferSizeMismatch {
                    actual: data.len(),
                    expected: AdvancedOptions3::PACKING_SIZE,
                })?;
        let data =
            data.get(AdvancedOptions3::PACKING_SIZE..)
                .ok_or(PackingError::BufferSizeMismatch {
                    actual: data.len(),
                    expected: AdvancedOptions3::PACKING_SIZE,
                })?;
        let advanced_options3 = AdvancedOptions3::unpack_from_slice(adv3_buf)?;

        if data.is_empty() {
            return Ok(Capabilities {
                base_capabilities,
                advanced_options2: Some(advanced_options2),
                max_sensrcore_channels: Some(*max_sensrcore_channels),
                advanced_options3: Some(advanced_options3),
                advanced_options4: None,
            });
        }

        let adv4_buf =
            data.get(..AdvancedOptions4::PACKING_SIZE)
                .ok_or(PackingError::BufferSizeMismatch {
                    actual: data.len(),
                    expected: AdvancedOptions4::PACKING_SIZE,
                })?;
        let data =
            data.get(AdvancedOptions4::PACKING_SIZE..)
                .ok_or(PackingError::BufferSizeMismatch {
                    actual: data.len(),
                    expected: AdvancedOptions4::PACKING_SIZE,
                })?;
        let advanced_options4 = AdvancedOptions4::unpack_from_slice(adv4_buf)?;

        if data.is_empty() {
            return Ok(Capabilities {
                base_capabilities,
                advanced_options2: Some(advanced_options2),
                max_sensrcore_channels: Some(*max_sensrcore_channels),
                advanced_options3: Some(advanced_options3),
                advanced_options4: Some(advanced_options4),
            });
        }

        let expected_size = BaseCapabilities::PACKING_SIZE
            + AdvancedOptions2::PACKING_SIZE
            + Self::MAX_SENSRCORE_CHANNELS_SIZE
            + AdvancedOptions3::PACKING_SIZE
            + AdvancedOptions4::PACKING_SIZE;
        Err(PackingError::BufferSizeMismatch {
            expected: expected_size,
            actual: expected_size + data.len(),
        })
    }
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "5")]
pub struct AdvancedBurstCapabilities {
    #[packed_field(bytes = "0")]
    _reserved: ReservedZeroes<packed_bits::Bits<8>>,
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
pub use crate::messages::config::ConfigureAdvancedBurst as AdvancedBurstCurrentConfiguration;
pub use crate::messages::config::ConfigureEventBuffer as EventBufferConfiguration;
pub use crate::messages::config::ConfigureEventFilter as EventFilter;
pub use crate::messages::config::EventBufferConfig;
pub use crate::messages::config::SetSelectiveDataUpdateMask as SelectiveDataUpdateMaskSetting;

#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "1")]
pub struct UserNvmHeader {
    #[packed_field(bytes = "0")]
    resered: ReservedZeroes<packed_bits::Bits<8>>,
}

// TODO conditionally compile this, also magic num
#[derive(Clone, Debug, PartialEq)]
pub struct UserNvm {
    header: UserNvmHeader,
    data: ArrayVec<u8, 255>,
}

impl UserNvm {
    pub(crate) fn unpack_from_slice(data: &[u8]) -> Result<UserNvm, PackingError> {
        let data_bytes = match data
            .get(1..)
            .ok_or(PackingError::BufferTooSmall)?
            .try_into()
        {
            Ok(x) => x,
            Err(_) => {
                return Err(PackingError::SliceIndexingError {
                    slice_len: data.len() - 1,
                })
            }
        };
        Ok(UserNvm {
            header: UserNvmHeader::unpack_from_slice(
                data.get(..1).ok_or(PackingError::BufferTooSmall)?,
            )?,
            data: data_bytes,
        })
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EncryptionModeParameters {
    pub requested_encryption_parameter: RequestedEncryptionParameter,
    pub requested_encryption_parameter_data: RequestedEncryptionParameterData,
}

impl EncryptionModeParameters {
    pub(crate) fn unpack_from_slice(data: &[u8]) -> Result<EncryptionModeParameters, PackingError> {
        if data.is_empty() {
            return Err(PackingError::BufferSizeMismatch {
                expected: 1,
                actual: 0,
            });
        }
        let parameter = RequestedEncryptionParameter::from_primitive(data[0])
            .ok_or(PackingError::InvalidValue)?;
        let data = &data[1..];
        let data = match parameter {
            RequestedEncryptionParameter::MaxSupportedEncryptionMode => {
                if data.len() != 1 {
                    return Err(PackingError::BufferSizeMismatch {
                        expected: 1,
                        actual: data.len(),
                    });
                }
                let param = match EncryptionMode::from_primitive(data[0]) {
                    Some(x) => x,
                    None => return Err(PackingError::InvalidValue),
                };
                RequestedEncryptionParameterData::MaxSupportedEncryptionMode(param)
            }
            RequestedEncryptionParameter::EncryptionId => {
                let encryption_id = match EncryptionId::try_from(data) {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(PackingError::SliceIndexingError {
                            slice_len: data.len(),
                        })
                    }
                };
                RequestedEncryptionParameterData::EncryptionId(encryption_id)
            }
            RequestedEncryptionParameter::UserInformationString => {
                let user_information_string = match UserInformationString::try_from(data) {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(PackingError::SliceIndexingError {
                            slice_len: data.len(),
                        })
                    }
                };
                RequestedEncryptionParameterData::UserInformationString(user_information_string)
            }
        };
        Ok(EncryptionModeParameters {
            requested_encryption_parameter: parameter,
            requested_encryption_parameter_data: data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inner::inner;

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
        assert_eq!(unpacked.standard_options.no_receive_channels, true);
        assert_eq!(unpacked.standard_options.no_transmit_channels, false);
        assert_eq!(unpacked.standard_options.no_receive_messages, true);
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
        assert_eq!(unpacked.no_receive_channels, false);
        assert_eq!(unpacked.no_transmit_channels, true);
        assert_eq!(unpacked.no_receive_messages, false);
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

    #[test]
    fn capabilities() {
        let unpacked =
            Capabilities::unpack_from_slice(&[16, 4, 0x15, 0x82, 4, 8, 0x40, 1]).unwrap();
        assert_eq!(unpacked.base_capabilities.max_ant_channels, 16);
        assert_eq!(unpacked.base_capabilities.max_networks, 4);
        assert_eq!(
            unpacked
                .base_capabilities
                .standard_options
                .no_receive_channels,
            true
        );
        assert_eq!(
            unpacked
                .base_capabilities
                .standard_options
                .no_receive_messages,
            true
        );
        assert_eq!(
            unpacked
                .base_capabilities
                .standard_options
                .no_acked_messages,
            true
        );
        assert_eq!(
            unpacked.base_capabilities.advanced_options.network_enabled,
            true
        );
        assert_eq!(
            unpacked
                .base_capabilities
                .advanced_options
                .search_list_enabled,
            true
        );
        assert_eq!(unpacked.advanced_options2.unwrap().scan_mode_enabled, true);
        assert_eq!(unpacked.max_sensrcore_channels.unwrap(), 8);
        assert_eq!(
            unpacked
                .advanced_options3
                .unwrap()
                .selective_data_updates_enabled,
            true
        );
        assert_eq!(
            unpacked
                .advanced_options4
                .unwrap()
                .rfactive_notification_enabled,
            true
        );
        let unpacked = Capabilities::unpack_from_slice(&[16, 4, 0x15, 0x82]).unwrap();
        assert_eq!(unpacked.base_capabilities.max_ant_channels, 16);
        assert_eq!(unpacked.base_capabilities.max_networks, 4);
        assert_eq!(
            unpacked
                .base_capabilities
                .standard_options
                .no_receive_channels,
            true
        );
        assert_eq!(
            unpacked
                .base_capabilities
                .standard_options
                .no_receive_messages,
            true
        );
        assert_eq!(
            unpacked
                .base_capabilities
                .standard_options
                .no_acked_messages,
            true
        );
        assert_eq!(
            unpacked.base_capabilities.advanced_options.network_enabled,
            true
        );
        assert_eq!(
            unpacked
                .base_capabilities
                .advanced_options
                .search_list_enabled,
            true
        );
        assert_eq!(unpacked.advanced_options2.is_none(), true);
        assert_eq!(unpacked.max_sensrcore_channels.is_none(), true);
        assert_eq!(unpacked.advanced_options3.is_none(), true);
        assert_eq!(unpacked.advanced_options4.is_none(), true);
        let unpacked = Capabilities::unpack_from_slice(&[16, 4, 0x15, 0x82, 4, 8]).unwrap();
        assert_eq!(unpacked.base_capabilities.max_ant_channels, 16);
        assert_eq!(unpacked.base_capabilities.max_networks, 4);
        assert_eq!(
            unpacked
                .base_capabilities
                .standard_options
                .no_receive_channels,
            true
        );
        assert_eq!(
            unpacked
                .base_capabilities
                .standard_options
                .no_receive_messages,
            true
        );
        assert_eq!(
            unpacked
                .base_capabilities
                .standard_options
                .no_acked_messages,
            true
        );
        assert_eq!(
            unpacked.base_capabilities.advanced_options.network_enabled,
            true
        );
        assert_eq!(
            unpacked
                .base_capabilities
                .advanced_options
                .search_list_enabled,
            true
        );
        assert_eq!(unpacked.advanced_options2.unwrap().scan_mode_enabled, true);
        assert_eq!(unpacked.max_sensrcore_channels.unwrap(), 8);
        assert_eq!(unpacked.advanced_options3.is_none(), true);
        assert_eq!(unpacked.advanced_options4.is_none(), true);
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
    fn advanced_burst_capabilities() {
        let unpacked = AdvancedBurstCapabilities::unpack(&[0, 2, 1, 0, 0]).unwrap();
        assert_eq!(
            unpacked.supported_max_packed_length,
            AdvancedBurstMaxPacketLength::Max16Byte
        );
        assert_eq!(
            unpacked.supported_features.adv_burst_frequency_hop_enabled,
            true
        );
    }

    #[test]
    fn encryption_mode_parameters() {
        let unpacked = EncryptionModeParameters::unpack_from_slice(&[0, 1]).unwrap();
        assert_eq!(
            unpacked.requested_encryption_parameter,
            RequestedEncryptionParameter::MaxSupportedEncryptionMode
        );
        let mode = inner!(unpacked.requested_encryption_parameter_data,
            if RequestedEncryptionParameterData::MaxSupportedEncryptionMode);
        assert_eq!(mode, EncryptionMode::Enable);
        let unpacked =
            EncryptionModeParameters::unpack_from_slice(&[1, 0xAA, 0xBB, 0xCC, 0xDD]).unwrap();
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
        let unpacked = EncryptionModeParameters::unpack_from_slice(&data).unwrap();
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
        let unpacked = UserNvm::unpack_from_slice(&[0, 1, 2, 3, 4]).unwrap();
        assert_eq!(unpacked.data.len(), 4);
        assert_eq!(unpacked.data.as_slice(), &[1, 2, 3, 4]);
        let unpacked = UserNvm::unpack_from_slice(&[0, 1, 2, 3, 4, 5, 6]).unwrap();
        assert_eq!(unpacked.data.len(), 6);
        assert_eq!(unpacked.data.as_slice(), &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn ant_version() {
        let input = [0x64, 0x65, 0x61, 0x64, 0x62, 0x65, 0x65, 0x66];
        let unpacked = AntVersion::unpack_from_slice(&input).unwrap();
        assert_eq!(unpacked.version.as_slice(), input);
    }
}
