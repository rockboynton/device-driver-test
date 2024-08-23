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
            // ByteOrder needs to be LE for some reason
            type ByteOrder = LE;
            const ADDRESS: u8 = 0;
            const SIZE_BITS: usize = 16;
            // Reset values need to be swapped
            const RESET_VALUE: [u8] = [0x70, 0x40];

            foo: bool = 6,
            bar: bool = 4,
            reset: bool = 1,
            powerdown: bool = 0,
        },

        register R76 {
            type RWType = R;
            type ByteOrder = LE;
            const ADDRESS: u8 = 76;
            const SIZE_BITS: usize = 16;
            const RESET_VALUE: u16 = 0;

            rb_temp_sens: u16 = 0..11,
        },
    }
);
