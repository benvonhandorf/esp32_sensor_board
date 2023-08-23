use anyhow::Result;
use embedded_hal::{
    blocking::delay::DelayMs, 
    prelude:: {
        _embedded_hal_blocking_i2c_Write, 
        _embedded_hal_blocking_i2c_WriteRead, 
        _embedded_hal_blocking_i2c_Read
    }
};
use esp_idf_hal::{
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::BlockingWifi, wifi::EspWifi};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use shared_bus::{BusManagerSimple, I2cProxy};
use shtcx::{self, PowerMode as shtPowerMode};

fn scan_i2c_bus(bus: &mut impl embedded_hal::blocking::i2c::Write) {
    let address_range = 0x00..=0x7F;
    let empty_body: [u8; 0] = [];

    for address in address_range {
        let result = bus.write(address, &empty_body);

        if result.is_ok() {
            info!("Found {:#02x}", address);
        }
    }
}

fn calculate_t_c(data: &[u8; 6]) -> f32 {
    let s_t = u16::from_be_bytes([data[0], data[1]]);
    // let s_t: u16 = u16::from(data[0]) << 8 + u16::from(data[1]);
    let t = -45. + 175. * (s_t as f32) / 65535. ;

    t
}

fn calculate_rh(data: &[u8; 6]) -> f32 {
    let s_rh = u16::from_be_bytes([data[3], data[4]]);
    // let s_rh: u16 = u16::from(data[3]) << 8 + u16::from(data[4]);
    let rh = -6. + 125. * (s_rh as f32)/65535. ;

    rh
}

// fn init_wifi(peripherals: &Peripherals) {

// }

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
        sys_loop,
    ).unwrap();

    let scan_result = wifi.scan().unwrap();

    info!("Scan Result: {:?}", scan_result);

    // init_wifi(&peripherals);

    let scl = peripherals.pins.gpio5;
    let sda = peripherals.pins.gpio4;

    let i2c_config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &i2c_config).unwrap();

    let i2c_bus = BusManagerSimple::new(i2c);

    let mut proxy_scan = i2c_bus.acquire_i2c();
    let mut proxy_sht = i2c_bus.acquire_i2c();

    info!("I2c Bus Configured");

    scan_i2c_bus(&mut proxy_scan);

    let mut message_6: [u8; 6] = [0; 6];

    loop {
        proxy_sht.write(0x44, &[0x89]).unwrap();

        FreeRtos.delay_ms(10u32);

        let r = proxy_sht.read(0x44, &mut message_6);

        if r.is_ok() {
            info!(
                "SN Response: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                message_6[0], message_6[1], message_6[2], message_6[3], message_6[4], message_6[5]
            );
        } else {
            warn!("Error: {}", r.err().unwrap())
        }

        FreeRtos.delay_ms(1000u32);

        proxy_sht.write(0x44, &[0xF6]).unwrap();

        FreeRtos.delay_ms(10u32);

        let r = proxy_sht.read(0x44, &mut message_6);

        if r.is_ok() {
            info!(
                "TH Response: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} : {:.02} {:0.02}",
                message_6[0], message_6[1], message_6[2], message_6[3], message_6[4], message_6[5],
                calculate_t_c(&message_6), calculate_rh(&message_6), 
            );
        } else {
            warn!("Error: {}", r.err().unwrap())
        }

        FreeRtos.delay_ms(1000u32);
    }

    //For SHT40-AD1B, use address 0x44
    let mut sht_sensor = shtcx::generic(proxy_sht, 0x44);

    let device_id = sht_sensor.device_identifier().unwrap();

    info!("SHT40 Sensor Device Id: {:#02x}", device_id);

    loop {
        scan_i2c_bus(&mut proxy_scan);

        sht_sensor
            .start_measurement(shtPowerMode::NormalMode)
            .unwrap();

        FreeRtos.delay_ms(100u32);

        let sht_measturement = sht_sensor.get_measurement_result().unwrap();

        info!(
            "Temp: {:.2}\tHumidity: {:.2}",
            sht_measturement.temperature.as_degrees_celsius(),
            sht_measturement.humidity.as_percent()
        );

        FreeRtos.delay_ms(800u32);
    }
}
