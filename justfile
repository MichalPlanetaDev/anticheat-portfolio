fmt:
    cargo fmt --all

check:
    cargo check --workspace

test:
    cargo test --workspace

ci: fmt check test

bot:
    cargo run -p ac-client-bot

run-normal:
    cargo run -p ac-server -- run samples/normal-commands.jsonl samples/normal-telemetry.jsonl

run-suspicious:
    cargo run -p ac-server -- run samples/suspicious-commands.jsonl samples/suspicious-telemetry.jsonl

replay-normal:
    cargo run -p ac-cli -- replay samples/normal-telemetry.jsonl

replay-suspicious:
    cargo run -p ac-cli -- replay samples/suspicious-telemetry.jsonl

inspect-suspicious:
    cargo run -p ac-cli -- inspect samples/suspicious-telemetry.jsonl

demo: bot run-normal replay-normal run-suspicious replay-suspicious inspect-suspicious
