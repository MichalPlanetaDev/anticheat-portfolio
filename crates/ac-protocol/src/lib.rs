use serde::{Deserialize, Serialize};

pub type SequenceNumber = u64;
pub type Milliseconds = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn distance(self, other: Self) -> f32 {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
        .length()
    }

    pub fn normalized(self) -> Self {
        let length = self.length();

        if length <= f32::EPSILON {
            Self::ZERO
        } else {
            Self {
                x: self.x / length,
                y: self.y / length,
            }
        }
    }

    pub fn scaled(self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MovementCommand {
    pub player_id: PlayerId,
    pub sequence: SequenceNumber,
    pub client_time_ms: Milliseconds,
    pub direction: Vec2,
    pub dt_ms: Milliseconds,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FireCommand {
    pub player_id: PlayerId,
    pub sequence: SequenceNumber,
    pub client_time_ms: Milliseconds,
    pub aim_direction: Vec2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientCommand {
    Move(MovementCommand),
    Fire(FireCommand),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub player_id: PlayerId,
    pub position: Vec2,
    pub health: i32,
    pub alive: bool,
    pub server_time_ms: Milliseconds,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MovementSample {
    pub player_id: PlayerId,
    pub sequence: SequenceNumber,
    pub from: Vec2,
    pub to: Vec2,
    pub dt_ms: Milliseconds,
    pub max_speed_units_per_second: f32,
    pub tolerance_units: f32,
}

impl MovementSample {
    pub fn elapsed_seconds(&self) -> f32 {
        self.dt_ms as f32 / 1000.0
    }

    pub fn distance(&self) -> f32 {
        self.from.distance(self.to)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FireSample {
    pub player_id: PlayerId,
    pub sequence: SequenceNumber,
    pub server_time_ms: Milliseconds,
    pub next_allowed_fire_time_ms: Milliseconds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuspicionKind {
    SpeedHack,
    FireRateViolation,
    InvalidStateTransition,
    PacketSequenceViolation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuspicionReport {
    pub player_id: PlayerId,
    pub sequence: SequenceNumber,
    pub kind: SuspicionKind,
    pub reason: String,
    pub observed_value: f32,
    pub expected_limit: f32,
}

impl SuspicionReport {
    pub fn new(
        player_id: PlayerId,
        sequence: SequenceNumber,
        kind: SuspicionKind,
        reason: impl Into<String>,
        observed_value: f32,
        expected_limit: f32,
    ) -> Self {
        Self {
            player_id,
            sequence,
            kind,
            reason: reason.into(),
            observed_value,
            expected_limit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TelemetryEvent {
    CommandAccepted(ClientCommand),
    PlayerSnapshot(PlayerSnapshot),
    Suspicion(SuspicionReport),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_vec2_distance() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);

        assert_eq!(a.distance(b), 5.0);
    }

    #[test]
    fn normalizes_non_zero_vector() {
        let vector = Vec2::new(10.0, 0.0).normalized();

        assert_eq!(vector, Vec2::new(1.0, 0.0));
    }

    #[test]
    fn zero_vector_stays_zero_when_normalized() {
        assert_eq!(Vec2::ZERO.normalized(), Vec2::ZERO);
    }
}
