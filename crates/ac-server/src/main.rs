use std::collections::HashMap;

use ac_detectors::{Detector, FireRateDetector, SpeedHackDetector};
use ac_protocol::{
    ClientCommand, FireCommand, FireSample, MovementCommand, MovementSample, PlayerId,
    PlayerSnapshot, SuspicionKind, SuspicionReport, TelemetryEvent, Vec2,
};
use ac_telemetry::TelemetryWriter;

const MAX_SPEED_UNITS_PER_SECOND: f32 = 10.0;
const MOVEMENT_TOLERANCE_UNITS: f32 = 0.15;
const FIXED_TICK_MS: u64 = 100;
const FIRE_COOLDOWN_MS: u64 = 500;

#[derive(Debug, Clone)]
struct PlayerState {
    position: Vec2,
    health: i32,
    alive: bool,
    next_allowed_fire_time_ms: u64,
    last_sequence: u64,
}

impl PlayerState {
    fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            health: 100,
            alive: true,
            next_allowed_fire_time_ms: 0,
            last_sequence: 0,
        }
    }

    fn snapshot(&self, player_id: PlayerId, server_time_ms: u64) -> PlayerSnapshot {
        PlayerSnapshot {
            player_id,
            position: self.position,
            health: self.health,
            alive: self.alive,
            server_time_ms,
        }
    }
}

struct GameServer {
    players: HashMap<PlayerId, PlayerState>,
    server_time_ms: u64,
    speed_detector: SpeedHackDetector,
    fire_detector: FireRateDetector,
    telemetry: Vec<TelemetryEvent>,
}

impl GameServer {
    fn new() -> Self {
        Self {
            players: HashMap::new(),
            server_time_ms: 0,
            speed_detector: SpeedHackDetector,
            fire_detector: FireRateDetector,
            telemetry: Vec::new(),
        }
    }

    fn add_player(&mut self, player_id: PlayerId) {
        self.players.insert(player_id, PlayerState::new());
    }

    fn process_command(&mut self, command: ClientCommand) {
        self.server_time_ms += FIXED_TICK_MS;

        match command {
            ClientCommand::Move(command) => self.process_move(command),
            ClientCommand::Fire(command) => self.process_fire(command),
        }
    }

    fn process_move(&mut self, command: MovementCommand) {
        if !self.players.contains_key(&command.player_id) {
            self.add_player(command.player_id);
        }

        let current = self
            .players
            .get(&command.player_id)
            .expect("player must exist")
            .clone();

        if let Some(report) = self.sequence_violation(command.player_id, command.sequence) {
            self.record_report(report);
            return;
        }

        if !current.alive {
            self.record_report(SuspicionReport::new(
                command.player_id,
                command.sequence,
                SuspicionKind::InvalidStateTransition,
                "dead player attempted to move",
                1.0,
                0.0,
            ));
            return;
        }

        if let Some(claimed_position) = command.claimed_position {
            let sample = MovementSample {
                player_id: command.player_id,
                sequence: command.sequence,
                from: current.position,
                to: claimed_position,
                dt_ms: FIXED_TICK_MS,
                max_speed_units_per_second: MAX_SPEED_UNITS_PER_SECOND,
                tolerance_units: MOVEMENT_TOLERANCE_UNITS,
            };

            if let Some(report) = self.speed_detector.inspect(&sample) {
                self.record_report(report);
            }
        }

        let distance_budget = MAX_SPEED_UNITS_PER_SECOND * (FIXED_TICK_MS as f32 / 1000.0);
        let movement = command.direction.normalized().scaled(distance_budget);

        let state = self
            .players
            .get_mut(&command.player_id)
            .expect("player must exist");

        state.position = Vec2::new(
            current.position.x + movement.x,
            current.position.y + movement.y,
        );
        state.last_sequence = command.sequence;

        let snapshot = state.snapshot(command.player_id, self.server_time_ms);

        self.telemetry
            .push(TelemetryEvent::CommandAccepted(ClientCommand::Move(
                command,
            )));
        self.telemetry
            .push(TelemetryEvent::PlayerSnapshot(snapshot));
    }

    fn process_fire(&mut self, command: FireCommand) {
        if !self.players.contains_key(&command.player_id) {
            self.add_player(command.player_id);
        }

        let current = self
            .players
            .get(&command.player_id)
            .expect("player must exist")
            .clone();

        if let Some(report) = self.sequence_violation(command.player_id, command.sequence) {
            self.record_report(report);
            return;
        }

        if !current.alive {
            self.record_report(SuspicionReport::new(
                command.player_id,
                command.sequence,
                SuspicionKind::InvalidStateTransition,
                "dead player attempted to fire",
                1.0,
                0.0,
            ));
            return;
        }

        let sample = FireSample {
            player_id: command.player_id,
            sequence: command.sequence,
            server_time_ms: self.server_time_ms,
            next_allowed_fire_time_ms: current.next_allowed_fire_time_ms,
        };

        if let Some(report) = self.fire_detector.inspect(&sample) {
            self.record_report(report);
            return;
        }

        let state = self
            .players
            .get_mut(&command.player_id)
            .expect("player must exist");

        state.last_sequence = command.sequence;
        state.next_allowed_fire_time_ms = self.server_time_ms + FIRE_COOLDOWN_MS;

        self.telemetry
            .push(TelemetryEvent::CommandAccepted(ClientCommand::Fire(
                command,
            )));
    }

    fn sequence_violation(&self, player_id: PlayerId, sequence: u64) -> Option<SuspicionReport> {
        let state = self.players.get(&player_id)?;

        if sequence <= state.last_sequence {
            Some(SuspicionReport::new(
                player_id,
                sequence,
                SuspicionKind::PacketSequenceViolation,
                "command sequence number did not increase",
                sequence as f32,
                (state.last_sequence + 1) as f32,
            ))
        } else {
            None
        }
    }

    fn record_report(&mut self, report: SuspicionReport) {
        self.telemetry.push(TelemetryEvent::Suspicion(report));
    }

    fn write_telemetry_jsonl(&self, path: &str) -> std::io::Result<()> {
        let mut writer = TelemetryWriter::create(path)?;
        writer.write_events(&self.telemetry)?;
        writer.flush()
    }

    fn print_telemetry(&self) {
        println!("Authoritative server simulation");
        println!();
        println!("Server time: {} ms", self.server_time_ms);
        println!("Telemetry events: {}", self.telemetry.len());
        println!();

        for event in &self.telemetry {
            match event {
                TelemetryEvent::CommandAccepted(command) => {
                    println!("[ACCEPTED] {:?}", command);
                }
                TelemetryEvent::PlayerSnapshot(snapshot) => {
                    println!(
                        "[SNAPSHOT] player={:?} pos=({:.2}, {:.2}) health={} alive={} time={}ms",
                        snapshot.player_id,
                        snapshot.position.x,
                        snapshot.position.y,
                        snapshot.health,
                        snapshot.alive,
                        snapshot.server_time_ms
                    );
                }
                TelemetryEvent::Suspicion(report) => {
                    println!(
                        "[FLAGGED] player={:?} seq={} kind={:?} observed={:.3} limit={:.3}",
                        report.player_id,
                        report.sequence,
                        report.kind,
                        report.observed_value,
                        report.expected_limit
                    );
                    println!("          reason={}", report.reason);
                }
            }
        }
    }
}

fn main() {
    let mut server = GameServer::new();
    let player_id = PlayerId(1);

    server.add_player(player_id);

    let commands = vec![
        ClientCommand::Move(MovementCommand {
            player_id,
            sequence: 1,
            client_time_ms: 100,
            direction: Vec2::new(1.0, 0.0),
            dt_ms: 100,
            claimed_position: Some(Vec2::new(1.0, 0.0)),
        }),
        ClientCommand::Move(MovementCommand {
            player_id,
            sequence: 2,
            client_time_ms: 200,
            direction: Vec2::new(1.0, 0.0),
            dt_ms: 100,
            claimed_position: Some(Vec2::new(20.0, 0.0)),
        }),
        ClientCommand::Fire(FireCommand {
            player_id,
            sequence: 3,
            client_time_ms: 300,
            aim_direction: Vec2::new(1.0, 0.0),
        }),
        ClientCommand::Fire(FireCommand {
            player_id,
            sequence: 4,
            client_time_ms: 400,
            aim_direction: Vec2::new(1.0, 0.0),
        }),
    ];

    for command in commands {
        server.process_command(command);
    }

    server.print_telemetry();

    let telemetry_path = "samples/server-session.jsonl";

    if let Err(error) = server.write_telemetry_jsonl(telemetry_path) {
        eprintln!("failed to write telemetry: {error}");
        std::process::exit(1);
    }

    println!();
    println!("Wrote telemetry: {telemetry_path}");
}
