# anticheat-portfolio

## Short description

`anticheat-portfolio` is a Rust-based defensive anticheat portfolio project focused on server-auth validation, telemetry, replay analysis, CLI investigation, Dockerized demo workflows, configurable detection policy, and player-level risk scoring.

## Technical summary

The project simulates a backend validation pipeline where a client bot generates normal and suspicious command streams, an authoritative server validates movement/fire/sequence behavior, telemetry is written as JSONL, and CLI tools replay, inspect, score, and export investigation results.

Pipeline:

    ac-client-bot -> command JSONL -> ac-server -> telemetry JSONL -> ac-cli replay / inspect / risk / export

## What I built

- Rust workspace split into protocol, detectors, telemetry, replay, server, client bot, and CLI crates
- Server-authoritative movement validation
- Fire-rate cooldown validation
- Packet sequence validation
- JSONL telemetry writer and reader
- Replay summaries for saved telemetry
- Detailed suspicion inspection command
- Player-level risk scoring
- CSV export for investigation reports
- TOML-based configurable detection policy
- Dockerfile and Docker Compose demo
- Rust CI and Docker CI with GitHub Actions

## Tools and technologies

- Rust
- Cargo workspaces
- Serde / JSONL / TOML
- Docker
- Docker Compose
- GitHub Actions
- Git / GitHub
- Linux / WSL
- CLI-first workflow

## Example commands

Generate command streams:

    cargo run -p ac-client-bot

Run suspicious commands through the server:

    cargo run -p ac-server -- run samples/suspicious-commands.jsonl samples/suspicious-telemetry.jsonl config/default.toml

Replay telemetry:

    cargo run -p ac-cli -- replay samples/suspicious-telemetry.jsonl

Inspect suspicion reports:

    cargo run -p ac-cli -- inspect samples/suspicious-telemetry.jsonl

Calculate player risk:

    cargo run -p ac-cli -- risk samples/suspicious-telemetry.jsonl

Export CSV report:

    cargo run -p ac-cli -- export samples/suspicious-telemetry.jsonl reports/suspicious-report.csv

Run the Docker demo:

    docker compose run --rm demo

## Recruiter-facing description

Built a Rust defensive anticheat backend portfolio project demonstrating server-auth validation, command stream generation, telemetry logging, replay analysis, detailed investigation tooling, player risk scoring, CSV export, Dockerized demo execution, and CI validation through GitHub Actions.

## Scope note

This is a defensive and educational portfolio project. It does not include cheats, bypasses, injectors, malware, commercial game reverse engineering, or instructions for attacking third-party software.