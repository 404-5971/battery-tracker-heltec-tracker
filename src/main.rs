use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys;
use log::info;
use std::error::Error;
use std::time::Duration;

mod deep_sleep;
use deep_sleep::enter_deep_sleep;

mod gps;
use gps::get_lat_lon;

mod sleep_store;
use sleep_store::{DeepSleepStore, LastLonLat};

fn main() -> Result<(), Box<dyn Error>> {
    sys::link_patches();
    EspLogger::initialize_default();

    // 1. Take peripherals ONCE at the top of main
    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // 2. Perform your logic
    // Give USB some time to enumerate (Crucial for seeing logs after a wake-up reboot)
    std::thread::sleep(Duration::from_secs(3));
    info!("Device is awake (Booted/Reset)! Waiting for USB enumeration...");

    // Check reset reason for logging purposes
    unsafe {
        let reset_reason = sys::esp_reset_reason();
        info!("Reset reason: {}", reset_reason);
    }

    // Load previous state safely
    let last_data = DeepSleepStore::load();
    info!(
        "Previous State -> Lat: {:?}, Lon: {:?}",
        last_data.last_lat, last_data.last_lon
    );

    info!("Initializing GPIOs...");
    unsafe {
        sys::gpio_hold_dis(pins.gpio3.pin());
        sys::gpio_hold_dis(pins.gpio2.pin());
    }

    let (gps_fix, gpio3) = get_lat_lon(peripherals.uart1, pins.gpio33, pins.gpio34, pins.gpio3)?;
    match gps_fix {
        Some((lat, lon)) => info!("Lat: {}, Lon: {}", lat, lon),
        None => info!("No GPS fix found"),
    }

    // Save new state safely
    DeepSleepStore::save(LastLonLat {
        last_lat: gps_fix.map(|(lat, _)| lat),
        last_lon: gps_fix.map(|(_, lon)| lon),
    });

    /* TODO: If the new GPS coords rounded aren't close enough to the last coords
    then we've moved, and we need to send our location to the LoRaWAN server
    else we can go back to sleep, and check again later*/

    for i in 0..5 {
        info!("Deep sleeping in {} seconds...", 5 - i);
        std::thread::sleep(Duration::from_secs(1));
    }

    info!("Deep sleeping now...");
    // 3. Prepare for sleep
    enter_deep_sleep(gpio3, pins.gpio2)
}
