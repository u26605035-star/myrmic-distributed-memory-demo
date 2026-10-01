#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;

use myrmic_sdk::db::state::State;
use myrmic_sdk::{Callback, Metadata, Result, Sri, send};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, myrmic_sdk::Message)]
#[codec(myrmic_sdk::Postcard)]
struct TapInput {
    runtime_id: String,
    value: f32,
}

#[derive(Serialize, Deserialize, Default)]
struct GameState {
    players: Vec<Sri>,
    sequence: Vec<u8>,
    position: usize,
    round: usize,
    playback_index: usize,
    playback_led_on: bool,
    playback_active: bool,
}

const GAME: State<'static, GameState> = State::new_const("game");

#[myrmic_sdk::init]
fn init(_md: Metadata) -> Result<()> {
    let state = GAME.load()?.unwrap_or_default();
    GAME.save(&state)?;

    let _ = myrmic_sdk::info!("[game] coordinator started");

    Ok(())
}

#[myrmic_sdk::evt]
fn tap_input(md: Metadata, tap: TapInput) -> Result<()> {
    let mut state = GAME.load()?.unwrap_or_default();

    let Some(player) = state.players.iter().position(|sri| *sri == md.sender) else {
        if state.players.len() < 2 {
            state.players.push(md.sender);
            let index = state.players.len() - 1;

            let _ = myrmic_sdk::info!(
                "[game] registered player={} runtime={}",
                index,
                tap.runtime_id
            );

            if state.players.len() == 2 && state.sequence.is_empty() {
                start_round(&mut state)?;
            }

            GAME.save(&state)?;
            return Ok(());
        }

        let _ = myrmic_sdk::warn!("[game] ignoring tap from unregistered cell");
        return Ok(());
    };

    if state.players.len() < 2 {
        GAME.save(&state)?;
        return Ok(());
    }

    if state.playback_active {
        let _ = myrmic_sdk::info!(
            "[game] ignoring player={} tap during sequence playback",
            player
        );
        return Ok(());
    }

    if state.sequence.is_empty() {
        start_round(&mut state)?;
        GAME.save(&state)?;
        return Ok(());
    }

    let expected = state.sequence[state.position] as usize;

    let _ = myrmic_sdk::info!(
        "[game] tap player={} expected={} position={} value={:.3}g",
        player,
        expected,
        state.position,
        tap.value
    );

    if player == expected {
        state.position += 1;

        if state.position >= state.sequence.len() {
            state.round += 1;
            state.position = 0;
            extend_sequence(&mut state);

            let _ = myrmic_sdk::info!(
                "[game] round complete; starting round={} length={}",
                state.round,
                state.sequence.len()
            );

            show_sequence(&mut state)?;
        }
    } else {
        let _ = myrmic_sdk::warn!(
            "[game] wrong player={} expected={}; restarting sequence",
            player,
            expected
        );

        state.position = 0;
        show_sequence(&mut state)?;
    }

    GAME.save(&state)
}

fn start_round(state: &mut GameState) -> Result<()> {
    state.round = 1;
    state.position = 0;
    state.sequence.clear();
    state.sequence.push(0);

    let _ = myrmic_sdk::info!("[game] both players ready; starting game");

    show_sequence(state)
}

fn extend_sequence(state: &mut GameState) {
    // Deterministic sequence for the demo:
    // A, A-B, A-B-B, A-B-B-A, ...
    const PATTERN: [u8; 8] = [0, 1, 1, 0, 1, 0, 0, 1];

    let next = PATTERN[state.sequence.len() % PATTERN.len()];
    state.sequence.push(next);
}

fn show_sequence(state: &mut GameState) -> Result<()> {
    state.playback_index = 0;
    state.playback_led_on = false;
    state.playback_active = true;

    GAME.save(state)?;
    playback_step(state)
}

fn playback_step(state: &mut GameState) -> Result<()> {
    if !state.playback_active {
        return Ok(());
    }

    if state.playback_index >= state.sequence.len() {
        state.playback_active = false;
        state.playback_led_on = false;
        GAME.save(state)?;

        let _ = myrmic_sdk::info!("[game] sequence playback complete");
        return Ok(());
    }

    let player = state.sequence[state.playback_index] as usize;

    if state.playback_led_on {
        set_player_led(state, player, false)?;
        state.playback_led_on = false;
        state.playback_index += 1;
        GAME.save(state)?;

        let _ = myrmic_sdk::delay(Callback::of::<playback_tick>(), Duration::from_millis(200))
            .build()
            .map_err(|_| "failed to create playback timer")?;
    } else {
        set_player_led(state, player, true)?;
        state.playback_led_on = true;
        GAME.save(state)?;

        let _ = myrmic_sdk::delay(Callback::of::<playback_tick>(), Duration::from_millis(400))
            .build()
            .map_err(|_| "failed to create playback timer")?;
    }

    Ok(())
}

#[myrmic_sdk::cmd]
fn playback_tick(_md: Metadata) -> Result<()> {
    let mut state = GAME.load()?.unwrap_or_default();
    playback_step(&mut state)
}

#[myrmic_sdk::cmd]
fn reset_game(_md: Metadata) -> Result<()> {
    let state = GAME.load()?.unwrap_or_default();

    for player in 0..state.players.len() {
        set_player_led(&state, player, false)?;
    }

    GAME.save(&GameState::default())?;

    let _ = myrmic_sdk::info!("[game] game reset; waiting for two players");

    Ok(())
}

fn set_player_led(state: &GameState, player: usize, on: bool) -> Result<()> {
    let Some(sri) = state.players.get(player) else {
        return Err("player not registered");
    };

    send(*sri, "set_led", &on)
}
