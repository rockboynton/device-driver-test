#![allow(missing_docs)]
use super::*;

device_driver::implement_device!(
    impl<Spi, InputPin> MyDriver<Spi, InputPin> 
    where
        Spi: spi::SpiDevice,
        InputPin: digital::InputPin,
    {
        register R0 {
            type RWType = RW;
            const ADDRESS: u8 = 0;
            const SIZE_BITS: usize = 16;
            const RESET_VALUE: [u8] = [0x70, 0x40];

            foo: bool = 6,
            bar: bool = 4,
            reset: bool = 1,
            powerdown: bool = 0,
        },
    }
);
