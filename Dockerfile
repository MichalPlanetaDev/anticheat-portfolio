FROM rust:1-bookworm AS builder

WORKDIR /app

COPY . .

RUN cargo build --workspace --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/ac-cli /usr/local/bin/ac-cli
COPY --from=builder /app/target/release/ac-server /usr/local/bin/ac-server
COPY --from=builder /app/target/release/ac-client-bot /usr/local/bin/ac-client-bot

CMD ["ac-cli", "help"]