// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Ant message and drivers for Rust
//!
//! [![Crates.io][crates-badge]][crates-url]
//! [![Documentation](https://docs.rs/ant/badge.svg)](https://docs.rs/ant)
//! ![main](https://github.com/cujomalainey/ant/workflows/Rust/badge.svg)
//!
//! ## Introduction
//!
//! This crate provides a typed interface for communicating with nRF52 ANT stacks. The drivers also
//! abstract the messaging interface for any applications build on top. Therefore SPI, USB, Serial
//! and Softdevice communication methods can be interchanged with little to no changes in the stack
//! above. The code here is a complete rewrite of the ant-arduino C++ library.
//!
//! For documentation of the actual implementation of the ANT stack and what each of these
//! individual messages do please visit the ant website at [thisisant.com](https://www.thisisant.com/)
//!
//! ## Features
//!  * Support for Serial and USB communication (SPI and Softdevice are on the roadmap)
//!  * Support for all documented modern messages with optional fields
//!  * Byte transport is abstracted so any platform can be used
//!  * No direct heap usage when only using the drivers
//!
//! ## Roadmap
//!  * Softdevice support
//!  * SPI support
//!  * Large message configurable buffer maximums
//!  * USB support conditional compilation
//!  * Safe processing of data (no_panic)
//!  * Extended format support
//!     * ANT-FS support
//!  * no_std support
//!  * Provide hooks for user to parse unknown messages/formats
//!
//! TODO TX example via usb

pub mod drivers;
pub mod fields;
pub mod messages;
pub mod plus;
#[cfg(feature = "usb")]
pub mod usb;
