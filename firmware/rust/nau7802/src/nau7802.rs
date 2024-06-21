use embedded_hal::{delay::DelayNs, digital::InputPin, i2c::{self, I2c}};

use crate::registers::*;

use core::marker::PhantomData;

const I2C_ADDR : u8 = 0x2a;

pub struct Nau7802<I2C, D>
where I2C: I2c,
    // DRP: InputPin,
    D: DelayNs {
    i2c: I2C,
    // data_ready_pin: Option<DRP>,
    // If we want to globally define the delay type for this struct, we have to consume the type
    // parameter.
    _delay: PhantomData<D>,
}

// #[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
#[non_exhaustive]
pub enum Error<E>
where E: i2c::Error {
    /// Initialization failed within the allotted timeframe.
    Initialize,
    InitializeUnknownRevision(u8),
    InitializeNoPowerup(u8),
    NoDataReadyPin,
    /// Failed I2C communication.
    I2C(E),
    
}

pub enum AdcChannel {
    A,
    B,
}

impl<I2C, E, D> Nau7802<I2C, D>
where
    I2C: I2c<Error = E>,
    E: i2c::Error,
    // DRP: InputPin,
    D: DelayNs, {
    pub fn new(i2c: I2C) -> Nau7802<I2C, D> {
        Self {
            i2c: i2c,
            // data_ready_pin: data_ready_pin,
            _delay: PhantomData,
        }
    }

    fn write_register(&mut self, register: Registers, value: u8) -> Result<(), E> {
        let buffer : [u8;2] = [register as u8, value];

        match self.i2c.write(I2C_ADDR, &buffer) {
            Ok(_) => Result::Ok(()),
            Err(e) => Result::Err(e),
        }
    }

    fn read_register(&mut self, register: Registers) -> Result<u8, E> {
        let write_buffer : [u8;1] = [register as u8];
        let mut read_buffer : [u8;1] = [0;1];

        self.i2c.write(I2C_ADDR, &write_buffer).unwrap();
        
        match self.i2c.read(I2C_ADDR, &mut read_buffer) {
            Ok(_) => Result::Ok(read_buffer[0]),
            Err(e) => Result::Err(e),
        }
    }

    pub fn initialize(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        //RR to 1
        self.write_register(Registers::PU_CTRL, PU_CTRL::reset(true).into()).unwrap();

        let mut cfg = PU_CTRL::reset(false);
        cfg.PUD = true;
        //RR to 0 and PUD to 1
        self.write_register(Registers::PU_CTRL, cfg.into()).unwrap();

        //After ~200ms, PWRUP should be 1
        let mut attempts = 50;
        loop {
            delay.delay_ms(20);

            let status = PU_CTRL::from(self.read_register(Registers::PU_CTRL).unwrap());

            if status.PUR {
                break;
            } else {
                attempts -= 1;

                if attempts == 0 {
                    return Result::Err(Error::InitializeNoPowerup(status.into()))
                }
            }
        }

        //Configure device
        let mut cfg = PU_CTRL::reset(false);
        cfg.AVDDS = true;
        cfg.OSCS = false;
        cfg.PUA = true;
        cfg.PUD = true;
        self.write_register(Registers::PU_CTRL, cfg.into()).unwrap();

        //Start conversions with CS = 1
        let mut cfg = PU_CTRL::reset(false);
        cfg.CS = true;
        cfg.OSCS = false;
        cfg.PUA = true;
        cfg.PUD = true;
        
        cfg.CS = true;
        self.write_register(Registers::PU_CTRL, cfg.into()).unwrap();
        
        Result::Ok(())
    }

    // pub fn is_data_ready(&mut self) -> Result<bool, Error<E>> {
    //     match self.data_ready_pin {
    //         None => Result::Err(Error::NoDataReadyPin),
    //         Some(_) => {
    //             let pin = self.data_ready_pin.as_mut().unwrap();

    //             match pin.is_high() {
    //                 Ok(result) => Result::Ok(result),
    //                 Err(e) => Result::Err(Error::Initialize),
    //             }
    //         }
    //     }
    // }

    pub fn is_data_ready(&mut self) -> Result<bool, Error<E>> {
        match self.read_register(Registers::PU_CTRL) {
            Ok(v) => Result::Ok(v & 0x20 != 0),
            Err(e) => Result::Err(Error::I2C(e)),
        }
    }

    pub fn revision_id(&mut self) -> Result<u8, Error<E>> {
        match self.read_register(Registers::DEVICE_REVISION) {
            Ok(v) => Result::Ok(v & 0x0F),
            Err(e) => Result::Err(Error::I2C(e)),
        }
    }

    pub fn ctrl2(&mut self) -> Result<CTRL2, Error<E>> {
        Result::Ok(CTRL2::from(self.read_register(Registers::CTRL2).unwrap()))
    }

    pub fn select_channel(&mut self, adc_channel: AdcChannel) -> Result<bool, Error<E>> {
        let mut ctrl2 = self.ctrl2().unwrap();

        let updated_ctrl2 = match adc_channel {
            AdcChannel::A => if ctrl2.Channel2Selected {
                ctrl2.Channel2Selected = false;
                Some(ctrl2)
            } else {
                None
            }
            AdcChannel::B => if ctrl2.Channel2Selected {
                None
            } else {
                ctrl2.Channel2Selected = false;
                Some(ctrl2)
            }
        };

        match updated_ctrl2 {
            Some(v) => {
                self.write_register(Registers::CTRL2, v.into()).unwrap();
                Result::Ok(true)
            },
            None => Result::Ok(false),
        }
    }

    pub fn read_adc(&mut self) -> Result<u32, Error<E>> {
        let write_buffer : [u8;1] = [Registers::ADCO_B2 as u8];
        let mut read_buffer : [u8;3] = [0;3];
        
        match self.i2c.write_read(I2C_ADDR, &write_buffer, &mut read_buffer) {
            Ok(_) => {
                let val: u32  = (read_buffer[0] as u32) << 16
                    | (read_buffer[1] as u32) << 8
                    | (read_buffer[2] as u32) << 0;

                Result::Ok(val)
            },
            Err(e) => Result::Err(Error::I2C(e)),
        }
    }
}