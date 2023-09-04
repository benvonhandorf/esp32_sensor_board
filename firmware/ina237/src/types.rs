use std::convert::Into;

pub struct Configuration {
    address: u8,
    shunt_cal: u16,
}

impl Configuration {
    pub fn new(addr: u8, shunt: u16) -> Configuration {
        Configuration {
            address: addr,
            shunt_cal: shunt
        }
    }
}

pub enum AdcRange {
    /// 0 = 163.84 mV
    HIGH = 0,
    /// 1 = 40.96 mV
    LOW = 1,
}

pub enum Mode {
    SHUTDOWN = 0x00,
    TriggeredBusVoltageSs = 0x01,
    TriggeredShuntVoltageSs = 0x02,
    TriggeredShuntBusVoltageSs = 0x03,
    TriggeredTempSs = 0x04,
    TriggeredTempBusVoltageSs = 0x05,
    TriggeredTempShuntVoltageSs = 0x06,
    TriggeredTempShuntBusVoltageSs = 0x07,
    Shutdown08 = 0x08,
    ContinuousBusVoltage = 0x09,
    ContinuousShuntVoltage = 0x0A,
    ContinuousShuntBusVoltage = 0x0B,
    ContinuousTemp = 0x0C,
    ContinuousTempBusVoltage = 0x0D,
    ContinuousTempShuntVoltage = 0x0E,
    ContinuousTempShuntBusVoltage = 0x0F,
}

pub enum ConversionTime {
    DurationUs50 = 0x00,
    DurationUs84 = 0x01,
    DurationUs150 = 0x02,
    DurationUs280 = 0x03,
    DurationUs540 = 0x04,
    DurationUs1052 = 0x05,
    DurationUs2074 = 0x06,
    DurationUs4120 = 0x07,
}

pub enum AdcAveraging {
    Avg1 = 0x00,
    Avg4 = 0x01,
    Avg16 = 0x02,
    Avg64 = 0x03,
    Avg128 = 0x04,
    Avg256 = 0x05,
    Avg512 = 0x06,
    Avg1024 = 0x07,
}

pub struct ConfigurationRegisterValues {
    /// Force sensor reset
    reset: bool,
    /// Conversion delay, 2ms steps.  Range 0ms - 510ms
    conversion_delay: u8,
    /// ADC Range.  Default: High
    adc_range: AdcRange,
    /// Mode.  Default: CONTINUOUS_TEMP_SHUNT_BUS_VOLTAGE
    mode: Mode,
    /// ADC Conversion Time for Bus Voltage.  Default: 1052us
    bus_voltage_conversion_time: ConversionTime,
    /// ADC Conversion Time for Shunt Voltage.  Default: 1052us
    shunt_voltage_conversion_time: ConversionTime,
    /// ADC Conversion Time for Temperature.  Default: 1052us
    temperature_conversion_time: ConversionTime,
    /// ADC Averaging
    adc_averaging: AdcAveraging,
}

impl ConfigurationRegisterValues {
    fn new() -> ConfigurationRegisterValues {
        ConfigurationRegisterValues {
            reset : false,
            conversion_delay : 0,
            adc_range : AdcRange::HIGH,
            mode: Mode::ContinuousTempShuntBusVoltage,
            bus_voltage_conversion_time : ConversionTime::DurationUs1052,
            shunt_voltage_conversion_time : ConversionTime::DurationUs1052,
            temperature_conversion_time : ConversionTime::DurationUs1052,
            adc_averaging : AdcAveraging::Avg1,
        }
    }
}

impl Into<u16> for ConfigurationRegisterValues {
    fn into(self) -> u16 {
        0x0000 | if self.reset {
            0x8000
        } else {
            0x0000
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_constructor_sets_values() {
        let result = Configuration::new(0x01, 2000);

        assert_eq!(2000, result.shunt_cal);
    }

    #[test]
    fn configuration_values_into_u16_reset_false() {
        let configuration_register_values = ConfigurationRegisterValues::new();
        let result: u16 = configuration_register_values.into();

        assert_eq!(0x0000, 0x8000 & result);
    }

    #[test]
    fn configuration_values_with_reset_into_u16_reset_true() {
        let configuration_register_values = ConfigurationRegisterValues::new();
        let result: u16 = configuration_register_values.into();

        assert_eq!(0x8000, 0x8000 & result);
    }
}

