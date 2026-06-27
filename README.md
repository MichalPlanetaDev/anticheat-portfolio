# Anti-Cheat Portfolio

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

## Architecture

Pipeline:

    ac-client-bot -> command JSONL -> ac-server -> telemetry JSONL -> ac-cli replay / inspect

The client bot generates controlled command streams. The server processes commands as input, not truth. The server owns the authoritative state, applies validation rules, records telemetry, and writes JSONL investigation logs. The CLI replays and inspects telemetry files.

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

If just is installed:

    just demo

## Current detection rules

SpeedHackDetector flags movement when the observed movement distance exceeds the server-side movement budget.

FireRateDetector flags fire commands that arrive before the server-side weapon cooldown expires.

Packet sequence validation flags commands with repeated or non-increasing sequence numbers.

It does not include cheats, bypasses, injectors, malware, kernel components, commercial game reverse engineering, or instructions for attacking third-party software.

All clients, servers, binaries, command streams, and telemetry samples are created specifically for this repository.
