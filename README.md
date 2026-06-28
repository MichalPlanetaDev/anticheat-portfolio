# anticheat-portfolio

This project is built as a portfolio for multiplayer backend security and anti-cheat architecture. It does not target, modify, inspect, bypass, or interact with third-party games.

![Rust CI](https://github.com/MichalPlanetaDev/anticheat-portfolio/actions/workflows/rust-ci.yml/badge.svg)

## What this project demonstrates

- Rust workspace architecture
- Server-authoritative multiplayer simulation
- Client command validation
- Movement anomaly detection
- Weapon cooldown validation
- Packet sequence validation
- Structured JSONL telemetry
- Offline replay summaries
- Detailed investigation reports
- CLI-first Linux and WSL workflow
- CI-ready project structure

## Current status

- Rust CI passing
- Docker CI passing
- Docker Compose demo available
- Configurable TOML detection policy
- Player-level risk scoring
- CSV investigation export
- Versioned GitHub releases

## Architecture

The project supports a local investigation flow:

    ac-client-bot -> ac-server -> telemetry JSONL -> replay -> inspect -> risk -> CSV export

The CLI can summarize telemetry, inspect individual suspicion reports, calculate player risk scores, and export reports to CSV.

## Workspace modules

- crates/ac-protocol: shared command, state, telemetry, and suspicion types
- crates/ac-detectors: deterministic detection rules
- crates/ac-telemetry: JSONL telemetry writer and reader
- crates/ac-replay: replay summaries and suspicion extraction
- crates/ac-server: authoritative backend simulation
- crates/ac-client-bot: normal and suspicious command generator
- crates/ac-cli: terminal investigation tool

## Quick start

Run checks:

    cargo fmt --all
    cargo check --workspace
    cargo test --workspace

Generate command streams:

    cargo run -p ac-client-bot

Run the normal scenario:

    cargo run -p ac-server -- run samples/normal-commands.jsonl samples/normal-telemetry.jsonl
    cargo run -p ac-cli -- replay samples/normal-telemetry.jsonl

Run the suspicious scenario:

    cargo run -p ac-server -- run samples/suspicious-commands.jsonl samples/suspicious-telemetry.jsonl
    cargo run -p ac-cli -- replay samples/suspicious-telemetry.jsonl
    cargo run -p ac-cli -- inspect samples/suspicious-telemetry.jsonl

Calculate player risk:

    cargo run -p ac-cli -- risk samples/suspicious-telemetry.jsonl

Export investigation CSV:

    cargo run -p ac-cli -- export samples/suspicious-telemetry.jsonl reports/suspicious-report.csv

Run full Docker demo:

    docker compose run --rm demo

If just is installed:

    just demo

## Current detection rules

SpeedHackDetector flags movement when the observed movement distance exceeds the server-side movement budget.

FireRateDetector flags fire commands that arrive before the server-side weapon cooldown expires.

Packet sequence validation flags commands with repeated or non-increasing sequence numbers.

## Scope

This project is defensive and educational.

It does not include cheats, bypasses, injectors, malware, kernel components, commercial game reverse engineering, or instructions for attacking third-party software.

All clients, servers, binaries, command streams, and telemetry samples are created specifically for this repository.

All clients, servers, binaries, command streams, and telemetry samples are created specifically for this repository.
