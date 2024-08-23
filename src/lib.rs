use device_driver::{bitvec::prelude::*, AddressableDevice, RegisterDevice};
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
        self.r_0().write(|w| w.foo(false).bar(false).reset(true))?;
        self.r_0().write(|w| w.foo(false).bar(false).reset(false))?;

        Ok(())
    }

    fn temp_transfer_function(temp_sensor_code: u16) -> f32 {
        0.85 * temp_sensor_code as f32 - 415.0
    }

    pub fn temp(&mut self) -> Result<f32, MyDriverError<SpiError, DigitalError>> {
        let temp_sensor_code = self.r_76().read()?.rb_temp_sens();
        let temp = Self::temp_transfer_function(temp_sensor_code);

        Ok(temp)
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
        let data = data.data;

        let command = [address, data[1], data[0]];

        self.spi.write(&command).map_err(MyDriverError::Spi)?;

        Ok(())
    }

    fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        let command = [
            // set the read bit
            address | (1 << 7),
            // empty data field
            0,
            0,
        ];

        let mut buf = [0; 3];
        self.spi
            .transfer(&mut buf, &command)
            .map_err(MyDriverError::Spi)?;

        let out_data = data.as_raw_mut_slice();

        out_data[0] = buf[2];
        out_data[1] = buf[1];

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

    #[test]
    fn temp() -> TestResult {
        let reg: u32 = 76 << 16;
        let mut reg_bytes: Vec<_> = reg.to_be_bytes()[1..].into();
        reg_bytes[0] |= 1 << 7;

        let spi_expectations = [
            mock::spi::Transaction::transaction_start(),
            // respond with sensor reading of 1000 (0x3E8)
            mock::spi::Transaction::transfer(reg_bytes, vec![0x00, 0x03, 0xE8]),
            mock::spi::Transaction::transaction_end(),
        ];

        let mut my_driver = new_mock_my_driver(&spi_expectations, &[])?;
        let temp = my_driver.temp()?;
        assert_eq!(temp, MockMyDriver::temp_transfer_function(0x03E8));
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
