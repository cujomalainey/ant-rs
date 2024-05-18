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

// TODO add a send and get response
//
// Logically since this is single threaded, if we send and recieve in the same call, all
// messages that may come inbetween send and recieve have no consequence on the code flow. The
// only challenge will be handling ownership since we will likely be holding the sender in a
// mutable state and if they recieve another message it will be a problem
