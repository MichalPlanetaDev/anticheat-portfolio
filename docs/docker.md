# Docker

This project can be built and demonstrated inside Docker.

## Build

    docker build -t anticheat-portfolio .

## Run the full demo

    docker compose run --rm demo

The demo pipeline:

    ac-client-bot -> ac-server -> ac-cli replay -> ac-cli inspect -> ac-cli export

Generated files:

    samples/suspicious-telemetry.jsonl
    reports/suspicious-report.csv

## Shortcut commands

If `just` is installed:

    just docker-build
    just docker-demo