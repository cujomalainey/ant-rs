use std::time::{Duration, Instant};

use ant::channel::{RxError, RxHandler, TxError, TxHandler};
use ant::drivers::{is_ant_usb_device_from_device, UsbDriver};
use ant::messages::channel::MessageCode;
use ant::messages::config::SetNetworkKey;
use ant::messages::RxMessage;
use ant::plus::profiles::fitness_equipment_controls::{Display, DisplayConfig, Period};
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
    let (router_tx, channel_rx) = channel(8);

    let mut router = Router::new(
        driver,
        RxReceiver {
            receiver: router_rx,
        },
    )
    .unwrap();
    let snk = SetNetworkKey::new(0, [0xB9, 0xA5, 0x21, 0xFB, 0xBD, 0x72, 0xC3, 0x45]);
    router.send(&snk).expect("failed to set network key");
    let chan = router
        .add_channel(TxSender { sender: router_tx })
        .expect("Add channel failed");
    let config = DisplayConfig {
        device_number: 0,
        device_number_extension: 0.into(),
        channel: chan,
        period: Period::FourHz,
        ant_plus_key_index: 0,
    };
    let mut tacx = Display::new(
        config,
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
            _ => {}
        }
    }));

    tacx.open();

    let mut last_send_time = Instant::now();
    let mut resistance_multiplier = 2;

    loop {
        router.process().unwrap();
        tacx.process().unwrap();

        // Every 5 seconds, increase the target power by 50 watts
        if last_send_time.elapsed() >= Duration::from_secs(5) {
            let target_power = 50 * resistance_multiplier;
            println!("Setting power target to {} watts", target_power);

            if let Err(_) = tacx.set_power_target(target_power) {
                println!("Failed to send power target");
            }

            resistance_multiplier += 1;
            last_send_time = Instant::now();
        }
    }
}
