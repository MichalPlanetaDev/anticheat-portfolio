# Detection Rules

This project begins with simple deterministic server-side detection rules.

## SpeedHackDetector

Flags movement when the observed distance between two server-authoritative positions exceeds:

max_speed_units_per_second * elapsed_seconds + tolerance_units

The client does not decide its final position. The server validates movement against the allowed speed budget.

## FireRateDetector

Flags fire commands that arrive before the server-side weapon cooldown has expired.

The client may request to fire, but the server decides whether the weapon is ready.
