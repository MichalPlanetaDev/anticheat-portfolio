# Telemetry

The `ac-telemetry` crate writes and reads JSONL telemetry.

JSONL means one JSON event per line. This is useful for investigation because telemetry can be streamed, searched, replayed, and processed line by line.

Current telemetry event types:

- accepted client commands
- server-authoritative player snapshots
- suspicion reports

Example output file:

    samples/server-session.jsonl

Generate it with:

    cargo run -p ac-server

Inspect it with:

    head -n 5 samples/server-session.jsonl
