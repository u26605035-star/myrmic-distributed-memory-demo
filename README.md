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

## Reproducing the Cell Replacement

Deploy the distributed game first:

    myrmic deploy app_specs.yml

This deploys `player-a` to ESP32 Node A, `player-b` to ESP32 Node B, and `coordinator` to the Linux node.

To replace the application behavior on Node B without reflashing the ESP32-C5 firmware, stop the running `player-b` Cell:

    myrmic delete player-b --cell

Then deploy the replacement Cell:

    myrmic deploy app_specs_knock_light.yml

This deploys `knock-light-b` to the same ESP32-C5 Node B (`@ESP32_RUNTIME_B`). The native firmware remains running; only the Myrmic Cell is replaced.
