use embedded_hal::{delay::DelayNs, i2c::I2c};

use crate::registers::*;

use core::marker::PhantomData;

const I2C_ADDR : u8 = 0x2a;

pub struct Nau7802<I2C, D>
where I2C: I2c,
    D: DelayNs {
    i2c: I2C,
    // If we want to globally define the delay type for this struct, we have to consume the type
    // parameter.
    _delay: PhantomData<D>,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
#[non_exhaustive]
pub enum Error<E> {
    /// Initialization failed within the allotted timeframe.
    Initialize,
    /// Failed I2C communication.
    I2C(E),
}


impl<I2C, E, D> Nau7802<I2C, D>
where
    I2C: I2c<Error = E>,
    D: DelayNs {
    pub fn new(i2c: I2C) -> Nau7802<I2C, D> {
        Self {
            i2c: i2c,
            _delay: PhantomData,
        }
    }

    fn write_register(&mut self, register: Registers, value: u8) {
        let buffer : [u8;2] = [register as u8, value];

        self.i2c.write(I2C_ADDR, &buffer);
    }

    fn read_register(&mut self, register: Registers) -> u8 {
        let write_buffer : [u8;1] = [register as u8];
        let mut read_buffer : [u8;1] = [0;1];
        
        self.i2c.write_read(I2C_ADDR, &write_buffer, &mut read_buffer);

        read_buffer[0]
    }

    pub fn initialize(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        //RR to 1
        self.write_register(Registers::PU_CTRL, PU_CTRL::reset(true).into());
        //RR to 0 and PUD to 1
        self.write_register(Registers::PU_CTRL, PU_CTRL::reset(false).into());

        //After ~200ms, PWRUP should be 1
        let mut attempts = 50;
        loop {
            delay.delay_ms(20);

            let status = PU_CTRL::from(self.read_register(Registers::PU_CTRL));

            if status.PUR {
                break;
            } else {
                attempts -= 1;

                if attempts == 0 {
                    return Result::Err(Error::Initialize)
                }
            }
        }

        //Configure device
        let mut cfg = PU_CTRL::reset(false);
        cfg.AVDDS = true;
        cfg.OSCS = false;
        cfg.PUA = true;
        cfg.PUD = true;
        self.write_register(Registers::PU_CTRL, cfg.into());

        //Start conversions with CS = 1
        let mut cfg = PU_CTRL::reset(false);
        cfg.AVDDS = true;
        cfg.OSCS = false;
        cfg.PUA = true;
        cfg.PUD = true;
        
        cfg.CS = true;
        self.write_register(Registers::PU_CTRL, cfg.into());
        
        Result::Ok(())
    }
}