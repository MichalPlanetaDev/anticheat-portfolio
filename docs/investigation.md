# Investigation Workflow

The project supports a simple local investigation workflow.

Generate command streams:

    cargo run -p ac-client-bot

Run a suspicious command stream through the server:

    cargo run -p ac-server -- run samples/suspicious-commands.jsonl samples/suspicious-telemetry.jsonl

Summarize the telemetry:

    cargo run -p ac-cli -- replay samples/suspicious-telemetry.jsonl

Inspect detailed suspicion reports:

    cargo run -p ac-cli -- inspect samples/suspicious-telemetry.jsonl

The inspect command prints each suspicion report with player ID, sequence number, violation kind, reason, observed value, and expected limit.
