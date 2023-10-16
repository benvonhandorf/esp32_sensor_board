use log::*;
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
    gpio::{
        AnyOutputPin,
        PinDriver,
    },
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, 
    nvs::EspDefaultNvsPartition, 
    wifi::BlockingWifi, 
    wifi::EspWifi,
};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use embedded_sdmmc::*;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let spi = peripherals.spi2;

    /*
    #define CONFIG_EXAMPLE_PIN_MOSI 9
#define CONFIG_EXAMPLE_PIN_MISO 8
#define CONFIG_EXAMPLE_PIN_CLK 7
#define CONFIG_EXAMPLE_PIN_CS 21
 */

    let sck = peripherals.pins.gpio7; //D8
    let cipo = peripherals.pins.gpio8; //D9
    let copi = peripherals.pins.gpio9; //D10
    let sd_cs = peripherals.pins.gpio21; //D2
    // let mut sd_det = PinDriver::input(peripherals.pins.gpio11).unwrap();

    // sd_det.set_pull(Pull::Up).unwrap();

    // let has_sd = sd_det.get_level();

    // if has_sd == esp_idf_hal::gpio::Level::Low {
    //     info!("SD Card Detected");
    // } else {
    //     info!("No SD Card Detected");
    // }

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

        let card_type = sdcard.get_card_type();

        if card_type.is_some() {
            info!("SD Card Type: {}", card_type.unwrap() as u8);
        }

        let num_bytes = sdcard.num_bytes();

        if num_bytes.is_ok() {
            info!("SD Card Size: {}", num_bytes.unwrap());
        }

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
}
