use anyhow::Error;
use embedded_svc;
use esp_idf_svc::{self, wifi::{BlockingWifi, EspWifi}};

use embedded_svc::{wifi::ClientConfiguration, wifi::Configuration};

use crate::wifi_config;

static mut WIFI: Mutex<RefCell<Option<BlockingWifi<EspWifi>>>> = Mutex::new(RefCell::new(None));

pub fn wifi_scan() {
    if let Some(wifi) = WIFI.borrow().borrow_mut().as_mut() {
        wifi.scan();
    }
    let scan_result = WIFI.unwrap().scan();

    if scan_result.is_err() {
        error!("Scan Failed: {:x?}", scan_result);
    }

    info!("Scan Result: {:?}", scan_result.unwrap());
}


fn wifi_start<'a>(
    wifi_modem: esp_idf_hal::modem::Modem,
    sys_loop: EspSystemEventLoop,
) -> Result<(), Error> {
    //Configure wifi modem
    WIFI. = BlockingWifi::wrap(
                EspWifi::new(wifi_modem, sys_loop.clone(), None).unwrap(),
                sys_loop,
            )
        Some(w) => {
            WIFI
            WIFI = Some(w);
        }

    WIFI.unwrap().set_configuration(&Configuration::Client(ClientConfiguration::default())).unwrap();

    WIFI.start().unwrap();

    Result::Ok(())
}