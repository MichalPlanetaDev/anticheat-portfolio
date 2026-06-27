use ac_protocol::{FireSample, MovementSample, SuspicionKind, SuspicionReport};

pub trait Detector<T> {
    fn inspect(&self, sample: &T) -> Option<SuspicionReport>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SpeedHackDetector;

impl Detector<MovementSample> for SpeedHackDetector {
    fn inspect(&self, sample: &MovementSample) -> Option<SuspicionReport> {
        let observed_distance = sample.distance();
        let allowed_distance =
            sample.max_speed_units_per_second * sample.elapsed_seconds() + sample.tolerance_units;

        if observed_distance > allowed_distance {
            Some(SuspicionReport::new(
                sample.player_id,
                sample.sequence,
                SuspicionKind::SpeedHack,
                "movement distance exceeded server-authoritative speed limit",
                observed_distance,
                allowed_distance,
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FireRateDetector;

impl Detector<FireSample> for FireRateDetector {
    fn inspect(&self, sample: &FireSample) -> Option<SuspicionReport> {
        if sample.server_time_ms < sample.next_allowed_fire_time_ms {
            Some(SuspicionReport::new(
                sample.player_id,
                sample.sequence,
                SuspicionKind::FireRateViolation,
                "fire command arrived before weapon cooldown expired",
                sample.server_time_ms as f32,
                sample.next_allowed_fire_time_ms as f32,
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_protocol::{FireSample, PlayerId, Vec2};

    #[test]
    fn speed_detector_accepts_valid_movement() {
        let detector = SpeedHackDetector;

        let sample = MovementSample {
            player_id: PlayerId(1),
            sequence: 10,
            from: Vec2::new(0.0, 0.0),
            to: Vec2::new(1.0, 0.0),
            dt_ms: 100,
            max_speed_units_per_second: 10.0,
            tolerance_units: 0.1,
        };

        assert!(detector.inspect(&sample).is_none());
    }

    #[test]
    fn speed_detector_flags_impossible_movement() {
        let detector = SpeedHackDetector;

        let sample = MovementSample {
            player_id: PlayerId(1),
            sequence: 11,
            from: Vec2::new(0.0, 0.0),
            to: Vec2::new(5.0, 0.0),
            dt_ms: 100,
            max_speed_units_per_second: 10.0,
            tolerance_units: 0.1,
        };

        let report = detector.inspect(&sample).expect("expected speed violation");

        assert_eq!(report.kind, SuspicionKind::SpeedHack);
        assert_eq!(report.player_id, PlayerId(1));
        assert_eq!(report.sequence, 11);
    }

    #[test]
    fn fire_rate_detector_accepts_ready_weapon() {
        let detector = FireRateDetector;

        let sample = FireSample {
            player_id: PlayerId(2),
            sequence: 20,
            server_time_ms: 1500,
            next_allowed_fire_time_ms: 1200,
        };

        assert!(detector.inspect(&sample).is_none());
    }

    #[test]
    fn fire_rate_detector_flags_early_fire() {
        let detector = FireRateDetector;

        let sample = FireSample {
            player_id: PlayerId(2),
            sequence: 21,
            server_time_ms: 900,
            next_allowed_fire_time_ms: 1200,
        };

        let report = detector
            .inspect(&sample)
            .expect("expected fire-rate violation");

        assert_eq!(report.kind, SuspicionKind::FireRateViolation);
        assert_eq!(report.player_id, PlayerId(2));
        assert_eq!(report.sequence, 21);
    }
}
