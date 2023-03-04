// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::time::Duration;

pub const NETWORK_RF_FREQUENCY: u8 = 57;

pub const fn duration_to_search_timeout(t: Duration) -> u8 {
    // Scale up by 10 to avoid floating point math as ratio is 2.5s to 1 count
    return ((t.as_secs() * 10) / (25)) as u8;
}

pub mod common_datapages;
pub mod profiles;
pub mod router;
