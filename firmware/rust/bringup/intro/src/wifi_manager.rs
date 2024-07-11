use anyhow::Error;
use embedded_svc;
use esp_idf_svc::{self, wifi::{BlockingWifi, EspWifi}};

use embedded_svc::{wifi::ClientConfiguration, wifi::Configuration};

pub fn wifi_create(
    sys_loop: &EspSystemEventLoop,
    nvs: &EspDefaultNvsPartition,
) -> Result<EspWifi<'static>, EspError> {
    let peripherals = Peripherals::take()?;

    let mut esp_wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs.clone()))?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sys_loop.clone())?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: wifi_config::WIFI_CONFIGURATION.SSID.try_into().unwrap(),
        password: wifi_config::WIFI_CONFIGURATION.psk.try_into().unwrap(),
        ..Default::default()
    }))?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected");

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(esp_wifi)
}
