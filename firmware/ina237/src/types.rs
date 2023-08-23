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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_constructor_sets_values() {
        let result = Configuration::new(0x01, 2000);

        assert_eq!(0x01, result.address);
        assert_eq!(2000, result.shunt_cal);
    }
}

