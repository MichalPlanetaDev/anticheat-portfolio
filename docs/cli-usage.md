# CLI Usage

The `ac-cli` crate provides a terminal interface for running local anti-cheat detection examples.

## Help

Command:
    cargo run -p ac-cli -- help

## Movement speed detection

Command:
    cargo run -p ac-cli -- speed-check

This command runs normal and suspicious movement samples through `SpeedHackDetector`.

## Fire-rate detection

Command:
    cargo run -p ac-cli -- fire-check

This command runs weapon cooldown samples through `FireRateDetector`.

## Demo report

Command:
    cargo run -p ac-cli -- demo-report

This command runs multiple detector examples and prints a short investigation report.
