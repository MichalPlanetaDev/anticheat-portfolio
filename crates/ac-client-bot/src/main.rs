use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use ac_protocol::{ClientCommand, FireCommand, MovementCommand, PlayerId, Vec2};

fn main() {
    if let Err(error) = run() {
        eprintln!("client bot failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        None | Some("all") => {
            write_named_scenario("normal", "samples/normal-commands.jsonl", normal_commands())?;
            write_named_scenario(
                "suspicious",
                "samples/suspicious-commands.jsonl",
                suspicious_commands(),
            )?;
        }
        Some("normal") => {
            write_named_scenario("normal", "samples/normal-commands.jsonl", normal_commands())?;
        }
        Some("suspicious") => {
            write_named_scenario(
                "suspicious",
                "samples/suspicious-commands.jsonl",
                suspicious_commands(),
            )?;
        }
        Some("help") | Some("--help") | Some("-h") => {
            print_help();
        }
        Some(unknown) => {
            eprintln!("unknown scenario: {unknown}");
            eprintln!();
            print_help();
            std::process::exit(2);
        }
    }

    Ok(())
}

fn print_help() {
    println!("Anti-Cheat Client Bot");
    println!();
    println!("Usage:");
    println!("  cargo run -p ac-client-bot -- <scenario>");
    println!();
    println!("Scenarios:");
    println!("  all         Generate normal and suspicious command streams");
    println!("  normal      Generate normal client commands");
    println!("  suspicious  Generate suspicious client commands");
}

fn write_named_scenario(name: &str, path: &str, commands: Vec<ClientCommand>) -> io::Result<()> {
    write_commands_jsonl(path, &commands)?;
    println!(
        "Wrote {name} scenario: {path} ({} commands)",
        commands.len()
    );
    Ok(())
}

fn write_commands_jsonl(path: impl AsRef<Path>, commands: &[ClientCommand]) -> io::Result<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for command in commands {
        serde_json::to_writer(&mut writer, command).map_err(to_invalid_data)?;
        writer.write_all(b"\n")?;
    }

    writer.flush()
}

fn normal_commands() -> Vec<ClientCommand> {
    let player_id = PlayerId(1);

    vec![
        move_command(
            player_id,
            1,
            100,
            Vec2::new(1.0, 0.0),
            Some(Vec2::new(1.0, 0.0)),
        ),
        move_command(
            player_id,
            2,
            200,
            Vec2::new(1.0, 0.0),
            Some(Vec2::new(2.0, 0.0)),
        ),
        fire_command(player_id, 3, 300),
        move_command(
            player_id,
            4,
            400,
            Vec2::new(1.0, 0.0),
            Some(Vec2::new(3.0, 0.0)),
        ),
        move_command(
            player_id,
            5,
            500,
            Vec2::new(1.0, 0.0),
            Some(Vec2::new(4.0, 0.0)),
        ),
        move_command(
            player_id,
            6,
            600,
            Vec2::new(1.0, 0.0),
            Some(Vec2::new(5.0, 0.0)),
        ),
        move_command(
            player_id,
            7,
            700,
            Vec2::new(1.0, 0.0),
            Some(Vec2::new(6.0, 0.0)),
        ),
        fire_command(player_id, 8, 800),
    ]
}

fn suspicious_commands() -> Vec<ClientCommand> {
    let player_id = PlayerId(1);

    vec![
        move_command(
            player_id,
            1,
            100,
            Vec2::new(1.0, 0.0),
            Some(Vec2::new(1.0, 0.0)),
        ),
        move_command(
            player_id,
            2,
            200,
            Vec2::new(1.0, 0.0),
            Some(Vec2::new(20.0, 0.0)),
        ),
        fire_command(player_id, 3, 300),
        fire_command(player_id, 4, 400),
        move_command(
            player_id,
            4,
            500,
            Vec2::new(1.0, 0.0),
            Some(Vec2::new(21.0, 0.0)),
        ),
    ]
}

fn move_command(
    player_id: PlayerId,
    sequence: u64,
    client_time_ms: u64,
    direction: Vec2,
    claimed_position: Option<Vec2>,
) -> ClientCommand {
    ClientCommand::Move(MovementCommand {
        player_id,
        sequence,
        client_time_ms,
        direction,
        dt_ms: 100,
        claimed_position,
    })
}

fn fire_command(player_id: PlayerId, sequence: u64, client_time_ms: u64) -> ClientCommand {
    ClientCommand::Fire(FireCommand {
        player_id,
        sequence,
        client_time_ms,
        aim_direction: Vec2::new(1.0, 0.0),
    })
}

fn to_invalid_data(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
