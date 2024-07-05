use num_enum::FromPrimitive;

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Registers {
    PU_CTRL = 0x00,
    CTRL1 = 0x01,
    CTRL2 = 0x02,
    OCAL1_B2 = 0x03,
    OCAL1_B1 = 0x04,
    OCAL1_B0 = 0x05,
    GCAL1_B3 = 0x06,
    GCAL1_B2 = 0x07,
    GCAL1_B1 = 0x08,
    GCAL1_B0 = 0x09,
    OCAL2_B2 = 0x0A,
    OCAL2_B1 = 0x0B,
    OCAL2_B0 = 0x0C,
    GCAL2_B3 = 0x0D,
    GCAL2_B2 = 0x0E,
    GCAL2_B1 = 0x0F,
    GCAL2_B0 = 0x10,
    I2C_CTRL = 0x11,
    ADCO_B2 = 0x12,
    ADCO_B1 = 0x13,
    ADCO_B0 = 0x14,
    OTP_B1 = 0x15,
    OTP_B0 = 0x16,
    DEVICE_REVISION = 0x1F,
}


#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PU_CTRL {
    pub AVDDS: bool,
    pub OSCS: bool,
    pub CR: bool, //ro
    pub CS: bool,
    pub PUR: bool, //ro
    pub PUA: bool,
    pub PUD: bool,
    pub RR: bool,
}

impl PU_CTRL {
    pub fn reset(reset: bool) -> PU_CTRL {
        Self {
            AVDDS: false,
            OSCS: false,
            CR: false,
            CS: false,
            PUR: false,
            PUA: false,
            PUD: false,
            RR: reset,
        }
    }
}

impl From<u8> for PU_CTRL {
    fn from(value: u8) -> Self {
        Self {
            AVDDS: value & 0x80 != 0,
            OSCS: value & 0x40 != 0,
            CR: value & 0x20 != 0,
            CS: value & 0x10 != 0,
            PUR: value & 0x08 != 0,
            PUA: value & 0x04 != 0,
            PUD: value & 0x02 != 0,
            RR: value & 0x01 != 0,
        }
    }
}

impl Into<u8> for PU_CTRL {
    fn into(self) -> u8 {
        0x00 | if self.AVDDS { 0x80 } else { 0x00 }
            | if self.OSCS { 0x40 } else { 0x00 }
            | if self.CR { 0x20 } else { 0x00 }
            | if self.CS { 0x10 } else { 0x00 }
            | if self.PUR { 0x08 } else { 0x00 }
            | if self.PUA { 0x04 } else { 0x00 }
            | if self.PUD { 0x02 } else { 0x00 }
            | if self.RR { 0x01 } else { 0x00 }
    }
}

#[repr(u8)]
// #[FromPrimitive(u8)]
#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
#[allow(non_camel_case_types)]
pub enum LdoVoltage {
    v2_4 = 0b111,
    v2_7 = 0b110,
    v3_0 = 0b101,
    v3_3 = 0b100,
    v3_6 = 0b011,
    v3_9 = 0b010,
    v4_2 = 0b001,
    #[default] v4_5 = 0b000,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
#[allow(non_camel_case_types)]
pub enum Gains {
    x128 = 0b111,
    x64 = 0b110,
    x32 = 0b101,
    x16 = 0b100,
    x8 = 0b011,
    x4 = 0b010,
    x2 = 0b001,
    #[default] x1 = 0b000,
}

#[derive(Copy, Clone, Debug, PartialEq)]

pub struct CTRL1 {
    pub conversion_ready_polarity_high: bool,
    pub drdy_clock_output: bool,
    pub ldo_voltage: LdoVoltage,
    pub gain_select: Gains,
}

impl From<u8> for CTRL1 {
    fn from(value: u8) -> Self {
        Self {
            conversion_ready_polarity_high: !(value & 0x80) == 0x80,
            drdy_clock_output: value & 0x40 != 0,
            ldo_voltage: LdoVoltage::from((value >> 3)  & 0x07),
            gain_select: Gains::from(value & 0x07),
        }
    }
}

impl Into<u8> for CTRL1 {
    fn into(self) -> u8 {
        0x00 | if self.conversion_ready_polarity_high { 0x00 } else { 0x80 }
            | if self.drdy_clock_output { 0x40 } else { 0x00 }
            | ((self.ldo_voltage as u8) & 0x07) << 3
            | ((self.gain_select as u8) & 0x07) << 0
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
#[allow(non_camel_case_types)]
pub enum ConversionRate {
    SPS_320 = 0b111,
    SPS_80 = 0b011,
    SPS_40 = 0b010,
    SPS_20 = 0b001,
    #[default] SPS_10 = 0b000,
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CTRL2 {
    pub channel2_selected: bool,
    pub conversion_rate: ConversionRate,
    pub cal_error: bool,
    pub calibrate: bool,
    pub cal_mod: u8,
}

impl From<u8> for CTRL2 {
    fn from(value: u8) -> Self {
        Self {
            channel2_selected: value & 0x80 != 0,
            conversion_rate: ConversionRate::from(value >> 4 & 0x07),
            cal_error: value & 0x08 != 0,
            calibrate: value & 0x04 != 0,
            cal_mod: value & 0x03,
        }
    }
}

impl Into<u8> for CTRL2 {
    fn into(self) -> u8 {
        0x00 | if self.channel2_selected { 0x80 } else { 0x00 }
            | ((self.conversion_rate as u8) & 0x07) << 4
            | if self.cal_error { 0x08 } else { 0x00 }
            | if self.calibrate { 0x04 } else { 0x00 }
            | self.cal_mod & 0x03 
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_gain_through_u8_returns_same_value_x32() {
        const INPUT: Gains = Gains::x32;

        let intermediate = INPUT as u8;

        let result = Gains::from(intermediate);

        assert_eq!(INPUT, result);
    }
}