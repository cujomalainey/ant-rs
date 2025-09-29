// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use const_utils::u64::min;
use core::time::Duration;

// TODO move this somewhere more appropriate
/// Helper to convert durations to search timeouts.
/// Anything greater than or equal to 637.5s will default to inifinite timeout per ANT spec.
pub const fn duration_to_search_timeout(t: Duration) -> u8 {
    // Scale up by 10 to avoid floating point math as ratio is 2.5s to 1 count
    min((t.as_secs() * 10) / (25), 255) as u8
}

#[derive(Clone, Debug)]
pub enum RxError {
    Empty,
    Closed,
    UnknownError,
}

#[derive(Clone, Debug)]
pub enum TxError {
    Full,
    Closed,
    UnknownError,
}

#[derive(Clone, Debug)]
pub enum ChanError {
    Rx(RxError),
    Tx(TxError),
}

impl From<RxError> for ChanError {
    fn from(err: RxError) -> ChanError {
        ChanError::Rx(err)
    }
}

impl From<TxError> for ChanError {
    fn from(err: TxError) -> ChanError {
        ChanError::Tx(err)
    }
}

pub trait TxHandler<T> {
    // TODO async versions
    fn try_send(&self, msg: T) -> Result<(), TxError>;
}

pub trait RxHandler<T> {
    // TODO async versions
    fn try_recv(&self) -> Result<T, RxError>;
}

#[cfg(feature = "std")]
pub mod mpsc {
    use super::*;
    use std::sync::mpsc::{Receiver, Sender};

    /// Abstraction implementation for std::sync::mpsc::Receiver
    ///
    /// Uses non-blocking calls
    pub struct RxChannel<T> {
        pub receiver: Receiver<T>,
    }

    /// Abstraction implementation for std::sync::mpsc::Receiver
    ///
    /// Uses blocking calls
    pub struct BlockingRxChannel<T> {
        pub receiver: Receiver<T>,
    }

    /// Abstraction implementation for std::sync::mpsc::Sender
    pub struct TxChannel<T> {
        pub sender: Sender<T>,
    }

    impl<T> TxHandler<T> for TxChannel<T> {
        fn try_send(&self, msg: T) -> Result<(), TxError> {
            match self.sender.send(msg) {
                Ok(_) => Ok(()),
                Err(_) => Err(TxError::Closed),
            }
        }
    }

    impl<T> RxHandler<T> for RxChannel<T> {
        fn try_recv(&self) -> Result<T, RxError> {
            match self.receiver.try_recv() {
                Ok(m) => Ok(m),
                Err(_) => Err(RxError::Closed),
            }
        }
    }

    impl<T> RxHandler<T> for BlockingRxChannel<T> {
        fn try_recv(&self) -> Result<T, RxError> {
            match self.receiver.recv() {
                Ok(m) => Ok(m),
                Err(_) => Err(RxError::Closed),
            }
        }
    }
}
