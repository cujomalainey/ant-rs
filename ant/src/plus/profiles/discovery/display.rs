use crate::channel::{duration_to_search_timeout};
use crate::channel::{ChanError, RxHandler, TxHandler};
use crate::messages::config::{
    ChannelType, LibConfig, TransmissionType
};
use crate::messages::control::{CloseChannel, OpenRxScanMode};
use crate::messages::{AntMessage, TxMessage, TxMessageChannelConfig, TxMessageData};
// use crate::plus::common::datapages::MANUFACTURER_SPECIFIC_RANGE;
use crate::plus::common::msg_handler::{ChannelConfig, MessageHandler};
use crate::plus::profiles::fitness_equipment_controls::{
    Error, MonitorTxDataPage
};
use crate::plus::NETWORK_RF_FREQUENCY;

use std::time::Duration;

pub struct Display<T: TxHandler<TxMessage>, R: RxHandler<AntMessage>> {
    msg_handler: MessageHandler,
    rx_message_callback: Option<fn(&AntMessage)>,
    rx_datapage_callback: Option<fn(Result<MonitorTxDataPage, Error>)>,
    tx_message_callback: Option<fn() -> Option<TxMessageChannelConfig>>,
    tx_datapage_callback: Option<fn() -> Option<TxMessageData>>,
    tx: T,
    rx: R,
    is_scanning: bool,
}

pub struct DisplayConfig {
    pub channel: u8,
    pub ant_plus_key_index: u8,
}

impl<T: TxHandler<TxMessage>, R: RxHandler<AntMessage>> Display<T, R> {
    pub fn new(
        conf: DisplayConfig,
        tx: T,
        rx: R,
    ) -> Self {
        let channel_config = ChannelConfig {
            channel: conf.channel,
            device_number: 0,
            device_type: 0,
            channel_type: ChannelType::BidirectionalSlave,
            network_key_index: conf.ant_plus_key_index,
            transmission_type: TransmissionType::new_wildcard(),
            radio_frequency: NETWORK_RF_FREQUENCY,
            timeout_duration: duration_to_search_timeout(Duration::from_secs(30)),
            channel_period: 8192,
        };
        Self {
            rx_message_callback: None,
            rx_datapage_callback: None,
            tx_message_callback: None,
            tx_datapage_callback: None,
            msg_handler: MessageHandler::new(&channel_config),
            tx,
            rx,
            is_scanning: false,
        }
    }

    pub fn open(&mut self) {
        self.msg_handler.open();
    }

    pub fn close(&mut self) {
        self.msg_handler.close();
        self.is_scanning = false;
    }

    pub fn get_device_id(&self) -> u16 {
        self.msg_handler.get_device_id()
    }

    pub fn set_rx_message_callback(&mut self, f: Option<fn(&AntMessage)>) {
        self.rx_message_callback = f;
    }

    pub fn set_rx_datapage_callback(&mut self, f: Option<fn(Result<MonitorTxDataPage, Error>)>) {
        self.rx_datapage_callback = f;
    }

    pub fn set_tx_message_callback(&mut self, f: Option<fn() -> Option<TxMessageChannelConfig>>) {
        self.tx_message_callback = f;
    }

    pub fn set_tx_datapage_callback(&mut self, f: Option<fn() -> Option<TxMessageData>>) {
        self.tx_datapage_callback = f;
    }

    pub fn reset_state(&mut self) {
        // TODO
    }

    pub fn process(&mut self) -> Result<(), ChanError> {
        if self.msg_handler.is_tx_ready() && !self.is_scanning {  // Start scanning instead of opening channel
            self.tx.try_send(CloseChannel::new(self.msg_handler.get_channel()).into()).unwrap();
            self.tx.try_send(LibConfig::new(true, false, false).into()).unwrap();
            self.tx.try_send(OpenRxScanMode::new(Some(false)).into()).unwrap();
            self.is_scanning = true;
        }

        // TODO handle closed channel
        while let Ok(msg) = self.rx.try_recv() {
            if let Some(f) = self.rx_message_callback {
                f(&msg);
            }
            match self.msg_handler.receive_message(&msg) {
                Ok(_) => (),
                Err(e) => {
                    if let Some(f) = self.rx_datapage_callback {
                        f(Err(e.into()));
                    }
                }
            }
        }

        // TODO handle errors
        if let Some(msg) = self.msg_handler.send_message() {
            println!("Sending message: {:?}", msg);
            self.tx.try_send(msg)?;
        }
        if let Some(callback) = self.tx_message_callback {
            if let Some(mut msg) = callback() {
                msg.set_channel(self.msg_handler.get_channel());
                self.tx.try_send(msg.into())?;
            }
        }
        if self.msg_handler.is_tx_ready() {
            if let Some(callback) = self.tx_datapage_callback {
                if let Some(mut msg) = callback() {
                    msg.set_channel(self.msg_handler.get_channel());
                    self.msg_handler.tx_sent();
                    self.tx.try_send(msg.into())?;
                }
            }
        }
        Ok(())
    }
}