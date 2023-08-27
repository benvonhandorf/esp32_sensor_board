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
    TRIGGERED_BUS_VOLTAGE_SS = 0x01,
    TRIGGERED_SHUNT_VOLTAGE_SS = 0x02,
    TRIGGERED_SHUNT_BUS_VOLTAGE_SS = 0x03,
    TRIGGERED_TEMP_SS = 0x04,
    TRIGGERED_TEMP_BUS_VOLTAGE_SS = 0x05,
    TRIGGERED_TEMP_SHUNT_VOLTAGE_SS = 0x06,
    TRIGGERED_TEMP_SHUNT_BUS_VOLTAGE_SS = 0x07,
    SHUTDOWN_08 = 0x08,
    CONTINUOUS_BUS_VOLTAGE = 0x09,
    CONTINUOUS_SHUNT_VOLTAGE = 0x0A,
    CONTINUOUS_SHUNT_BUS_VOLTAGE = 0x0B,
    CONTINUOUS_TEMP = 0x0C,
    CONTINUOUS_TEMP_BUS_VOLTAGE = 0x0D,
    CONTINUOUS_TEMP_SHUNT_VOLTAGE = 0x0E,
    CONTINUOUS_TEMP_SHUNT_BUS_VOLTAGE = 0x0F,
}

pub enum ConversionTime {
    50_us = 0x00,
    84_us = 0x01,
    150_us = 0x02,
    280_us = 0x03,
    540_us = 0x04,
    1052_us = 0x05,
    2074_us = 0x06,
    4120_us = 0x07,
}

pub enum AdcAveraging {
    AVG_1 = 0x00,
    AVG_4 = 0x01,
    AVG_16 = 0x02,
    AVG_64 = 0x03,
    AVG_128 = 0x04,
    AVG_256 = 0x05,
    AVG_512 = 0x06,
    AVG_1024 = 0x07,
}

pub struct ConfigurationValues {
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
    temp_conversion_time: ConversionTime,
    /// ADC Averaging
    adc_averaging: AdcAveraging,
}

impl ConfigurationValues {
    fn new() -> ConfigurationValues {
        ConfigurationValues {
            reset = false,
            conversion_delay = 0,
            adc_range = AdcRange::HIGH,
            mode: Mode::CONTINUOUS_TEMP_SHUNT_BUS_VOLTAGE,
            bus_voltage_conversion_time = ConversionTime::1052_us,
            shunt_voltage_conversion_time = ConversionTime::1052_us,
            temperature_conversion_time = ConversionTime::1052_us,
            adc_averaging = AdcAveraging::AVG_1,
        }
    }
}

impl Into<u16> ConfigurationValues {
    fn into(self) -> u16 {
        0x00
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_constructor_sets_values() {
        let result = Configuration::new(0x01, 2000);

        assert_eq!(0x01, result.address);
        assert_eq!(2000, result.shunt_cal);
    }

    #[test]
    fn configuration_values_into_u16_reset_false() {
        let result = ConfigurationValues::new();

        assert_eq!(0x01, result.address);
        assert_eq!(2000, result.shunt_cal);
    }
}

