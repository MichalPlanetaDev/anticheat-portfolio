# Risk Scoring

`anticheat-portfolio` can aggregate suspicion reports into a player-level risk summary.

Run:

    cargo run -p ac-cli -- risk samples/suspicious-telemetry.jsonl

The risk command groups suspicion reports by player and calculates a capped risk score.

Current weights:

    SpeedHack = 40
    FireRateViolation = 25
    InvalidStateTransition = 35
    PacketSequenceViolation = 20

The score is capped at 100.

This is not a production scoring model. It is a portfolio implementation that demonstrates how raw detection events can be converted into triage-oriented investigation output.