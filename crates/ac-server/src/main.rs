use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use ac_detectors::{Detector, FireRateDetector, SpeedHackDetector};
use ac_protocol::{
    ClientCommand, FireCommand, FireSample, MovementCommand, MovementSample, PlayerId,
    PlayerSnapshot, SuspicionKind, SuspicionReport, TelemetryEvent, Vec2,
};
use ac_telemetry::TelemetryWriter;

const DEFAULT_CONFIG_PATH: &str = "config/default.toml";

#[derive(Debug, Clone, Copy, serde::Deserialize)]
struct DetectionPolicy {
    max_speed_units_per_second: f32,
    movement_tolerance_units: f32,
    fixed_tick_ms: u64,
    fire_cooldown_ms: u64,
}

impl DetectionPolicy {
    fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        toml::from_str(&content).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse config '{}': {}", path.display(), error),
            )
        })
    }
}

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
    policy: DetectionPolicy,
    speed_detector: SpeedHackDetector,
    fire_detector: FireRateDetector,
    telemetry: Vec<TelemetryEvent>,
}

impl GameServer {
    fn new(policy: DetectionPolicy) -> Self {
        Self {
            players: HashMap::new(),
            server_time_ms: 0,
            policy,
            speed_detector: SpeedHackDetector,
            fire_detector: FireRateDetector,
            telemetry: Vec::new(),
        }
    }

    fn add_player(&mut self, player_id: PlayerId) {
        self.players
            .entry(player_id)
            .or_insert_with(PlayerState::new);
    }

    fn process_command(&mut self, command: ClientCommand) {
        self.server_time_ms += self.policy.fixed_tick_ms;

        match command {
            ClientCommand::Move(command) => self.process_move(command),
            ClientCommand::Fire(command) => self.process_fire(command),
        }
    }

    fn process_move(&mut self, command: MovementCommand) {
        self.add_player(command.player_id);

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
                dt_ms: self.policy.fixed_tick_ms,
                max_speed_units_per_second: self.policy.max_speed_units_per_second,
                tolerance_units: self.policy.movement_tolerance_units,
            };

            if let Some(report) = self.speed_detector.inspect(&sample) {
                self.record_report(report);
            }
        }

        let distance_budget =
            self.policy.max_speed_units_per_second * (self.policy.fixed_tick_ms as f32 / 1000.0);
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
        self.add_player(command.player_id);

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
        state.next_allowed_fire_time_ms = self.server_time_ms + self.policy.fire_cooldown_ms;

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

    fn write_telemetry_jsonl(&self, path: &str) -> io::Result<()> {
        let mut writer = TelemetryWriter::create(path)?;
        writer.write_events(&self.telemetry)?;
        writer.flush()
    }

    fn print_telemetry(&self) {
        println!("anticheat-portfolio server simulation");
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
    if let Err(error) = run() {
        eprintln!("server failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        None | Some("demo") => {
            let config_path = args
                .get(1)
                .map(String::as_str)
                .unwrap_or(DEFAULT_CONFIG_PATH);

            run_scenario(demo_commands(), "samples/server-session.jsonl", config_path)?;
        }
        Some("run") => {
            let command_path = args.get(1).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "missing command JSONL path after 'run'",
                )
            })?;

            let telemetry_path = args
                .get(2)
                .map(String::as_str)
                .unwrap_or("samples/server-session.jsonl");

            let config_path = args
                .get(3)
                .map(String::as_str)
                .unwrap_or(DEFAULT_CONFIG_PATH);

            let commands = read_commands_jsonl(command_path)?;
            run_scenario(commands, telemetry_path, config_path)?;
        }
        Some("help") | Some("--help") | Some("-h") => {
            print_help();
        }
        Some(unknown) => {
            eprintln!("unknown command: {unknown}");
            eprintln!();
            print_help();
            std::process::exit(2);
        }
    }

    Ok(())
}

fn print_help() {
    println!("anticheat-portfolio server simulation");
    println!();
    println!("Usage:");
    println!("  cargo run -p ac-server -- demo [config-toml]");
    println!("  cargo run -p ac-server -- run <commands-jsonl> [telemetry-jsonl] [config-toml]");
}

fn run_scenario(
    commands: Vec<ClientCommand>,
    telemetry_path: &str,
    config_path: &str,
) -> io::Result<()> {
    let policy = DetectionPolicy::load(config_path)?;
    let mut server = GameServer::new(policy);

    println!("Loaded detection policy: {config_path}");
    println!(
        "max_speed={} tolerance={} tick_ms={} fire_cooldown_ms={}",
        policy.max_speed_units_per_second,
        policy.movement_tolerance_units,
        policy.fixed_tick_ms,
        policy.fire_cooldown_ms
    );
    println!();

    for command in commands {
        server.process_command(command);
    }

    server.print_telemetry();
    server.write_telemetry_jsonl(telemetry_path)?;

    println!();
    println!("Wrote telemetry: {telemetry_path}");

    Ok(())
}

fn read_commands_jsonl(path: impl AsRef<Path>) -> io::Result<Vec<ClientCommand>> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    let mut commands = Vec::new();

    for (line_index, line) in reader.lines().enumerate() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let command = serde_json::from_str(&line).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse command line {}: {}", line_index + 1, error),
            )
        })?;

        commands.push(command);
    }

    Ok(commands)
}

fn demo_commands() -> Vec<ClientCommand> {
    let player_id = PlayerId(1);

    vec![
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
    ]
}
