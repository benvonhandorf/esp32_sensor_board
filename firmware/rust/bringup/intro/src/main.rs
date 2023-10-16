#![feature(never_type)]

use anyhow::Result;
use embedded_hal::{
    prelude:: {
        _embedded_hal_blocking_i2c_Write, 
        _embedded_hal_blocking_i2c_WriteRead, 
        _embedded_hal_blocking_i2c_Read
    },
    digital::v2,
};

use embedded_svc:: {
    wifi::Configuration,
    wifi::ClientConfiguration,
};

use esp_idf_hal::{
    delay::{FreeRtos, Delay},
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
    spi::*, gpio::*,
};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop, 
    nvs::EspDefaultNvsPartition, 
    wifi::BlockingWifi, 
    wifi::EspWifi,
};

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use shared_bus::{BusManagerSimple, I2cProxy};
use sht4x:: {
    Sht4x,
    Precision
};

use embedded_sdmmc::*;

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

fn sd_test( ) {

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

    // let mut clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // let mut system = peripherals.SYSTEM.split();

    // // Disable the watchdog timers. For the ESP32-C3, this includes the Super WDT,
    // // the RTC WDT, and the TIMG WDTs.
    // let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    // let timer_group0 = TimerGroup::new(
    //     peripherals.TIMG0,
    //     &clocks,
    //     &mut system.peripheral_clock_control,
    // );
    // let mut wdt0 = timer_group0.wdt;
    // let timer_group1 = TimerGroup::new(
    //     peripherals.TIMG1,
    //     &clocks,
    //     &mut system.peripheral_clock_control,
    // );
    // let mut wdt1 = timer_group1.wdt;

    // rtc.swd.disable();
    // rtc.rwdt.disable();
    // wdt0.disable();
    // wdt1.disable();

    let mut delay = Delay;

    let scl = peripherals.pins.gpio5;
    let sda = peripherals.pins.gpio4;

    let i2c_config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &i2c_config).unwrap();

    let i2c_bus = BusManagerSimple::new(i2c);

    let mut proxy_scan = i2c_bus.acquire_i2c();
    let mut proxy_sht = i2c_bus.acquire_i2c();

    info!("I2c Bus Configured");

    scan_i2c_bus(&mut proxy_scan);

    let spi = peripherals.spi2;

    let sck = peripherals.pins.gpio15;
    let cipo = peripherals.pins.gpio17;
    let copi = peripherals.pins.gpio16;
    let sd_cs = peripherals.pins.gpio14;
    let mut sd_det = PinDriver::input(peripherals.pins.gpio11).unwrap();

    sd_det.set_pull(Pull::Up).unwrap();

    let has_sd = sd_det.get_level();

    if has_sd == esp_idf_hal::gpio::Level::Low {
        info!("SD Card Detected");
    } else {
        info!("No SD Card Detected");
    }

    //cargo build --bin launch --features launch --target .vscode --profile debug-no-opt

    let driver_config = SpiDriverConfig::new();

        info!("Initializing SD Card");

        let spi_driver = SpiDriver::new(spi,
        sck,
        copi,
        Some(cipo),
        &driver_config
        ).unwrap();

        info!("Preparing SD Card");

        let sdcard_config = config::Config::new().baudrate(26.MHz().into());

        let sd_device = SpiDeviceDriver::new(
            &spi_driver,
            None::<AnyOutputPin>,
            &sdcard_config
        ).unwrap();

        let sdcard = embedded_sdmmc::SdCard::new(sd_device, PinDriver::output(sd_cs).unwrap(), FreeRtos);

        info!("SD Card Type: {}", sdcard.get_card_type().unwrap() as u8);

        info!("SD Card Size: {}", sdcard.num_bytes().unwrap());

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), None).unwrap(),
        sys_loop,
    ).unwrap();

    let _ = wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()));

    wifi.start().unwrap();

    let scan_result = wifi.scan();

    if scan_result.is_err() {
        error!("Scan Failed: {:x?}", scan_result);
    }

    info!("Scan Result: {:?}", scan_result.unwrap());

    // init_wifi(&peripherals);

    let mut message_6: [u8; 6] = [0; 6];

    // For SHT40-AD1B, use address 0x44
    let mut sht40 = Sht4x::new(proxy_sht);

    let device_id = sht40.serial_number(&mut delay).unwrap();

    info!("SHT40 Sensor Device Id: {:#02x}", device_id);

    loop {
        let measurement = sht40.measure(Precision::Low, &mut delay).unwrap();
        info!(
            "Temp: {:.2}\tHumidity: {:.2}",
            measurement.temperature_celsius(),
            measurement.humidity_percent()
        );

        FreeRtos::delay_ms(1000u32);
    }
}
