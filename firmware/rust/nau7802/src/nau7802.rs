use embedded_hal::{
    delay::DelayNs,
    // digital::InputPin, //TODO: v2 will use InputPin for conversion ready notification
    i2c::{self, I2c},
};

use crate::registers::*;

use core::marker::PhantomData;

const I2C_ADDR: u8 = 0x2a;

pub struct Nau7802<I2C, D>
where
    I2C: I2c,
    // DRP: InputPin,
    D: DelayNs,
{
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
where
    E: i2c::Error,
{
    /// Initialization failed within the allotted timeframe.
    Initialize,
    InitializeUnknownRevision(u8),
    InitializeNoPowerup(u8),
    NoDataReadyPin,
    DataNotReady,
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
    D: DelayNs,
{
    pub fn new(i2c: I2C) -> Nau7802<I2C, D> {
        Self {
            i2c: i2c,
            // data_ready_pin: data_ready_pin,
            _delay: PhantomData,
        }
    }

    fn write_register(&mut self, register: &Registers, value: &u8) -> Result<(), E> {
        let buffer: [u8; 2] = [*register as u8, *value];

        match self.i2c.write(I2C_ADDR, &buffer) {
            Ok(_) => Result::Ok(()),
            Err(e) => Result::Err(e),
        }
    }

    fn read_register(&mut self, register: Registers) -> Result<u8, E> {
        let write_buffer: [u8; 1] = [register as u8];
        let mut read_buffer: [u8; 1] = [0; 1];

        self.i2c.write(I2C_ADDR, &write_buffer).unwrap();

        match self.i2c.read(I2C_ADDR, &mut read_buffer) {
            Ok(_) => Result::Ok(read_buffer[0]),
            Err(e) => Result::Err(e),
        }
    }

    pub fn initialize(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        //RR to 1
        self.pu_ctrl_write(&&PU_CTRL::reset(true)).unwrap();

        let mut cfg = PU_CTRL::reset(false);
        cfg.PUD = true;
        //RR to 0 and PUD to 1
        self.pu_ctrl_write(&cfg).unwrap();

        //After ~200ms, PWRUP should be 1
        let mut attempts = 50;
        loop {
            delay.delay_ms(20);

            let status = self.pu_ctrl().unwrap();

            if status.PUR {
                break;
            } else {
                attempts -= 1;

                if attempts == 0 {
                    return Result::Err(Error::InitializeNoPowerup(status.into()));
                }
            }
        }

        //Configure device
        let mut cfg = PU_CTRL::reset(false);
        cfg.AVDDS = true;
        cfg.OSCS = false;
        cfg.PUA = true;
        cfg.PUD = true;
        self.pu_ctrl_write(&cfg).unwrap();

        //Start conversions with CS = 1
        let mut cfg = PU_CTRL::reset(false);
        cfg.CS = true;
        cfg.OSCS = false;
        cfg.PUA = true;
        cfg.PUD = true;

        cfg.CS = true;
        self.pu_ctrl_write(&cfg).unwrap();

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

    fn pu_ctrl(&mut self) -> Result<PU_CTRL, Error<E>> {
        Result::Ok(PU_CTRL::from(self.read_register(Registers::PU_CTRL).unwrap()))
    }

    fn pu_ctrl_write(&mut self, pu_ctrl: &PU_CTRL) -> Result<(), Error<E>> {
        match self.write_register(&Registers::PU_CTRL, &((*pu_ctrl).into())) {
            Ok(_) => Result::Ok(()),
            Err(e) => Result::Err(Error::I2C(e))
        }
    }

    pub fn ctrl2(&mut self) -> Result<CTRL2, Error<E>> {
        Result::Ok(CTRL2::from(self.read_register(Registers::CTRL2).unwrap()))
    }

    fn ctrl2_write(&mut self, ctrl2: &CTRL2) -> Result<(), Error<E>> {
        match self.write_register(&Registers::CTRL2, &((*ctrl2).into())) {
            Ok(_) => Result::Ok(()),
            Err(e) => Result::Err(Error::I2C(e))
        }
    }

    pub fn ctrl1(&mut self) -> Result<CTRL1, Error<E>> {
        Result::Ok(CTRL1::from(self.read_register(Registers::CTRL1).unwrap()))
    }

    fn ctrl1_write(&mut self, ctrl1: CTRL1) -> Result<(), Error<E>> {
        match self.write_register(&Registers::CTRL1, &ctrl1.into()) {
            Ok(_) => Result::Ok(()),
            Err(e) => Result::Err(Error::I2C(e))
        }
    }

    pub fn enable_ldo(&mut self) -> Result<(), Error<E>> {
        let mut pu_ctrl = self.pu_ctrl().unwrap();

        pu_ctrl.AVDDS = true;

        self.pu_ctrl_write(&pu_ctrl)
    }

    pub fn set_ldo_voltage(&mut self, ldo_voltage: LdoVoltage) -> Result<(), Error<E>> {
        let mut ctrl1 = self.ctrl1().unwrap();

        ctrl1.ldo_voltage = ldo_voltage;

        self.ctrl1_write(ctrl1)
    }

    pub fn set_gain(&mut self, gain: Gains) -> Result<(), Error<E>> {
        let mut ctrl1 = self.ctrl1().unwrap();

        ctrl1.gain_select = gain;

        self.ctrl1_write(ctrl1)
    }

    fn wait_for_calibration_completion(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        //Wait for calibration register to read as 0
        let mut attempts = 50;
        loop {
            delay.delay_ms(20);

            let ctrl2 = self.ctrl2().unwrap();

            if ctrl2.calibrate {
                attempts -= 1;

                if attempts == 0 {
                    return Result::Err(Error::Initialize);
                }

                delay.delay_ms(10);
            } else {
                break;
            }
        }

        Result::Ok(())
    }

    pub fn calibrate(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        let mut ctrl2 = self.ctrl2().unwrap();

        ctrl2.calibrate = true;

        self.ctrl2_write(&ctrl2).unwrap();

        self.wait_for_calibration_completion(delay)
    }

    pub fn select_channel(&mut self, adc_channel: AdcChannel, delay: &mut D) -> Result<bool, Error<E>> {
        let ctrl2 = self.ctrl2().unwrap();

        let updated_ctrl2 = match adc_channel {
            AdcChannel::A => {
                if ctrl2.channel2_selected {
                    let mut v = ctrl2.clone();
                    v.channel2_selected = false;
                    v.calibrate = true;
                    Some(v)
                } else {
                    None
                }
            }
            AdcChannel::B => {
                if ctrl2.channel2_selected {
                    None
                } else {
                    let mut v = ctrl2.clone();
                    v.channel2_selected = true;
                    v.calibrate = true;
                    Some(v)
                }
            }
        };

        match updated_ctrl2 {
            Some(v) => {
                self.ctrl2_write(&v).unwrap();

                self.wait_for_calibration_completion(delay).unwrap();

                Result::Ok(true)
            }
            None => Result::Ok(false),
        }
    }

    ///Converts 3 i24 bytes into an i32
    fn i32_from_i24_be_bytes(b: &[u8; 3]) -> i32 {
        let mut i32_bytes: [u8; 4] = [0x00; 4];
        i32_bytes[0..3].copy_from_slice(b);

        i32::from_be_bytes(i32_bytes) >> 8
    }

    fn i32_to_i24_be_bytes(v: i32) -> [u8; 3] {
        let b = v.to_be_bytes();
        [b[0], b[1], b[2]]
    }

    pub fn set_adc_offset(&mut self, adc_channel: AdcChannel, offset: i32) -> Result<(), Error<E>> {
        let b = Self::i32_to_i24_be_bytes(offset);

        let registers = match adc_channel {
            AdcChannel::A => [
                Registers::OCAL1_B0,
                Registers::OCAL1_B1,
                Registers::OCAL1_B2,
            ],
            AdcChannel::B => [
                Registers::OCAL2_B0,
                Registers::OCAL2_B1,
                Registers::OCAL2_B2,
            ],
        };

        registers.iter().zip(b.iter()).for_each(|(r, b)| {
            self.write_register(r, b).unwrap();
        });

        Result::Ok(())
    }

    pub fn set_adc_gain_calibration(&mut self, adc_channel: AdcChannel, gain: i32) -> Result<(), Error<E>> {
        let b = gain.to_be_bytes();

        let registers = match adc_channel {
            AdcChannel::A => [
                Registers::OCAL1_B0,
                Registers::GCAL1_B1,
                Registers::GCAL1_B2,
                Registers::GCAL1_B3,
            ],
            AdcChannel::B => [
                Registers::GCAL2_B0,
                Registers::GCAL2_B1,
                Registers::GCAL2_B2,
                Registers::GCAL2_B3,
            ],
        };

        registers.iter().zip(b.iter()).for_each(|(r, b)| {
            self.write_register(r, b).unwrap();
        });

        Result::Ok(())
    }

    ///Reads the current ADC result.  i24 value returned in an i32.
    pub fn read_adc(&mut self) -> Result<i32, Error<E>> {
        let pu_ctrl = self.pu_ctrl().unwrap();

        if !pu_ctrl.CR {
            return Result::Err(Error::DataNotReady)
        }

        let write_buffer: [u8; 1] = [Registers::ADCO_B2 as u8];
        let mut read_buffer: [u8; 3] = [0; 3];

        match self
            .i2c
            .write_read(I2C_ADDR, &write_buffer, &mut read_buffer)
        {
            Ok(_) => {
                let val = Self::i32_from_i24_be_bytes(&read_buffer);

                Result::Ok(val)
            }
            Err(e) => Result::Err(Error::I2C(e)),
        }
    }
}
