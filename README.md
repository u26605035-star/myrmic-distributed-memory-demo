# Myrmic ESP32-C5 Distributed Demo

This repository contains a distributed Myrmic demo using two ESP32-C5 nodes, two ADXL345 accelerometers, LEDs, and a Linux node.

## Architecture

The Myrmic swarm consists of:

- ESP32-C5 Node A
- ESP32-C5 Node B
- Linux node

Each ESP32-C5 has an ADXL345 accelerometer and an external LED.

Native MCU firmware handles hardware access, sensor acquisition, threshold processing, and LED output through the Myrmic Signal Layer. Application behavior is implemented separately as deployable Myrmic Cells.

## Memory / Sequence Game

The main application is a distributed memory/sequence game.

`game-player` runs on each ESP32-C5 and consumes ADXL345 tap events.

`game-coordinator` runs on the Linux node and maintains the game state, generates the sequence, controls LED playback, and validates player inputs.

## Cell Replacement Demo

`knock-light` demonstrates changing MCU application behavior without reflashing the native ESP32-C5 firmware.

The `player-b` Cell can be removed from Node B and replaced with `knock-light-b` on the same running MCU.

The replacement Cell uses the same ADXL345 and LED hardware but implements different behavior: detected knocks toggle the LED.

## Repository Structure

- `firmware/esp32c5-adxl345` - ESP32-C5 native firmware and Signal Layer configuration
- `cells/game-player` - MCU memory-game Cell
- `cells/game-coordinator` - Linux game coordinator
- `cells/knock-light` - replacement MCU Cell

## Hardware

Each MCU node uses:

- ESP32-C5 development board
- ADXL345 accelerometer
- External LED with current-limiting resistor

Current configuration:

- I2C SCL: GPIO23
- I2C SDA: GPIO24
- LED: GPIO6
- ADXL345 address: 0x53

## Demonstrated Functionality

- Three-node Myrmic swarm
- Two ESP32-C5 MCU nodes
- Linux node
- ADXL345 acquisition
- Native threshold processing
- MCU-to-Linux events
- Linux-to-MCU interaction
- Distributed memory/sequence game
- RISC-V AOT Cell deployment
- MCU Cell replacement without native firmware reflashing

## Application Placement

The application specifications contain the placeholders
`@ESP32_RUNTIME_A` and `@ESP32_RUNTIME_B`.

Before deployment, replace these placeholders with the system runtime tags
reported for the two ESP32-C5 runtimes. Myrmic runtime system tags have the
form `@<runtime-id>`.

For example:

    @ESP32_RUNTIME_A -> @<runtime-id-of-node-a>
    @ESP32_RUNTIME_B -> @<runtime-id-of-node-b>

The application specifications explicitly select `riscv32imac` for MCU Cells
and `linux` for the coordinator. This causes Myrmic to generate RISC-V AOT
artifacts for the ESP32-C5 Cells while building the coordinator for Linux.

## Reproducing the Cell Replacement

First deploy the distributed game:

    myrmic deploy app_specs.yml --timeout 30s

This deploys two `mcu-player` Cells to the two ESP32-C5 runtimes and one
`game-coordinator` Cell to the Linux runtime.

Inspect the deployed Cells:

    myrmic cells --once

Identify the SRI of the `mcu-player` running on ESP32 Node B. Remove only that
Cell:

    myrmic delete <PLAYER_B_SRI> --cell --timeout 30s

The player on Node A and the Linux coordinator remain deployed.

Then deploy the replacement application:

    myrmic deploy app_specs_knock_light.yml --timeout 30s

Inspect the swarm again:

    myrmic cells --once

The resulting deployment consists of:

- `mcu-player` on ESP32-C5 Node A
- `knock-light` on ESP32-C5 Node B
- `game-coordinator` on the Linux node

No native ESP32-C5 firmware reflash is required during this replacement. The
running MCU firmware remains in place while the deployable Myrmic Cell on Node
B is changed from the game player to `knock-light`.
