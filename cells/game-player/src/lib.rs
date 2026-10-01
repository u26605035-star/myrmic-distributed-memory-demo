#![no_std]

extern crate alloc;

use alloc::string::String;
use core::time::Duration;

use myrmic_sdk::outlet::Outlet;
use myrmic_sdk::signal_layer::ThresholdAlarm;
use myrmic_sdk::tap::Tap;
use myrmic_sdk::{Callback, Metadata, Result, publish, runtime_id};
use serde::{Deserialize, Serialize};
use signal_layer_types::DigitalState;

#[derive(Serialize, Deserialize, myrmic_sdk::Message)]
#[codec(myrmic_sdk::Postcard)]
struct TapInput {
    runtime_id: String,
    value: f32,
}

#[myrmic_sdk::init]
fn init(_md: Metadata) -> Result<()> {
    let _ = myrmic_sdk::interval(Callback::of::<check_tap>(), Duration::from_millis(100))
        .build()
        .map_err(|_| "failed to create tap timer")?;

    let _ = myrmic_sdk::info!("[adxl345-demo] MCU tap bridge started");

    Ok(())
}

#[myrmic_sdk::cmd]
fn check_tap(_md: Metadata) -> Result<()> {
    let Some(tap) = Tap::resolve("tap")? else {
        let _ = myrmic_sdk::warn!("[adxl345-demo] tap not found");
        return Ok(());
    };

    while let Some(alarm) = tap.take_event_typed::<ThresholdAlarm>()? {
        let id = runtime_id()?;

        publish(
            "tap_input",
            &TapInput {
                runtime_id: id,
                value: alarm.value,
            },
        )?;

        let _ = myrmic_sdk::info!(
            "[adxl345-demo] TAP value={:.3}g threshold={:.3}g",
            alarm.value,
            alarm.threshold
        );
    }

    Ok(())
}

#[myrmic_sdk::cmd]
fn set_led(_md: Metadata, on: bool) -> Result<()> {
    let Some(led) = Outlet::resolve("led_cmd")? else {
        return Err("led_cmd outlet not found");
    };

    led.write_typed(&DigitalState { on })?;

    let _ = myrmic_sdk::info!("[adxl345-demo] LED command={}", on);

    Ok(())
}
