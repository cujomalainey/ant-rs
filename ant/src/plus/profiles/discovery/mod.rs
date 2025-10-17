//! Based off V5 of the Fitness Equipment specification

// mod datapages;
mod display;

// pub use datapages::*;
pub use display::*;

// use crate::plus::common::datapages::{ModeSettings, RequestDataPage};
use crate::plus::common::msg_handler::StateError;

#[derive(Debug, Clone)]
pub enum Error {
    BytePatternError(packed_struct::PackingError),
    UnsupportedDataPage(u8),
    PageAlreadyPending(),
    NotAssociated(),
    ConfigurationError(StateError),
}

impl From<packed_struct::PackingError> for Error {
    fn from(err: packed_struct::PackingError) -> Self {
        Self::BytePatternError(err)
    }
}

impl From<StateError> for Error {
    fn from(err: StateError) -> Self {
        Self::ConfigurationError(err)
    }
}
