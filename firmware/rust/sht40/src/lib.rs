#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use embedded_hal::i2c::I2c;




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
