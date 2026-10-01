#![no_std]

use core::time::Duration;

use myrmic_sdk::outlet::Outlet;
use myrmic_sdk::signal_layer::ThresholdAlarm;
use myrmic_sdk::tap::Tap;
use myrmic_sdk::{Callback, Metadata, Result};
use signal_layer_types::DigitalState;

static mut LED_ON: bool = false;

#[myrmic_sdk::init]
fn init(_md: Metadata) -> Result<()> {
    let _ = myrmic_sdk::info!("[knock-light] application started");

    // Start with LED OFF.
    if let Some(led) = Outlet::resolve("led_cmd")? {
        led.write_typed(&DigitalState { on: false })?;
    }

    let _ = myrmic_sdk::interval(
        Callback::of::<check_tap>(),
        Duration::from_millis(100),
    )
    .build()
    .map_err(|_| "failed to create tap timer")?;

    Ok(())
}

#[myrmic_sdk::cmd]
fn check_tap(_md: Metadata) -> Result<()> {
    let Some(tap) = Tap::resolve("tap")? else {
        return Ok(());
    };

    // Treat the queued threshold events as one knock for this polling cycle.
    let mut detected = false;
    let mut value = 0.0_f32;

    while let Some(alarm) = tap.take_event_typed::<ThresholdAlarm>()? {
        detected = true;
        value = alarm.value;
    }

    if !detected {
        return Ok(());
    }

    let new_state = unsafe {
        LED_ON = !LED_ON;
        LED_ON
    };

    let Some(led) = Outlet::resolve("led_cmd")? else {
        return Err("led_cmd outlet not found");
    };

    led.write_typed(&DigitalState { on: new_state })?;

    let _ = myrmic_sdk::info!(
        "[knock-light] KNOCK value={:.3}g LED={}",
        value,
        new_state
    );

    Ok(())
}
