use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys;
use log::info;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

mod deep_sleep;
use deep_sleep::enter_deep_sleep;

mod gps;
use gps::get_lat_lon;

mod sleep_store;
use sleep_store::DeepSleepStore;

// 0.0001 degrees is roughly 11 meters
const MOVEMENT_THRESHOLD: f32 = 0.0001;

fn main() -> Result<(), Box<dyn Error>> {
    sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // Give USB some time to enumerate (Crucial for seeing logs after a wake-up reboot)
    sleep(Duration::from_secs(3));
    info!("Device is awake (Booted/Reset)! Waiting for USB enumeration...");

    unsafe {
        let reset_reason = sys::esp_reset_reason();
        info!("Reset reason: {}", reset_reason);
    }

    let last_data: Option<(f32, f32)> = DeepSleepStore::load();
    match last_data {
        Some((lat, lon)) => info!("Previous State -> Lat: {}, Lon: {}", lat, lon),
        None => info!("Previous State -> Lat: None, Lon: None"),
    }

    info!("Initializing GPIOs...");
    unsafe {
        sys::gpio_hold_dis(pins.gpio3.pin());
        sys::gpio_hold_dis(pins.gpio2.pin());
    }

    let (gps_fix, gpio3) = get_lat_lon(peripherals.uart1, pins.gpio33, pins.gpio34, pins.gpio3)?;

    let Some((lat, lon)) = gps_fix else {
        info!("No GPS fix found. Sleeping immediately");
        return enter_deep_sleep(gpio3, pins.gpio2);
    };

    info!("Lat: {}, Lon: {}", lat, lon);

    let Some((last_lat, last_lon)) = last_data else {
        info!("No previous data found. Saving new data and sleeping");
        DeepSleepStore::save(lat, lon);
        return enter_deep_sleep(gpio3, pins.gpio2);
    };

    let lat_diff = (lat - last_lat).abs();
    let lon_diff = (lon - last_lon).abs();

    info!("Drift -> Lat: {:.6}, Lon: {:.6}", lat_diff, lon_diff);

    if lat_diff > MOVEMENT_THRESHOLD || lon_diff > MOVEMENT_THRESHOLD {
        info!("Moved significantly! Saving and sending.");
        DeepSleepStore::save(lat, lon);
        // TODO: Send to LoRaWAN server
    } else {
        info!(
            "Drift is small ({:.6} < {}). Staying asleep.",
            lat_diff.max(lon_diff),
            MOVEMENT_THRESHOLD
        );
    }

    info!("Deep sleeping now...");
    enter_deep_sleep(gpio3, pins.gpio2)
}
