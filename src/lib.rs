use device_driver::{
    bitvec::{prelude::*, view::BitView},
    AddressableDevice, RegisterDevice,
};
use embedded_hal::{digital, spi};

pub mod registers;

#[derive(Debug, PartialEq, Eq)]
pub struct MyDriver<Spi, InputPin>
where
    Spi: spi::SpiDevice,
    InputPin: digital::InputPin,
{
    /// SPI peripheral
    spi: Spi,
    /// Chip Enable.
    ce: InputPin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MyDriverError<SpiError, DigitalError> {
    /// SPI error
    Spi(SpiError),
    /// Digital Error
    Digital(DigitalError),
}

impl<SpiError, DigitalError, Spi, InputPin> MyDriver<Spi, InputPin>
where
    Spi: spi::SpiDevice<Error = SpiError>,
    InputPin: digital::InputPin<Error = DigitalError>,
{
    pub const fn new(
        spi: Spi,
        ce: InputPin,
    ) -> Result<Self, MyDriverError<SpiError, DigitalError>> {
        Ok(Self { spi, ce })
    }

    pub fn destroy(self) -> (Spi, InputPin) {
        (self.spi, self.ce)
    }

    pub fn reset(&mut self) -> Result<(), MyDriverError<SpiError, DigitalError>> {
        // self.r_0()
        //     .write(|w| w.foo(false).bar(false).reset(true))?;
        // self.r_0()
        //     .write(|w| w.foo(false).bar(false).reset(false))?;

        self.r_0().write(|w| w.powerdown(true))?;

        // self.spi.write(&[0x00, 0x40, 0x22]).map_err(MyDriverError::Spi)?;
        // self.spi.write(&[0x00, 0x40, 0x20]).map_err(MyDriverError::Spi)?;

        Ok(())
    }
}

impl<Spi, InputPin> AddressableDevice for MyDriver<Spi, InputPin>
where
    Spi: spi::SpiDevice,
    InputPin: digital::InputPin,
{
    type AddressType = u8;
}

/// The protocol for reading/writing registers is as follows:
///
/// | 23 | 22 | 21 | 20 | 19 | 18 | 17 | 16 | 15 | 14 | 13 | 12 | 10 | 09 | 08 | 07 | 06 | 05 | 04 | 03 | 02 | 01 | 00 |
/// | R/!W |        `REG_ADDR`[7:0]         |                                       DATA[15:0]                         |
impl<SpiError, DigitalError, Spi, InputPin> RegisterDevice for MyDriver<Spi, InputPin>
where
    Spi: spi::SpiDevice<Error = SpiError>,
    InputPin: digital::InputPin<Error = DigitalError>,
{
    type Error = MyDriverError<SpiError, DigitalError>;

    fn write_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        println!(
            "{address:#02X?} {:#02X?} {:#02X?}",
            data.as_raw_slice()[0],
            data.as_raw_slice()[1]
        );
        println!("{data}");
        let command = [address, data.as_raw_slice()[0], data.as_raw_slice()[1]];

        self.spi.write(&command).map_err(MyDriverError::Spi)?;

        Ok(())
    }

    fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        let command = [
            address & (1 << 7),
            data.as_raw_slice()[1],
            data.as_raw_slice()[0],
        ];

        let mut buf = [0; 3];

        self.spi
            .transfer(&mut buf, &command)
            .map_err(MyDriverError::Spi)?;

        data.copy_from_bitslice(buf.view_bits());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{MyDriver, MyDriverError};
    use embedded_hal as hal;
    use embedded_hal_mock::common::Generic;
    use embedded_hal_mock::eh1 as mock;

    type TestResult = Result<(), MyDriverError<hal::spi::ErrorKind, mock::MockError>>;

    #[test]
    fn reset() -> TestResult {
        let spi_expectations = [
            mock::spi::Transaction::transaction_start(),
            mock::spi::Transaction::write_vec(vec![0x00, 0x40, 0x22]),
            mock::spi::Transaction::transaction_end(),
            mock::spi::Transaction::transaction_start(),
            mock::spi::Transaction::write_vec(vec![0x00, 0x40, 0x20]),
            mock::spi::Transaction::transaction_end(),
        ];

        let mut my_driver = new_mock_my_driver(&spi_expectations, &[])?;
        my_driver.reset()?;
        all_done(my_driver);

        Ok(())
    }

    type MockMyDriver =
        MyDriver<Generic<mock::spi::Transaction<u8>>, Generic<mock::digital::Transaction>>;

    fn new_mock_my_driver(
        spi_expectations: &[mock::spi::Transaction<u8>],
        ce_expectations: &[mock::digital::Transaction],
    ) -> Result<MockMyDriver, MyDriverError<hal::spi::ErrorKind, mock::MockError>> {
        let spi_mock = mock::spi::Mock::new(spi_expectations);
        let ce = mock::digital::Mock::new(ce_expectations);
        let my_driver = MyDriver::new(spi_mock, ce)?;
        Ok(my_driver)
    }

    fn all_done(my_driver: MockMyDriver) {
        let (mut spi_mock, mut ce) = my_driver.destroy();
        // verify expectations
        spi_mock.done();
        ce.done();
    }
}
