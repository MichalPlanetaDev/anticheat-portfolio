use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use ac_protocol::{PlayerId, SuspicionKind, SuspicionReport, TelemetryEvent};
use ac_telemetry::read_events;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerRiskSummary {
    pub player_id: PlayerId,
    pub reports: usize,
    pub risk_score: u32,
    pub suspicion_by_kind: BTreeMap<String, usize>,
}

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

pub fn risk_summaries_from_file(path: impl AsRef<Path>) -> io::Result<Vec<PlayerRiskSummary>> {
    let events = read_events(path)?;
    Ok(risk_summaries_from_events(&events))
}

pub fn risk_summaries_from_events(events: &[TelemetryEvent]) -> Vec<PlayerRiskSummary> {
    let reports = suspicion_reports_from_events(events);
    let mut grouped: HashMap<PlayerId, Vec<SuspicionReport>> = HashMap::new();

    for report in reports {
        grouped.entry(report.player_id).or_default().push(report);
    }

    let mut summaries = grouped
        .into_iter()
        .map(|(player_id, reports)| {
            let mut suspicion_by_kind = BTreeMap::new();
            let mut raw_score = 0;

            for report in &reports {
                let kind = format!("{:?}", report.kind);
                *suspicion_by_kind.entry(kind).or_insert(0) += 1;
                raw_score += risk_weight(&report.kind);
            }

            PlayerRiskSummary {
                player_id,
                reports: reports.len(),
                risk_score: raw_score.min(100),
                suspicion_by_kind,
            }
        })
        .collect::<Vec<_>>();

    summaries.sort_by_key(|summary| summary.player_id.0);
    summaries
}

fn risk_weight(kind: &SuspicionKind) -> u32 {
    match kind {
        SuspicionKind::SpeedHack => 40,
        SuspicionKind::FireRateViolation => 25,
        SuspicionKind::InvalidStateTransition => 35,
        SuspicionKind::PacketSequenceViolation => 20,
    }
}

pub fn suspicion_reports_from_events(events: &[TelemetryEvent]) -> Vec<SuspicionReport> {
    events
        .iter()
        .filter_map(|event| match event {
            TelemetryEvent::Suspicion(report) => Some(report.clone()),
            _ => None,
        })
        .collect()
}

pub fn suspicion_reports_from_file(path: impl AsRef<Path>) -> io::Result<Vec<SuspicionReport>> {
    let events = read_events(path)?;
    Ok(suspicion_reports_from_events(&events))
}

pub fn export_suspicion_reports_csv(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> io::Result<usize> {
    let reports = suspicion_reports_from_file(input_path)?;
    write_suspicion_reports_csv(output_path, &reports)?;
    Ok(reports.len())
}

pub fn write_suspicion_reports_csv(
    output_path: impl AsRef<Path>,
    reports: &[SuspicionReport],
) -> io::Result<()> {
    let output_path = output_path.as_ref();

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    writeln!(
        writer,
        "player_id,sequence,kind,reason,observed_value,expected_limit"
    )?;

    for report in reports {
        writeln!(
            writer,
            "{},{},{},{},{:.3},{:.3}",
            report.player_id.0,
            report.sequence,
            csv_escape(&format!("{:?}", report.kind)),
            csv_escape(&report.reason),
            report.observed_value,
            report.expected_limit
        )?;
    }

    writer.flush()
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_protocol::{
        PlayerId, PlayerSnapshot, SuspicionKind, SuspicionReport, TelemetryEvent, Vec2,
    };

    fn sample_events() -> Vec<TelemetryEvent> {
        vec![
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
        ]
    }

    #[test]
    fn summarizes_telemetry_events() {
        let summary = ReplaySummary::from_events(&sample_events());

        assert_eq!(summary.total_events, 2);
        assert_eq!(summary.accepted_commands, 0);
        assert_eq!(summary.player_snapshots, 1);
        assert_eq!(summary.suspicion_reports, 1);
        assert_eq!(summary.suspicion_by_kind.get("SpeedHack"), Some(&1));
        assert!(summary.has_suspicions());
    }

    #[test]
    fn extracts_suspicion_reports() {
        let reports = suspicion_reports_from_events(&sample_events());

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].player_id, PlayerId(1));
        assert_eq!(reports[0].sequence, 42);
        assert_eq!(reports[0].kind, SuspicionKind::SpeedHack);
    }

    #[test]
    fn builds_player_risk_summary() {
        let events = vec![
            TelemetryEvent::Suspicion(SuspicionReport::new(
                PlayerId(1),
                10,
                SuspicionKind::SpeedHack,
                "speed violation",
                5.0,
                1.0,
            )),
            TelemetryEvent::Suspicion(SuspicionReport::new(
                PlayerId(1),
                11,
                SuspicionKind::FireRateViolation,
                "cooldown violation",
                100.0,
                500.0,
            )),
        ];

        let summaries = risk_summaries_from_events(&events);

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].player_id, PlayerId(1));
        assert_eq!(summaries[0].reports, 2);
        assert_eq!(summaries[0].risk_score, 65);
        assert_eq!(summaries[0].suspicion_by_kind.get("SpeedHack"), Some(&1));
        assert_eq!(
            summaries[0].suspicion_by_kind.get("FireRateViolation"),
            Some(&1)
        );
    }
}
