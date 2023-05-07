// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::messages::{TransmitableMessage, TxMessage, TxMessageId};
use arrayvec::ArrayVec;
use const_utils::{max, min};
use derive_new::new;
use konst::{option::unwrap_or, primitive::parse_usize, unwrap_ctx};
use packed_struct::prelude::*;

pub use crate::messages::config::{
    DeviceType, TransmissionChannelType, TransmissionGlobalDataPages, TransmissionType,
};

// TODO make this crash compilation if out of bounds rather than silently correct
// TODO skip this if NVM is enabled
pub(crate) const ADVANCED_BURST_BUFFER_SIZE: usize = min(
    max(
        unwrap_ctx!(parse_usize(unwrap_or!(
            option_env!("ADV_BURST_BUF_SIZE"),
            "64"
        ))),
        24,
    ),
    254,
);

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

impl ChannelIdOutput {
    const PACKING_SIZE: usize = 4;
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RssiMeasurementValue {
    Dbm(MeasurementValueDbm),
    Agc(MeasurementValueAgc),
}

#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct MeasurementValueDbm {
    #[packed_field(bytes = "0")]
    pub rssi_value: i8,
    #[packed_field(bytes = "1")]
    pub threshold_configuration_value: i8,
}

impl MeasurementValueDbm {
    // +1 for type byte
    const PACKING_SIZE: usize = 3;
}

// https://www.thisisant.com/forum/viewthread/4280/
#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "3")]
pub struct MeasurementValueAgc {
    #[packed_field(bytes = "0")]
    pub threshold_offset: i8,
    #[packed_field(bytes = "1:2")]
    pub register: u16,
}

impl MeasurementValueAgc {
    // +1 for type byte
    const PACKING_SIZE: usize = 4;
}

impl RssiOutput {
    pub(crate) fn unpack_from_slice(data: &[u8]) -> Result<RssiOutput, PackingError> {
        let measurement_type =
            RssiMeasurementType::from_primitive(data[0]).ok_or(PackingError::InvalidValue)?;
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

#[derive(PackedStruct, Clone, Copy, Debug, PartialEq)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "2")]
pub struct TimestampOutput {
    #[packed_field(bytes = "0:1")]
    pub rx_timestamp: u16,
}

impl TimestampOutput {
    const PACKING_SIZE: usize = 2;
}

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
    _reserved: ReservedZeroes<packed_bits::Bits<5>>,
}

impl FlagByte {
    const PACKING_SIZE: usize = 1;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtendedInfo {
    pub flag_byte: FlagByte,
    pub channel_id_output: Option<ChannelIdOutput>,
    pub rssi_output: Option<RssiOutput>,
    pub timestamp_output: Option<TimestampOutput>,
}

impl ExtendedInfo {
    pub(crate) fn unpack_from_slice(data: &[u8]) -> Result<Option<ExtendedInfo>, PackingError> {
        if data.is_empty() {
            return Ok(None);
        }

        let (flag_buf, data) = data.split_at(FlagByte::PACKING_SIZE);
        let flag_byte = FlagByte::unpack_from_slice(flag_buf)?;

        let mut extended_info = ExtendedInfo {
            flag_byte,
            channel_id_output: None,
            rssi_output: None,
            timestamp_output: None,
        };

        let mut expected_size = 0;

        let data = if flag_byte.channel_id_output {
            if data.len() < ChannelIdOutput::PACKING_SIZE {
                return Err(PackingError::BufferSizeMismatch {
                    expected: ChannelIdOutput::PACKING_SIZE,
                    actual: data.len(),
                });
            }
            let (msg_data, data) = data.split_at(ChannelIdOutput::PACKING_SIZE);

            extended_info.channel_id_output = Some(ChannelIdOutput::unpack_from_slice(msg_data)?);
            expected_size += ChannelIdOutput::PACKING_SIZE;

            data
        } else {
            data
        };

        let data = if flag_byte.rssi_output {
            // Hack to handle https://www.thisisant.com/forum/viewthread/4280/
            let format = RssiMeasurementType::from_primitive(*data.first().ok_or(
                PackingError::BufferSizeMismatch {
                    expected: 1,
                    actual: 0,
                },
            )?)
            .ok_or(PackingError::InvalidValue)?;
            let slice_size = match format {
                RssiMeasurementType::Agc => MeasurementValueAgc::PACKING_SIZE,
                RssiMeasurementType::Dbm => MeasurementValueDbm::PACKING_SIZE,
            };
            if data.len() < slice_size {
                return Err(PackingError::BufferSizeMismatch {
                    expected: slice_size,
                    actual: data.len(),
                });
            }
            let (msg_data, data) = data.split_at(slice_size);

            extended_info.rssi_output = Some(RssiOutput::unpack_from_slice(msg_data)?);
            expected_size += slice_size;

            data
        } else {
            data
        };

        let data = if flag_byte.timestamp_output {
            if data.len() < TimestampOutput::PACKING_SIZE {
                return Err(PackingError::BufferSizeMismatch {
                    expected: TimestampOutput::PACKING_SIZE,
                    actual: data.len(),
                });
            }
            let (msg_data, data) = data.split_at(TimestampOutput::PACKING_SIZE);

            extended_info.timestamp_output = Some(TimestampOutput::unpack_from_slice(msg_data)?);
            expected_size += TimestampOutput::PACKING_SIZE;

            data
        } else {
            data
        };

        if !data.is_empty() {
            return Err(PackingError::BufferSizeMismatch {
                expected: expected_size,
                actual: expected_size + data.len(),
            });
        }

        Ok(Some(extended_info))
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

impl BroadcastDataPayload {
    const PACKING_SIZE: usize = 9;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BroadcastData {
    pub payload: BroadcastDataPayload,
    pub extended_info: Option<ExtendedInfo>,
}

impl TransmitableMessage for BroadcastData {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
        // Data payloads have optional RX fields but are ignored on TX
        self.payload
            .pack_to_slice(&mut buf[..BroadcastDataPayload::PACKING_SIZE])?;
        Ok(BroadcastDataPayload::PACKING_SIZE)
    }

    fn get_tx_msg_id(&self) -> TxMessageId {
        TxMessageId::BroadcastData
    }
}

impl From<BroadcastData> for TxMessage {
    fn from(msg: BroadcastData) -> TxMessage {
        TxMessage::BroadcastData(msg)
    }
}

impl BroadcastData {
    /// Constructs a new `BroadcastData`.
    pub fn new(channel_number: u8, data: [u8; 8]) -> Self {
        Self {
            payload: BroadcastDataPayload {
                channel_number,
                data,
            },
            extended_info: None,
        }
    }

    pub(crate) fn unpack_from_slice(data: &[u8]) -> Result<BroadcastData, PackingError> {
        if data.len() < BroadcastDataPayload::PACKING_SIZE {
            return Err(PackingError::BufferSizeMismatch {
                expected: BroadcastDataPayload::PACKING_SIZE,
                actual: data.len(),
            });
        }
        let (payload, extended) = data.split_at(BroadcastDataPayload::PACKING_SIZE);
        Ok(BroadcastData {
            payload: BroadcastDataPayload::unpack_from_slice(payload)?,
            extended_info: ExtendedInfo::unpack_from_slice(extended)?,
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

impl TransmitableMessage for AcknowledgedData {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
        // Data payloads have optional RX fields but are ignored on TX
        self.payload
            .pack_to_slice(&mut buf[..BroadcastDataPayload::PACKING_SIZE])?;
        Ok(BroadcastDataPayload::PACKING_SIZE)
    }
    fn get_tx_msg_id(&self) -> TxMessageId {
        TxMessageId::AcknowledgedData
    }
}

impl From<AcknowledgedData> for TxMessage {
    fn from(msg: AcknowledgedData) -> TxMessage {
        TxMessage::AcknowledgedData(msg)
    }
}

impl AcknowledgedData {
    /// Constructs a new `AcknowledgedData`.
    pub fn new(channel_number: u8, data: [u8; 8]) -> Self {
        Self {
            payload: BroadcastDataPayload {
                channel_number,
                data,
            },
            extended_info: None,
        }
    }

    pub(crate) fn unpack_from_slice(data: &[u8]) -> Result<AcknowledgedData, PackingError> {
        if data.len() < BroadcastDataPayload::PACKING_SIZE {
            return Err(PackingError::BufferSizeMismatch {
                expected: BroadcastDataPayload::PACKING_SIZE,
                actual: data.len(),
            });
        }
        let (payload, extended) = data.split_at(BroadcastDataPayload::PACKING_SIZE);
        Ok(AcknowledgedData {
            payload: AcknowledgedDataPayload::unpack_from_slice(payload)?,
            extended_info: ExtendedInfo::unpack_from_slice(extended)?,
        })
    }
}

#[derive(PackedStruct, new, Clone, Copy, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "1")]
pub struct ChannelSequence {
    #[packed_field(bits = "7:5")]
    pub sequence_number: Integer<u8, packed_bits::Bits<3>>,
    #[packed_field(bits = "4:0")]
    pub channel_number: Integer<u8, packed_bits::Bits<5>>,
}

#[derive(PackedStruct, Copy, Clone, Debug, Default, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "9")]
pub struct BurstTransferDataPayload {
    #[packed_field(bytes = "0")]
    pub channel_sequence: ChannelSequence,
    #[packed_field(bytes = "1:8")]
    pub data: [u8; 8],
}

impl BurstTransferDataPayload {
    const PACKING_SIZE: usize = 9;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BurstTransferData {
    pub payload: BurstTransferDataPayload,
    pub extended_info: Option<ExtendedInfo>,
}

impl TransmitableMessage for BurstTransferData {
    fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
        // Data payloads have optional RX fields but are ignored on TX
        self.payload
            .pack_to_slice(&mut buf[..BurstTransferDataPayload::PACKING_SIZE])?;
        Ok(BurstTransferDataPayload::PACKING_SIZE)
    }
    fn get_tx_msg_id(&self) -> TxMessageId {
        TxMessageId::BurstTransferData
    }
}

impl From<BurstTransferData> for TxMessage {
    fn from(msg: BurstTransferData) -> TxMessage {
        TxMessage::BurstTransferData(msg)
    }
}

impl BurstTransferData {
    /// Constructs a new `BurstTransferData`.
    pub fn new(channel_sequence: ChannelSequence, data: [u8; 8]) -> Self {
        Self {
            payload: BurstTransferDataPayload {
                channel_sequence,
                data,
            },
            extended_info: None,
        }
    }

    pub(crate) fn unpack_from_slice(data: &[u8]) -> Result<BurstTransferData, PackingError> {
        if data.len() < BurstTransferDataPayload::PACKING_SIZE {
            return Err(PackingError::BufferSizeMismatch {
                expected: BurstTransferDataPayload::PACKING_SIZE,
                actual: data.len(),
            });
        }
        let (payload, extended) = data.split_at(BurstTransferDataPayload::PACKING_SIZE);
        Ok(BurstTransferData {
            payload: BurstTransferDataPayload::unpack_from_slice(payload)?,
            extended_info: ExtendedInfo::unpack_from_slice(extended)?,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AdvancedBurstData {
    pub channel_sequence: ChannelSequence,
    pub data: ArrayVec<u8, ADVANCED_BURST_BUFFER_SIZE>,
}

impl AdvancedBurstData {
    /// Constructs a new `AdvancedBurstData`.
    pub fn new(
        channel_sequence: ChannelSequence,
        data: ArrayVec<u8, ADVANCED_BURST_BUFFER_SIZE>,
    ) -> Self {
        Self {
            channel_sequence,
            data,
        }
    }

    pub(crate) fn unpack_from_slice(data: &[u8]) -> Result<Self, PackingError> {
        // TODO this could be cleaned up
        let data_bytes = match data
            .get(1..)
            .ok_or(PackingError::BufferSizeMismatch {
                actual: data.len(),
                expected: 10,
            })?
            .try_into()
        {
            Ok(x) => x,
            Err(_) => return Err(PackingError::SliceIndexingError { slice_len: 1 }),
        };
        Ok(AdvancedBurstData {
            channel_sequence: ChannelSequence::unpack_from_slice(&data[..1])?,
            data: data_bytes,
        })
    }
}

impl TransmitableMessage for AdvancedBurstData {
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

impl From<AdvancedBurstData> for TxMessage {
    fn from(msg: AdvancedBurstData) -> TxMessage {
        TxMessage::AdvancedBurstData(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let unpacked = RssiOutput::unpack_from_slice(&[0x20, 0b10110000, 0b10111010]).unwrap();
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
        let unpacked = ExtendedInfo::unpack_from_slice(&[]).unwrap();
        assert_eq!(unpacked, None);

        let unpacked = ExtendedInfo::unpack_from_slice(&[0x40, 0x20, 0xCE, 0x80])
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

        let unpacked = ExtendedInfo::unpack_from_slice(&[0x60, 0x10, 0xCE, 0x80, 0x60, 0xAA, 0xBB])
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
    fn broadcast_data() {
        let unpacked = BroadcastData::unpack_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        assert_eq!(unpacked.payload.channel_number, 0);
        assert_eq!(unpacked.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(unpacked.extended_info, None);
        let unpacked =
            BroadcastData::unpack_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 0x20, 0xBB, 0xAA])
                .unwrap();
        assert_eq!(unpacked.payload.channel_number, 0);
        assert_eq!(unpacked.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        let ext_info = unpacked.extended_info.unwrap();
        assert_eq!(ext_info.flag_byte.channel_id_output, false);
        assert_eq!(ext_info.flag_byte.rssi_output, false);
        assert_eq!(ext_info.flag_byte.timestamp_output, true);
        assert_eq!(ext_info.timestamp_output.unwrap().rx_timestamp, 0xAABB);

        let mut buf: [u8; 12] = [0; 12];
        let size = BroadcastData::new(5, [5, 6, 7, 8, 9, 0, 1, 2])
            .serialize_message(&mut buf)
            .unwrap();
        assert_eq!(buf, [5, 5, 6, 7, 8, 9, 0, 1, 2, 0, 0, 0]);
        assert_eq!(size, 9);
    }

    #[test]
    fn acknowledged_data() {
        let unpacked = AcknowledgedData::unpack_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        assert_eq!(unpacked.payload.channel_number, 0);
        assert_eq!(unpacked.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(unpacked.extended_info, None);
        let unpacked =
            AcknowledgedData::unpack_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 0x20, 0xBB, 0xAA])
                .unwrap();
        assert_eq!(unpacked.payload.channel_number, 0);
        assert_eq!(unpacked.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        let ext_info = unpacked.extended_info.unwrap();
        assert_eq!(ext_info.flag_byte.channel_id_output, false);
        assert_eq!(ext_info.flag_byte.rssi_output, false);
        assert_eq!(ext_info.flag_byte.timestamp_output, true);
        assert_eq!(ext_info.timestamp_output.unwrap().rx_timestamp, 0xAABB);

        let mut buf: [u8; 12] = [0; 12];
        let size = AcknowledgedData::new(5, [5, 6, 7, 8, 9, 0, 1, 2])
            .serialize_message(&mut buf)
            .unwrap();
        assert_eq!(buf, [5, 5, 6, 7, 8, 9, 0, 1, 2, 0, 0, 0]);
        assert_eq!(size, 9);
    }

    #[test]
    fn burst_transfer_data() {
        let unpacked =
            BurstTransferData::unpack_from_slice(&[0x21, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        assert_eq!(unpacked.payload.channel_sequence.channel_number, 1.into());
        assert_eq!(unpacked.payload.channel_sequence.sequence_number, 1.into());
        assert_eq!(unpacked.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(unpacked.extended_info, None);
        let unpacked =
            BurstTransferData::unpack_from_slice(&[0x20, 1, 2, 3, 4, 5, 6, 7, 8, 0x20, 0xBB, 0xAA])
                .unwrap();
        assert_eq!(unpacked.payload.channel_sequence.channel_number, 0.into());
        assert_eq!(unpacked.payload.channel_sequence.sequence_number, 1.into());
        assert_eq!(unpacked.payload.data, [1, 2, 3, 4, 5, 6, 7, 8]);
        let ext_info = unpacked.extended_info.unwrap();
        assert_eq!(ext_info.flag_byte.channel_id_output, false);
        assert_eq!(ext_info.flag_byte.rssi_output, false);
        assert_eq!(ext_info.flag_byte.timestamp_output, true);
        assert_eq!(ext_info.timestamp_output.unwrap().rx_timestamp, 0xAABB);

        let mut buf: [u8; 12] = [0; 12];
        let size = BurstTransferData::new(
            ChannelSequence::new(2.into(), 4.into()),
            [5, 6, 7, 8, 9, 0, 1, 2],
        )
        .serialize_message(&mut buf)
        .unwrap();
        assert_eq!(buf, [0x44, 5, 6, 7, 8, 9, 0, 1, 2, 0, 0, 0]);
        assert_eq!(size, 9);
    }

    #[test]
    fn advanced_burst_data() {
        let unpacked = AdvancedBurstData::unpack_from_slice(&[10, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
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
