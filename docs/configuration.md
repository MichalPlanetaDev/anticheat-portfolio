# Configuration

`anticheat-portfolio` uses a TOML detection policy file for server-side validation thresholds.

Default config:

    config/default.toml

Current fields:

    max_speed_units_per_second = 10.0
    movement_tolerance_units = 0.15
    fixed_tick_ms = 100
    fire_cooldown_ms = 500

Run with the default config:

    cargo run -p ac-server -- run samples/suspicious-commands.jsonl samples/suspicious-telemetry.jsonl

Run with an explicit config path:

    cargo run -p ac-server -- run samples/suspicious-commands.jsonl samples/suspicious-telemetry.jsonl config/default.toml

The goal is to keep detection thresholds operationally configurable instead of hardcoding them into the server binary.