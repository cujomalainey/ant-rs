// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use const_utils::u64::min;
use core::time::Duration;

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
pub enum TxError<T> {
    Full(T),
    Closed(T),
    UnknownError,
}

pub trait TxHandler<T> {
    // TODO async versions
    fn try_send(&self, msg: T) -> Result<(), TxError<T>>;
}

pub trait RxHandler<T> {
    // TODO async versions
    fn try_recv(&self) -> Result<T, RxError>;
}
