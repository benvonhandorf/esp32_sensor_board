// use emb_hal::blocking::i2c;

extern crate embedded_hal;

use embedded_hal::i2c;

use crate::types::Configuration;
use crate::types::ConfigurationRegisterValues;
use crate::types::Measurement;
use core::fmt;

pub struct Ina237<I2C> {
    i2c: I2C,
    configuration: Configuration,
}

pub struct Error {

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
    I2C: i2c::I2c<Error = E>,
    E: i2c::Error
{
    pub fn new(i2c: I2C, configuration: Configuration) -> Ina237<I2C> {
        Ina237 {
            i2c: i2c,
            configuration: configuration,
        }
    }

    pub fn destroy(self) -> I2C {
        self.i2c
    }

    fn write_register(&mut self, register: Registers, data: &[u8; 2]) {
        let buffer: [u8; 3] = [register as u8, data[0], data[1]];

        self.i2c.write(self.configuration.addr(), &buffer).unwrap_or_else(|error| {
            panic!("i2c write failed: {}", error.kind());
        });
    }

    fn read_register(&mut self, register: Registers) -> [u8; 2] {
        let write_buffer: [u8; 1] = [register as u8];
        let mut read_buffer: [u8; 2] = [0x00 ; 2];

        self.i2c.write_read(self.configuration.addr(), &write_buffer, &mut read_buffer).unwrap_or_else(|error|  {
            panic!("i2c write_read failed: {}", error.kind());
        });
        
        read_buffer
    }

    pub fn initialize(&mut self, configuration_register_values: ConfigurationRegisterValues) {
        let data = self.configuration.shunt().to_be_bytes();

        self.write_register(Registers::ShuntCal, &data);

        let data = configuration_register_values.into_configuration().to_be_bytes();

        self.write_register(Registers::Config, &data);

        let data = configuration_register_values.into_adc_configuration().to_be_bytes();

        self.write_register(Registers::AdcConfig, &data);
    }

    pub fn configuration(&mut self) -> u16 {
        let result = self.read_register(Registers::Config);

        u16::from_be_bytes(result)
    }

    pub fn adc_configuration(&mut self) -> u16 {
        let result = self.read_register(Registers::AdcConfig);

        u16::from_be_bytes(result)
    }

    pub fn manufacturer_id(&mut self) -> u16 {
        let result = self.read_register(Registers::ManufacturerId);

        u16::from_be_bytes(result)
    }

    pub fn shunt_cal(&mut self) -> u16 {
        let result = self.read_register(Registers::ShuntCal);

        u16::from_be_bytes(result)
    }
    
    pub fn read(&mut self) -> Result<Measurement, Error> {
        let vbus_reading = self.read_register(Registers::VBus);

        let shunt_reading = self.read_register(Registers::VShunt);

        let current_reading = self.read_register(Registers::Current);

        let dietemp_reading = self.read_register(Registers::DieTemp);

        Result::Ok(Measurement::from_readings(i16::from_be_bytes(vbus_reading),
        i16::from_be_bytes(shunt_reading),
         i16::from_be_bytes(current_reading), 
         i16::from_be_bytes(dietemp_reading)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate embedded_hal_mock;

    use embedded_hal_mock::eh1:: {
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
        let i2c = I2cMock::new([]);

        let configuration = Configuration::new(0x01, 2000);

        let under_test = Ina237::new(i2c, configuration);

        assert_eq!(0x01, under_test.configuration.addr());
        // assert_eq!(2000, under_test.configuration.shunt_cal);

        let mut i2c = under_test.destroy();

        i2c.done()
    }

    #[test]
    fn initialize_sets_calibration_register() {
        let i2c = I2cMock::new([]);

        let configuration = Configuration::new(0x01, 2000);

        let under_test = Ina237::new(i2c, configuration);

        // let under_test = default_instance(i2c);

        // assert_eq!(0x01, under_test.configuration.address);
        // assert_eq!(2000, under_test.configuration.shunt_cal);
        // assert_eq!(i2c, under_test.i2c);

        let mut i2c = under_test.destroy();

        i2c.done()

    }
}
