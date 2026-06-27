use std::{collections::BTreeMap, io, path::Path};

use ac_protocol::TelemetryEvent;
use ac_telemetry::read_events;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaySummary {
    pub total_events: usize,
    pub accepted_commands: usize,
    pub player_snapshots: usize,
    pub suspicion_reports: usize,
    pub suspicion_by_kind: BTreeMap<String, usize>,
}

impl ReplaySummary {
    pub fn from_events(events: &[TelemetryEvent]) -> Self {
        let mut accepted_commands = 0;
        let mut player_snapshots = 0;
        let mut suspicion_reports = 0;
        let mut suspicion_by_kind = BTreeMap::new();

        for event in events {
            match event {
                TelemetryEvent::CommandAccepted(_) => {
                    accepted_commands += 1;
                }
                TelemetryEvent::PlayerSnapshot(_) => {
                    player_snapshots += 1;
                }
                TelemetryEvent::Suspicion(report) => {
                    suspicion_reports += 1;
                    let kind = format!("{:?}", report.kind);
                    *suspicion_by_kind.entry(kind).or_insert(0) += 1;
                }
            }
        }

        Self {
            total_events: events.len(),
            accepted_commands,
            player_snapshots,
            suspicion_reports,
            suspicion_by_kind,
        }
    }

    pub fn has_suspicions(&self) -> bool {
        self.suspicion_reports > 0
    }
}

pub fn summarize_file(path: impl AsRef<Path>) -> io::Result<ReplaySummary> {
    let events = read_events(path)?;
    Ok(ReplaySummary::from_events(&events))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_protocol::{
        PlayerId, PlayerSnapshot, SuspicionKind, SuspicionReport, TelemetryEvent, Vec2,
    };

    #[test]
    fn summarizes_telemetry_events() {
        let events = vec![
            TelemetryEvent::PlayerSnapshot(PlayerSnapshot {
                player_id: PlayerId(1),
                position: Vec2::new(1.0, 2.0),
                health: 100,
                alive: true,
                server_time_ms: 100,
            }),
            TelemetryEvent::Suspicion(SuspicionReport::new(
                PlayerId(1),
                42,
                SuspicionKind::SpeedHack,
                "test suspicion",
                5.0,
                1.0,
            )),
        ];

        let summary = ReplaySummary::from_events(&events);

        assert_eq!(summary.total_events, 2);
        assert_eq!(summary.accepted_commands, 0);
        assert_eq!(summary.player_snapshots, 1);
        assert_eq!(summary.suspicion_reports, 1);
        assert_eq!(summary.suspicion_by_kind.get("SpeedHack"), Some(&1));
        assert!(summary.has_suspicions());
    }
}
