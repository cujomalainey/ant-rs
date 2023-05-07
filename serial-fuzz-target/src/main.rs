#[macro_use]
extern crate afl;
extern crate ant;
use ant::drivers::{Driver, SerialDriver};
use embedded_hal::serial::{Read, Write};

struct SerialMock {
    data: Vec<u8>,
}

impl Write<u8> for SerialMock {
    type Error = u8;
    fn write(&mut self, _word: u8) -> nb::Result<(), Self::Error> {
        Ok(())
    }
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

impl Read<u8> for SerialMock {
    type Error = u8;
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        if self.data.len() > 0 {
            Ok(self.data.remove(0))
        } else {
            Err(nb::Error::WouldBlock)
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
