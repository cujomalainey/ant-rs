#[macro_use]
extern crate afl;
extern crate ant;
use ant::drivers::{Driver, SerialDriver};
use core::convert::Infallible;
use embedded_hal_nb::serial::{ErrorType, Read, Write};

struct SerialMock {
    data: Vec<u8>,
}

impl ErrorType for SerialMock {
    type Error = Infallible;
}

impl Write<u8> for SerialMock {
    fn write(&mut self, _word: u8) -> nb::Result<(), Self::Error> {
        Ok(())
    }
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

impl Read<u8> for SerialMock {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        if self.data.is_empty() {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(self.data.remove(0))
        }
    }
}

fn main() {
    fuzz!(|data: &[u8]| {
        let mut buf = Vec::new();
        buf.extend_from_slice(data);
        let mock = SerialMock { data: buf };
        let mut driver: SerialDriver<SerialMock, ant::drivers::StubPin> =
            ant::drivers::SerialDriver::new(mock, None);
        while driver.get_message() != Ok(None) {}
    });
}
