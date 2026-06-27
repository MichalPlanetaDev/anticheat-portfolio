# Replay

The `ac-replay` crate reads saved JSONL telemetry and summarizes investigation results.

This is useful because anti-cheat work is rarely only live detection. Suspicious sessions also need to be replayed, inspected, counted, and compared.

Generate telemetry:

    cargo run -p ac-server

Replay telemetry:

    cargo run -p ac-cli -- replay samples/server-session.jsonl

The replay summary reports:

- total telemetry events
- accepted commands
- player snapshots
- suspicion reports
- suspicion count grouped by kind
