#![feature(never_type)]

mod wifi_manager;
mod wifi_config;
mod mqtt_manager;

use std::borrow::BorrowMut;
use std::ptr::null;
use std::sync::Mutex;

use anyhow::Result;

use embedded_hal::i2c::I2c;
use embedded_hal::i2c::SevenBitAddress;
use embedded_hal_bus::i2c::AtomicError;
use embedded_hal_bus::util::AtomicCell;
use embedded_svc;
use esp_idf_svc;
use esp_idf_svc::hal::peripherals;
use esp_idf_svc::hal as esp_idf_hal;
use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::i2c::I2C0;
use esp_idf_svc::hal::modem;
use esp_idf_svc::sys as esp_idf_sys;

use embedded_hal::{digital, i2c};

use esp_idf_hal::{
    delay::{Delay, FreeRtos},
    gpio::*,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
    spi::*,
    units::*,
};

use embedded_hal_bus::i2c::AtomicDevice;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::BlockingWifi, wifi::EspWifi,
};

use esp_idf_sys as _;
use ina237::types::AdcAveraging;
use ina237::types::AdcRange;
use ina237::types::Mode;
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
// use shared_bus::{BusManagerSimple, I2cProxy};

use sht4x::{Precision, Sht4x};

use ina237::ina237::Ina237;

use nau7802::nau7802::Nau7802;

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

fn sht_init<'a>(
    i2c_bus: AtomicDevice<'a, I2cDriver<'a>>,
) -> Sht4x<AtomicDevice<'a, I2cDriver<'a>>, Delay> {
    // // For SHT40-AD1B, use address 0x44
    let sht40 = Sht4x::new(i2c_bus);

    return sht40;
}

fn sht_read(sht_driver: &mut Sht4x<AtomicDevice<I2cDriver>, Delay>, delay: &mut Delay) -> (f32, f32) {
    let measurement = sht_driver.measure(Precision::High, delay).unwrap();

    return (measurement.temperature_celsius().to_num::<f32>(), measurement.humidity_percent().to_num::<f32>());

    // info!(
    //     "Temp: {:.2}\tHumidity: {:.2}",
    //     measurement.temperature_celsius(),
    //     measurement.humidity_percent(),
    // );
}

fn ina_read(ina_driver: &mut Ina237<AtomicDevice<I2cDriver>>, delay: &mut Delay) -> (i32, i32, i32, i32) {
    let m = ina_driver.read().unwrap_or_else(|error| {
        panic!("Error reading INA")
    });

    return (m.voltage_mV(), m.shunt_uV(), m.current_uA(), m.temp_mC());

    // info!("INA A: {} mV {} uV {} uA {} mC", m.voltage_mV(), m.shunt_uV(), m.current_uA(), m.temp_mC());
}

fn nau_read(nau_driver: &mut Nau7802<AtomicDevice<I2cDriver>,Delay>, delay: &mut Delay) -> (i32) {
    let is_ready = nau_driver.is_data_ready().unwrap();
    let ctrl2 = nau_driver.ctrl2().unwrap();

    let reading_a = if is_ready {
        nau_driver.read_adc().unwrap()
    } else {

    info!("NAU: data ready: {} {:04x}", is_ready, Into::<u8>::into(ctrl2));

        0
    };

    delay.delay_ms(10);

    return (reading_a)
}

fn sd_test() {}




fn main() -> ! {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let _wifi = wifi_manager::wifi_create(&sys_loop, &nvs).unwrap();

    let mqtt_client = mqtt_manager::mqtt_create().unwrap();

    mqtt_manager::mqtt_post(&mqtt_client, "sensor/esp123/status", "alive").unwrap();

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

    // {
    //     let mut i2c_scan_bus = AtomicDevice::new(&i2c_bus_cell);

    //     i2c_scan(&mut i2c_scan_bus);
    // }

    {
        let i2c_nau_bus = AtomicDevice::new( &i2c_bus_cell);
        let mut nau_driver = Nau7802::new(i2c_nau_bus);

        nau_driver.initialize(&mut delay).unwrap();
        
        info!("NAU Revision: {:#04x}", nau_driver.revision_id().unwrap());

        info!("NAU CTRL1: {:#04x} CTRL2: {:#04x}", Into::<u8>::into(nau_driver.ctrl1().unwrap()), Into::<u8>::into(nau_driver.ctrl2().unwrap()));

        nau_driver.set_ldo_voltage(nau7802::registers::LdoVoltage::v3_0).unwrap();
        nau_driver.enable_ldo().unwrap();

        // if nau_driver.select_channel(nau7802::nau7802::AdcChannel::B, &mut delay).unwrap() {
        //     info!("NAU Channel Changed to B");
        //     delay.delay_ms(20);
        // }

        // info!("NAU CTRL1: {:#04x} CTRL2: {:#04x}", Into::<u8>::into(nau_driver.ctrl1().unwrap()), Into::<u8>::into(nau_driver.ctrl2().unwrap()));

        // info!("NAU Calibrate begin");

        // nau_driver.calibrate(&mut delay).unwrap();

        // info!("NAU Calibrate end");

        info!("NAU CTRL1: {:#04x} CTRL2: {:#04x}", Into::<u8>::into(nau_driver.ctrl1().unwrap()), Into::<u8>::into(nau_driver.ctrl2().unwrap()));

        if nau_driver.select_channel(nau7802::nau7802::AdcChannel::A, &mut delay).unwrap() {
            info!("NAU Channel Changed to A");
            delay.delay_ms(20);
        }

        nau_driver.set_gain(nau7802::registers::Gains::x16).unwrap();

        info!("NAU CTRL1: {:#04x} CTRL2: {:#04x}", Into::<u8>::into(nau_driver.ctrl1().unwrap()), Into::<u8>::into(nau_driver.ctrl2().unwrap()));

        info!("NAU Calibrate begin");

        nau_driver.calibrate(&mut delay).unwrap();

        info!("NAU Calibrate end");

        info!("NAU CTRL1: {:#04x} CTRL2: {:#04x}", Into::<u8>::into(nau_driver.ctrl1().unwrap()), Into::<u8>::into(nau_driver.ctrl2().unwrap()));

        let i2c_sht_bus = AtomicDevice::new(&i2c_bus_cell);

        let mut sht40 = Sht4x::new(i2c_sht_bus);

        let device_id = sht40.serial_number(&mut delay).unwrap();

        info!("SHT40 Sensor Device Id: {:#02x}", device_id);

        let i2c_ina_bus = AtomicDevice::new(&i2c_bus_cell);

        let mut ina_config_registers = ina237::types::ConfigurationRegisterValues::new();

        ina_config_registers.adc_range = AdcRange::LOW;
        ina_config_registers.mode = Mode::ContinuousTempShuntBusVoltage;
        ina_config_registers.adc_averaging = AdcAveraging::Avg64;

        let ina_configuration_a = ina237::types::Configuration::new(0x46, 4000);

        let mut ina_a = Ina237::new(i2c_ina_bus, ina_configuration_a);

        ina_a.initialize(ina_config_registers);

        info!("INA237 A: Configuration {:#04x}", ina_a.configuration());

        info!("INA237 A: ADC Configuration {:#04x}", ina_a.adc_configuration());

        info!("INA 237 A: {:#04x}", ina_a.manufacturer_id(),);

        info!("INA 237 A: Shunt Cal: {}", ina_a.shunt_cal());

        FreeRtos::delay_ms(200u32);

        loop {
            let sht_reading = sht_read(&mut sht40, &mut delay);

            FreeRtos::delay_ms(200u32);

            let ina_reading = ina_read(&mut ina_a, &mut delay);

            FreeRtos::delay_ms(200u32);

            let nau_reading = nau_read(&mut nau_driver, &mut delay);

            info!("Reading: {} mV, {} uA, {} C, {} %RH, {} ({:04x}) A", 
            ina_reading.0, ina_reading.3,
            sht_reading.0, sht_reading.1,
            nau_reading, nau_reading);
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

    loop {}
}
