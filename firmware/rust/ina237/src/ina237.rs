// use emb_hal::blocking::i2c;

extern crate embedded_hal;

use embedded_hal::blocking::i2c;

use crate::types::Configuration;
use crate::types::ConfigurationRegisterValues;
use core::fmt;

pub struct Ina237<I2C> {
    i2c: I2C,
    configuration: Configuration,
}

enum Registers {
    Config = 0x00,
    AdcConfig = 0x01,
    ShuntCal = 0x02,
    VShunt = 0x04,
    VBus = 0x05,
    DieTemp = 0x06,
    Current = 0x07,
    Power = 0x08,
    DiagAlert = 0x09,
    SOVL = 0x0C,
    SUVL = 0x0D,
    BOVL = 0x0E,
    BUVOL = 0x0F,
    TempLimit = 0x10,
    PowerLimit = 0x11,
    ManufacturerId = 0x3E,
}

impl<I2C, E> Ina237<I2C>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
{
    pub fn new(i2c: I2C, configuration: Configuration) -> Ina237<I2C> {
        Ina237 {
            i2c: i2c,
            configuration: configuration,
        }
    }

    fn write_register(&self, register: Registers, data: &[u8; 2]) {

    }

    fn select_register(&self, register: Registers) {
        
    }

    fn read_register(&self, register: Registers) {
        self.select_register(register);

        
    }

    pub fn initialize(&self, configuration_register_values: ConfigurationRegisterValues) {

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate embedded_hal_mock;

    use embedded_hal_mock::{
        i2c::{Mock as I2cMock, Transaction as I2cTransaction},
        MockError,
    };

    // fn default_instance<I2C>(bus: I2cMock) -> Ina237<I2C>
    // where I2C: i2c::Write + i2c::Read {
    //     let configuration = Configuration::new(0x01, 2000);

    //     Ina237::new(bus, configuration)
    // }

    #[test]
    fn constructor_returns_struct() {
        let mut i2c = I2cMock::new([]);

        let configuration = Configuration::new(0x01, 2000);

        let under_test = Ina237::new(i2c, configuration);

        assert_eq!(0x01, under_test.configuration.address);
        assert_eq!(2000, under_test.configuration.shunt_cal);
        assert_eq!(i2c, under_test.i2c);
    }

    #[test]
    fn initialize_sets_calibration_register() {
        let mut i2c = I2cMock::new([]);

        let configuration = Configuration::new(0x01, 2000);

        let under_test = Ina237::new(i2c, configuration);

        // let under_test = default_instance(i2c);

        // assert_eq!(0x01, under_test.configuration.address);
        // assert_eq!(2000, under_test.configuration.shunt_cal);
        // assert_eq!(i2c, under_test.i2c);
    }
}
