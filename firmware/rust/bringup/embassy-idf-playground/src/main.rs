#![no_main]

use core::time::Duration;
use esp_idf_svc::{hal::task::*, sys::EspError, timer::EspTaskTimerService};

#[main]
async fn main(spawner: Spawner) {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let timer_service = EspTaskTimerService::new().unwrap();

    log::info!("Hello, world!");

    run(timer_service).unwrap();
}

fn run(timer_service: EspTaskTimerService) -> Result<(), EspError> {
    block_on(init(timer_service))?;

    block_on(timer_one(timer_service), timer_two(timer_service))
}

async fn init(timer_service: EspTaskTimerService) -> Result<(), EspError> {
    log::info!("init");

    Result::Ok(())
}

async fn timer_one(timer_service: EspTaskTimerService) -> Result<(), EspError> {
    let mut async_timer = timer_service.timer_async()?;

    loop {
        log::info!("timer_one");

        async_timer.after(Duration::from_secs(1)).await?;
    }
}

async fn timer_two(timer_service: EspTaskTimerService) -> Result<(), EspError> {
    let mut async_timer = timer_service.timer_async()?;

    loop {
        log::info!("timer_two");

        async_timer.after(Duration::from_millis(750)).await?;
    }
}
