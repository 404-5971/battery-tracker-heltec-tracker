use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys;
use log::info;
use std::error::Error;
use std::time::Duration;

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

    info!("Initializing GPIOs...");
    unsafe {
        sys::gpio_hold_dis(pins.gpio3.pin() as i32);
        sys::gpio_hold_dis(pins.gpio2.pin() as i32);
    }

    for i in 0..5 {
        info!("Deep sleeping in {} seconds...", 5 - i);
        std::thread::sleep(Duration::from_secs(1));
    }

    info!("Deep sleeping now...");

    // 3. Prepare for sleep
    enter_deep_sleep(pins.gpio3, pins.gpio2)?;

    // This line is *probably* unreachable
    Ok(())
}

fn enter_deep_sleep(gpio3: Gpio3, gpio2: Gpio2) -> Result<(), Box<dyn Error>> {
    // 1. Configure Pins
    let mut vext = PinDriver::output(gpio3)?;
    let mut adc_ctrl = PinDriver::output(gpio2)?;

    vext.set_low()?;
    adc_ctrl.set_low()?;

    // 2. Enable Hold (Still requires unsafe for the raw sys call)
    // There is no standard "safe" HAL wrapper for hold_en yet that covers all chips perfectly.
    unsafe {
        sys::gpio_hold_en(vext.pin() as i32);
        sys::gpio_hold_en(adc_ctrl.pin() as i32);
    }

    // 3. Configure Sleep
    // We can use the safe wrapper for sleeping if we want,
    // though 'esp_deep_sleep_start' is often preferred for clarity on what exactly is happening.
    let sleep_duration = Duration::from_secs(10);
    info!("Entering Deep Sleep for {:?}...", sleep_duration);

    unsafe {
        sys::esp_sleep_enable_timer_wakeup(sleep_duration.as_micros() as u64);

        // Free memory/resources before hard shutdown (optional but good practice)
        // Explicitly drop drivers (though memory is wiped on sleep anyway)
        drop(vext);
        drop(adc_ctrl);

        sys::esp_deep_sleep_start();
    }
}
