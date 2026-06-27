use ac_detectors::{Detector, FireRateDetector, SpeedHackDetector};
use ac_protocol::{FireSample, MovementSample, PlayerId, SuspicionReport, Vec2};

fn main() {
    let command = std::env::args().nth(1);

    match command.as_deref() {
        None | Some("help") | Some("--help") | Some("-h") => print_help(),
        Some("speed-check") => run_speed_check(),
        Some("fire-check") => run_fire_check(),
        Some("demo-report") => run_demo_report(),
        Some(unknown) => {
            eprintln!("Unknown command: {unknown}");
            eprintln!();
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_help() {
    println!("Anti-Cheat Engineering Lab CLI");
    println!();
    println!("Usage:");
    println!("  cargo run -p ac-cli -- <command>");
    println!();
    println!("Commands:");
    println!("  help          Show this help message");
    println!("  speed-check   Run movement speed anomaly examples");
    println!("  fire-check    Run weapon cooldown anomaly examples");
    println!("  demo-report   Run all demo detections and print a summary");
}

fn run_speed_check() {
    let detector = SpeedHackDetector;

    let samples = [
        (
            "normal movement",
            MovementSample {
                player_id: PlayerId(1),
                sequence: 100,
                from: Vec2::new(0.0, 0.0),
                to: Vec2::new(0.9, 0.0),
                dt_ms: 100,
                max_speed_units_per_second: 10.0,
                tolerance_units: 0.15,
            },
        ),
        (
            "suspicious movement",
            MovementSample {
                player_id: PlayerId(1),
                sequence: 101,
                from: Vec2::new(0.0, 0.0),
                to: Vec2::new(5.0, 0.0),
                dt_ms: 100,
                max_speed_units_per_second: 10.0,
                tolerance_units: 0.15,
            },
        ),
    ];

    println!("Movement speed analysis");
    println!();

    for (label, sample) in samples {
        println!("Case: {label}");
        println!("Player: {:?}", sample.player_id);
        println!("Sequence: {}", sample.sequence);
        println!("Observed distance: {:.3}", sample.distance());

        let allowed_distance =
            sample.max_speed_units_per_second * sample.elapsed_seconds() + sample.tolerance_units;

        println!("Allowed distance: {:.3}", allowed_distance);

        match detector.inspect(&sample) {
            Some(report) => print_report(&report),
            None => println!("Result: OK"),
        }

        println!();
    }
}

fn run_fire_check() {
    let detector = FireRateDetector;

    let samples = [
        (
            "ready weapon",
            FireSample {
                player_id: PlayerId(2),
                sequence: 200,
                server_time_ms: 1500,
                next_allowed_fire_time_ms: 1200,
            },
        ),
        (
            "early fire request",
            FireSample {
                player_id: PlayerId(2),
                sequence: 201,
                server_time_ms: 900,
                next_allowed_fire_time_ms: 1200,
            },
        ),
    ];

    println!("Weapon cooldown analysis");
    println!();

    for (label, sample) in samples {
        println!("Case: {label}");
        println!("Player: {:?}", sample.player_id);
        println!("Sequence: {}", sample.sequence);
        println!("Server time: {} ms", sample.server_time_ms);
        println!(
            "Next allowed fire time: {} ms",
            sample.next_allowed_fire_time_ms
        );

        match detector.inspect(&sample) {
            Some(report) => print_report(&report),
            None => println!("Result: OK"),
        }

        println!();
    }
}

fn run_demo_report() {
    let speed_detector = SpeedHackDetector;
    let fire_detector = FireRateDetector;

    let movement_samples = [
        MovementSample {
            player_id: PlayerId(1),
            sequence: 300,
            from: Vec2::new(0.0, 0.0),
            to: Vec2::new(0.8, 0.0),
            dt_ms: 100,
            max_speed_units_per_second: 10.0,
            tolerance_units: 0.15,
        },
        MovementSample {
            player_id: PlayerId(1),
            sequence: 301,
            from: Vec2::new(0.0, 0.0),
            to: Vec2::new(6.0, 0.0),
            dt_ms: 100,
            max_speed_units_per_second: 10.0,
            tolerance_units: 0.15,
        },
    ];

    let fire_samples = [
        FireSample {
            player_id: PlayerId(2),
            sequence: 400,
            server_time_ms: 2000,
            next_allowed_fire_time_ms: 1800,
        },
        FireSample {
            player_id: PlayerId(2),
            sequence: 401,
            server_time_ms: 2100,
            next_allowed_fire_time_ms: 2500,
        },
    ];

    let mut reports = Vec::new();

    for sample in movement_samples {
        if let Some(report) = speed_detector.inspect(&sample) {
            reports.push(report);
        }
    }

    for sample in fire_samples {
        if let Some(report) = fire_detector.inspect(&sample) {
            reports.push(report);
        }
    }

    println!("Demo investigation report");
    println!();
    println!("Suspicion reports: {}", reports.len());
    println!();

    if reports.is_empty() {
        println!("No suspicious behavior detected.");
        return;
    }

    for report in reports {
        print_report(&report);
        println!();
    }
}

fn print_report(report: &SuspicionReport) {
    println!("Result: FLAGGED");
    println!("Kind: {:?}", report.kind);
    println!("Reason: {}", report.reason);
    println!("Observed value: {:.3}", report.observed_value);
    println!("Expected limit: {:.3}", report.expected_limit);
}
