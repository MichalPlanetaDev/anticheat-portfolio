use ac_detectors::{Detector, FireRateDetector, SpeedHackDetector};
use ac_protocol::{FireSample, MovementSample, PlayerId, SuspicionReport, Vec2};
use ac_replay::{
    export_suspicion_reports_csv, risk_summaries_from_file, summarize_file,
    suspicion_reports_from_file,
};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        None | Some("help") | Some("--help") | Some("-h") => print_help(),
        Some("speed-check") => run_speed_check(),
        Some("fire-check") => run_fire_check(),
        Some("demo-report") => run_demo_report(),
        Some("replay") => {
            let Some(path) = args.get(1) else {
                eprintln!("Missing telemetry file path.");
                eprintln!();
                eprintln!("Usage:");
                eprintln!("  cargo run -p ac-cli -- replay samples/server-session.jsonl");
                std::process::exit(2);
            };

            run_replay(path);
        }
        Some("inspect") => {
            let Some(path) = args.get(1) else {
                eprintln!("Missing telemetry file path.");
                eprintln!();
                eprintln!("Usage:");
                eprintln!("  cargo run -p ac-cli -- inspect samples/suspicious-telemetry.jsonl");
                std::process::exit(2);
            };

            run_inspect(path);
        }

        Some("risk") => {
            let Some(path) = args.get(1) else {
                eprintln!("Missing telemetry file path.");
                eprintln!();
                eprintln!("Usage:");
                eprintln!("  cargo run -p ac-cli -- risk samples/suspicious-telemetry.jsonl");
                std::process::exit(2);
            };

            run_risk(path);
        }

        Some("export") => {
            let Some(input_path) = args.get(1) else {
                eprintln!("Missing input telemetry file path.");
                eprintln!();
                eprintln!("Usage:");
                eprintln!(
                    "  cargo run -p ac-cli -- export samples/suspicious-telemetry.jsonl reports/suspicious-report.csv"
                );
                std::process::exit(2);
            };

            let Some(output_path) = args.get(2) else {
                eprintln!("Missing output CSV file path.");
                eprintln!();
                eprintln!("Usage:");
                eprintln!(
                    "  cargo run -p ac-cli -- export samples/suspicious-telemetry.jsonl reports/suspicious-report.csv"
                );
                std::process::exit(2);
            };

            run_export(input_path, output_path);
        }
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
    println!("  help                 Show this help message");
    println!("  speed-check          Run movement speed anomaly examples");
    println!("  fire-check           Run weapon cooldown anomaly examples");
    println!("  demo-report          Run all demo detections and print a summary");
    println!("  replay <jsonl-path>  Summarize saved telemetry from a JSONL file");
    println!("  inspect <jsonl-path> Print detailed suspicion reports from telemetry");
    println!("  export <jsonl-path> <csv-path> Export suspicion reports to CSV");
    println!("  risk <jsonl-path>    Show player risk summaries from telemetry");
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

        println!("Allowed distance: {allowed_distance:.3}");

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

fn run_replay(path: &str) {
    match summarize_file(path) {
        Ok(summary) => {
            println!("Telemetry replay summary");
            println!();
            println!("File: {path}");
            println!("Total events: {}", summary.total_events);
            println!("Accepted commands: {}", summary.accepted_commands);
            println!("Player snapshots: {}", summary.player_snapshots);
            println!("Suspicion reports: {}", summary.suspicion_reports);
            println!();

            if summary.suspicion_by_kind.is_empty() {
                println!("No suspicious behavior detected.");
                return;
            }

            println!("Suspicion breakdown:");

            for (kind, count) in summary.suspicion_by_kind {
                println!("  {kind}: {count}");
            }
        }
        Err(error) => {
            eprintln!("Failed to replay telemetry file '{path}': {error}");
            std::process::exit(1);
        }
    }
}

fn run_inspect(path: &str) {
    match suspicion_reports_from_file(path) {
        Ok(reports) => {
            println!("Telemetry suspicion inspection");
            println!();
            println!("File: {path}");
            println!("Suspicion reports: {}", reports.len());
            println!();

            if reports.is_empty() {
                println!("No suspicious behavior detected.");
                return;
            }

            for (index, report) in reports.iter().enumerate() {
                println!("Report #{}", index + 1);
                println!("  Player: {:?}", report.player_id);
                println!("  Sequence: {}", report.sequence);
                println!("  Kind: {:?}", report.kind);
                println!("  Reason: {}", report.reason);
                println!("  Observed value: {:.3}", report.observed_value);
                println!("  Expected limit: {:.3}", report.expected_limit);
                println!();
            }
        }
        Err(error) => {
            eprintln!("Failed to inspect telemetry file '{path}': {error}");
            std::process::exit(1);
        }
    }
}

fn run_risk(path: &str) {
    match risk_summaries_from_file(path) {
        Ok(summaries) => {
            println!("Player risk summary");
            println!();
            println!("File: {path}");
            println!("Players with suspicion reports: {}", summaries.len());
            println!();

            if summaries.is_empty() {
                println!("No suspicious behavior detected.");
                return;
            }

            for summary in summaries {
                println!("Player: {:?}", summary.player_id);
                println!("Reports: {}", summary.reports);
                println!("Risk score: {}", summary.risk_score);
                println!("Breakdown:");

                for (kind, count) in summary.suspicion_by_kind {
                    println!("  {kind}: {count}");
                }

                println!();
            }
        }
        Err(error) => {
            eprintln!("Failed to build player risk summary: {error}");
            std::process::exit(1);
        }
    }
}

fn run_export(input_path: &str, output_path: &str) {
    match export_suspicion_reports_csv(input_path, output_path) {
        Ok(report_count) => {
            println!("Exported suspicion reports");
            println!();
            println!("Input: {input_path}");
            println!("Output: {output_path}");
            println!("Reports exported: {report_count}");
        }
        Err(error) => {
            eprintln!("Failed to export suspicion reports: {error}");
            std::process::exit(1);
        }
    }
}

fn print_report(report: &SuspicionReport) {
    println!("Result: FLAGGED");
    println!("Kind: {:?}", report.kind);
    println!("Reason: {}", report.reason);
    println!("Observed value: {:.3}", report.observed_value);
    println!("Expected limit: {:.3}", report.expected_limit);
}
