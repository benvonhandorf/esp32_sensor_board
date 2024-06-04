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
