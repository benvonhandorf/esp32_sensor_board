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
use sht4x::Sht4x;

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


    // let mut clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    // let mut delay = Delay::new(&clocks);

    //For SHT40-AD1B, use address 0x44
    // let mut sht40 = Sht4x::new(i2c);

    // let device_id = sht40.serial_number(&mut delay);

    // info!("SHT40 Sensor Device Id: {:#02x}", device_id);

    // loop {
    //     let measurement = sht40.measure(Precision::Low, &mut delay);
    //     info!(
    //         "Temp: {:.2}\tHumidity: {:.2}",
    //         measurement.temperature_celsius(),
    //         measurement.humidity_percent()
    //     );

    //     FreeRtos.delay_ms(1000u32);
    // }
}
