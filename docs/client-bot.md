# Client Bot

The `ac-client-bot` crate generates local JSONL command streams.

It is not a cheat tool. It only creates controlled test data for this repository.

Generate both scenarios:

    cargo run -p ac-client-bot

Generate only normal commands:

    cargo run -p ac-client-bot -- normal

Generate suspicious commands:

    cargo run -p ac-client-bot -- suspicious

Run generated commands through the server:

    cargo run -p ac-server -- run samples/normal-commands.jsonl samples/normal-telemetry.jsonl

    cargo run -p ac-server -- run samples/suspicious-commands.jsonl samples/suspicious-telemetry.jsonl

Replay the telemetry:

    cargo run -p ac-cli -- replay samples/normal-telemetry.jsonl

    cargo run -p ac-cli -- replay samples/suspicious-telemetry.jsonl
