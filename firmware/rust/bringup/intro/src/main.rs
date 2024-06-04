#![feature(never_type)]

use std::borrow::BorrowMut;
use std::ptr::null;
use std::sync::Mutex;

use anyhow::Result;


use embedded_hal::i2c::I2c;
use embedded_hal::i2c::SevenBitAddress;
use embedded_hal_bus::i2c::AtomicError;
use embedded_hal_bus::util::AtomicCell;
use esp_idf_svc;
use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::i2c::I2C0;
use esp_idf_svc::hal::modem;
use esp_idf_svc::hal as esp_idf_hal;
use esp_idf_svc::sys as esp_idf_sys;
use embedded_svc;

use embedded_hal::{
    i2c,
    digital,
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
    spi::*, 
    gpio::*,
    units::*,
};

use embedded_hal_bus::i2c::AtomicDevice;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop, 
    nvs::EspDefaultNvsPartition, 
    wifi::BlockingWifi, 
    wifi::EspWifi,
};

use esp_idf_sys as _;
use ina237::types::AdcAveraging;
use ina237::types::AdcRange;
use ina237::types::Mode;
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
// use shared_bus::{BusManagerSimple, I2cProxy};

use sht4x:: {
    Sht4x,
    Precision
};

use ina237::ina237:: {
    Ina237,
};

// use embedded_sdmmc::*;

fn i2c_scan(i2c_bus_device: &mut AtomicDevice<I2cDriver>) {
    let address_range = 0x00..=0x7F;
    let empty_body: [u8; 0] = [];

    let i2c_bus = i2c_bus_device.borrow_mut();

    for address in address_range {
        let result = i2c_bus.write(address, &empty_body);

        if result.is_ok() {
            info!("Found {:#02x}", address);
        } else {
            match result.err() {
                Some(AtomicError::Busy) => warn!("Unable to scan {:#02x} - Busy", address),
                Some(AtomicError::Other(e)) => warn!("Unable to scan {:#02x} - {}", address, e),
                _ => warn!("Unable to scan {:#02x} - Unknown Reason", address),
            }
        }
    }
}

fn sht_init<'a>(i2c_bus: AtomicDevice<'a, I2cDriver<'a>> ) -> Sht4x<AtomicDevice<'a, I2cDriver<'a>>, Delay> {
    // // For SHT40-AD1B, use address 0x44
    let sht40 = Sht4x::new(i2c_bus);

    return sht40;
}

fn sht_read(sht_driver: &mut Sht4x<AtomicDevice<I2cDriver>, Delay>, delay:&mut Delay ) {
// // For SHT40-AD1B, use address 0x44
    let device_id = sht_driver.serial_number(delay).unwrap();

    info!("SHT40 Sensor Device Id: {:#02x}", device_id);

    let measurement = sht_driver.measure(Precision::High, delay).unwrap();
    info!(
        "Temp: {:.2}\tHumidity: {:.2}",
        measurement.temperature_celsius(),
        measurement.humidity_percent(),
    );
}

fn sd_test( ) {

}

fn wifi_scan(wifi: &mut BlockingWifi<EspWifi>) {
    let scan_result = wifi.scan();

    if scan_result.is_err() {
        error!("Scan Failed: {:x?}", scan_result);
    }

    info!("Scan Result: {:?}", scan_result.unwrap());
}

fn wifi_init<'a>(wifi_modem: esp_idf_hal::modem::Modem, sys_loop: EspSystemEventLoop) -> BlockingWifi<EspWifi<'a>> {
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(wifi_modem, sys_loop.clone(), None).unwrap(),
        sys_loop,
    ).unwrap();

    let _ = wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()));

    wifi.start().unwrap();

    return wifi;
}

fn main() -> ! {
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

    let mut delay = Delay::new_default();

    let mut wifi = wifi_init(peripherals.modem, sys_loop);
    wifi_scan(&mut wifi);

    let scl = peripherals.pins.gpio5;
    let sda = peripherals.pins.gpio4;

    let i2c_config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_bus_raw = I2cDriver::new(peripherals.i2c0, sda, scl, &i2c_config).unwrap();

    let i2c_bus_cell = AtomicCell::new(i2c_bus_raw);

    info!("I2c Bus Configured");

    {
        let mut i2c_scan_bus = AtomicDevice::new(&i2c_bus_cell);

        i2c_scan(&mut i2c_scan_bus);
    }

    {
        let i2c_sht_bus = AtomicDevice::new(&i2c_bus_cell);

        let mut sht40 = Sht4x::new(i2c_sht_bus);

        let i2c_ina_bus = AtomicDevice::new(&i2c_bus_cell);

        let mut ina_config_registers = ina237::types::ConfigurationRegisterValues::new();
        ina_config_registers.adc_range = AdcRange::LOW;
        ina_config_registers.mode = Mode::ContinuousTempShuntBusVoltage;
        ina_config_registers.adc_averaging = AdcAveraging::Avg64;

        let ina_configuration_a = ina237::types::Configuration::new(0x46, 4000);

        let mut ina_a = Ina237::new(i2c_ina_bus, ina_configuration_a);
        
        ina_a.initialize(ina_config_registers);

        info!("INA237 A: Configuration {:#04}", ina_a.configuration());

        loop {
            sht_read(&mut sht40, &mut delay);

            FreeRtos::delay_ms(1000u32);

            info!(
                "INA 237 A: {:#04x}",
                ina_a.manufacturer_id(),
            );
        }
    }

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

    // let driver_config = SpiDriverConfig::new();

        // info!("Initializing SD Card");

        // let spi_driver = SpiDriver::new(spi,
        // sck,
        // copi,
        // Some(cipo),
        // &driver_config
        // ).unwrap();

        // info!("Preparing SD Card");

        // let sdcard_config = config::Config::new().baudrate(26.MHz().into());

        // let sd_device = SpiDeviceDriver::new(
        //     &spi_driver,
        //     None::<AnyOutputPin>,
        //     &sdcard_config
        // ).unwrap();

        // let sdcard = embedded_sdmmc::SdCard::new(sd_device, PinDriver::output(sd_cs).unwrap(), FreeRtos);

        // info!("SD Card Type: {}", sdcard.get_card_type().unwrap() as u8);

        // info!("SD Card Size: {}", sdcard.num_bytes().unwrap());


    let mut message_6: [u8; 6] = [0; 6];

    loop {

    }
}
