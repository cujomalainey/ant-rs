// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::messages::{AntMessage, TxMessage};
use core::time::Duration;

pub const fn duration_to_search_timeout(t: Duration) -> u8 {
    // Scale up by 10 to avoid floating point math as ratio is 2.5s to 1 count
    ((t.as_secs() * 10) / (25)) as u8
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelAssignment {
    Assigned(u8),
    UnAssigned(),
}

/// Channel is the trait all channels must implement to register with the router
pub trait Channel {
    /// All channels must be able to recieve messages and must be infalliable. If you have an
    /// error with a recieved message, deal with it internally, the router does not care.
    fn receive_message(&mut self, msg: &AntMessage);
    /// Yield the next message the profile wishes to send
    fn send_message(&mut self) -> Option<TxMessage>;
    /// Assign channel from associated router or manually if not using a router
    fn set_channel(&mut self, channel: ChannelAssignment);
}
