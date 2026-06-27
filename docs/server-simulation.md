# Server Simulation

The `ac-server` crate demonstrates a small server-authoritative multiplayer simulation.

The client sends commands. The server owns the real player state, advances time in fixed ticks, validates suspicious client claims, applies detection rules, and records telemetry events.

Current behavior:

- movement commands are accepted as input, not final truth
- claimed client positions are validated against server-side speed limits
- fire commands are validated against server-side weapon cooldowns
- command sequence numbers must increase
- suspicious behavior is stored as telemetry

Run:

    cargo run -p ac-server
