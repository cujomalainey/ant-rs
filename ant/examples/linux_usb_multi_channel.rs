use std::cell::OnceCell;
use std::sync::OnceLock;
use std::time::Instant;

use ant::channel::{RxError, RxHandler, TxError, TxHandler};
use ant::drivers::{is_ant_usb_device_from_device, UsbDriver};
use ant::messages::channel::MessageCode;
use ant::messages::config::{LibConfig, SetNetworkKey};
use ant::messages::data::BroadcastData;
use ant::messages::test_mode::CwTest;
use ant::messages::{AntMessage, RxMessage, TxMessage};
use ant::plus::profiles::{discovery, fitness_equipment_controls, speed_and_cadence};
use ant::router::Router;
use dialoguer::Select;
use rusb::{Device, DeviceList};

use thingbuf::mpsc::errors::{TryRecvError, TrySendError};
use thingbuf::mpsc::{channel, Receiver, Sender};

struct TxSender<T> {
    sender: Sender<T>,
}

struct RxReceiver<T> {
    receiver: Receiver<T>,
}

impl<T: Default + Clone> TxHandler<T> for TxSender<T> {
    fn try_send(&self, msg: T) -> Result<(), TxError> {
        match self.sender.try_send(msg) {
            Ok(_) => Ok(()),
            Err(TrySendError::Full(_)) => Err(TxError::Full),
            Err(TrySendError::Closed(_)) => Err(TxError::Closed),
            Err(_) => Err(TxError::UnknownError),
        }
    }
}

impl<T: Default + Clone> RxHandler<T> for RxReceiver<T> {
    fn try_recv(&self) -> Result<T, RxError> {
        match self.receiver.try_recv() {
            Ok(e) => Ok(e),
            Err(TryRecvError::Empty) => Err(RxError::Empty),
            Err(TryRecvError::Closed) => Err(RxError::Closed),
            Err(_) => Err(RxError::UnknownError),
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut devices: Vec<Device<_>> = DeviceList::new()
        .expect("Unable to lookup usb devices")
        .iter()
        .filter(|x| is_ant_usb_device_from_device(x))
        .collect();

    if devices.is_empty() {
        panic!("No devices found");
    }

    let device = if devices.len() == 1 {
        devices.remove(0)
    } else {
        let selection = Select::new()
            .with_prompt("Multiple devices found, please select a radio to use.")
            .items(
                &devices
                    .iter()
                    .map(|x| x.device_descriptor().unwrap())
                    .map(|x| format!("{:04x}:{:04x}", x.vendor_id(), x.product_id()))
                    .collect::<Vec<String>>(),
            )
            .interact()
            .expect("Selection failed");
        devices.remove(selection)
    };

    let driver = UsbDriver::new(device).unwrap();

    let (channel_tx, router_rx) = channel(8);

    let mut router = Router::new(
        driver,
        RxReceiver {
            receiver: router_rx,
        },
    )
    .unwrap();
    let snk = SetNetworkKey::new(0, [0xB9, 0xA5, 0x21, 0xFB, 0xBD, 0x72, 0xC3, 0x45]);
    router.send(&snk).expect("failed to set network key");

    let mut discovery = setup_discovery_channel(&mut router, channel_tx.clone());
    let mut tacx: Option<fitness_equipment_controls::Display<TxSender<TxMessage>, RxReceiver<AntMessage>>> = None;
    // let mut tacx2 = setup_sac_channel(&mut router, channel_tx.clone());

    router.send(&LibConfig::new(true, false, false)).unwrap();

    let mut last_time = Instant::now();
    let mut started = false;

    loop {
        let _ = router.process();
        discovery.process().unwrap();
        if let Some(tacx) = &mut tacx {
            tacx.process().unwrap();
        }
        // tacx2.process().unwrap();

        let elapsed = last_time.elapsed();
        if elapsed.as_secs() > 10 && !started {
            tacx = Some(setup_fec_channel(&mut router, channel_tx.clone()));
            started = true;
        }
    }
}

fn setup_fec_channel(
    router: &mut Router<rusb::Error, UsbDriver<rusb::GlobalContext>, 
    TxSender<AntMessage>, RxReceiver<TxMessage>>, channel_tx: Sender<TxMessage>
) -> fitness_equipment_controls::Display<TxSender<TxMessage>, RxReceiver<AntMessage>> {
    let (router_tx, channel_rx) = channel(8);
    let chan = router
        .add_channel(TxSender { sender: router_tx })
        .expect("Add channel failed");
    let tacx_config = fitness_equipment_controls::DisplayConfig {
        device_number: 9609,
        device_number_extension: 0.into(),
        channel: chan,
        period: fitness_equipment_controls::Period::FourHz,
        ant_plus_key_index: 0,
    };
    let mut tacx = fitness_equipment_controls::Display::new(
        tacx_config,
        TxSender { sender: channel_tx },
        RxReceiver { receiver: channel_rx },
    );
    tacx.set_rx_message_callback(Some(|msg| {
        println!("{:#?}", msg);
        // match msg.message {
        //     RxMessage::ChannelEvent(event) => match event.payload.message_code {
        //         MessageCode::EventTransferTxCompleted => println!("Transfer TX completed"),
        //         MessageCode::EventTransferTxFailed => println!("Transfer TX failed"),
        //         _ => {}
        //     },
        //     RxMessage::BroadcastData(x) =>
        //         println!("17: {:x?}", x.payload.channel_number),
        //     _ => {}
        // }
    }));

    tacx.open();

    tacx
}

fn setup_sac_channel(
    router: &mut Router<rusb::Error, UsbDriver<rusb::GlobalContext>, 
    TxSender<AntMessage>, RxReceiver<TxMessage>>, channel_tx: Sender<TxMessage>
) -> speed_and_cadence::Display<TxSender<TxMessage>, RxReceiver<AntMessage>> {
    let (router_tx, channel_rx) = channel(8);
    let chan = router
        .add_channel(TxSender { sender: router_tx })
        .expect("Add channel failed");
    let tacx_config = speed_and_cadence::DisplayConfig {
        device_number: 0,
        device_number_extension: 0.into(),
        channel: chan,
        period: speed_and_cadence::Period::FourHz,
        ant_plus_key_index: 0,
    };
    let mut tacx = speed_and_cadence::Display::new(
        tacx_config,
        TxSender { sender: channel_tx },
        RxReceiver { receiver: channel_rx },
    );
    tacx.set_rx_message_callback(Some(|msg| {
        match msg.message {
            RxMessage::ChannelEvent(event) => match event.payload.message_code {
                MessageCode::EventTransferTxCompleted => println!("Transfer TX completed"),
                MessageCode::EventTransferTxFailed => println!("Transfer TX failed"),
                _ => {}
            },
            RxMessage::BroadcastData(x) =>
                println!("11: {:x?}", x.payload.channel_number),
            _ => {}
        }
    }));

    tacx.open();

    tacx
}

fn setup_discovery_channel(
    router: &mut Router<rusb::Error, UsbDriver<rusb::GlobalContext>, 
    TxSender<AntMessage>, RxReceiver<TxMessage>>, channel_tx: Sender<TxMessage>
) -> discovery::Display<TxSender<TxMessage>, RxReceiver<AntMessage>> {
    let (router_tx, channel_rx) = channel(8);
    let chan = router
        .add_channel(TxSender { sender: router_tx })
        .expect("Add channel failed");
    let tacx_config = discovery::DisplayConfig {
        channel: chan,
        ant_plus_key_index: 0,
    };
    let mut tacx = discovery::Display::new(
        tacx_config,
        TxSender { sender: channel_tx },
        RxReceiver { receiver: channel_rx },
    );
    tacx.set_rx_message_callback(Some(|msg| {
        // println!("{:#?}", msg);
        match msg.message {
            RxMessage::ChannelEvent(event) => match event.payload.message_code {
                MessageCode::EventTransferTxCompleted => println!("Transfer TX completed"),
                MessageCode::EventTransferTxFailed => println!("Transfer TX failed"),
                _ => {}
            },
            RxMessage::BroadcastData(x) => {
                if x.payload.data[0] == 80 || x.payload.data[0] == 81 {
                    println!("{:#?}", x);
                } else {
                    println!("discovered");
                }
            },
            RxMessage::SerialNumber(x) => 
                println!("Serial Number: {:#?}", x),
            _ => {}
        }
    }));

    tacx.open();

    tacx
}