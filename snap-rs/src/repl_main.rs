use std::process::exit;

use clap::{Arg, ArgAction, ArgGroup, Command};
use rustyline::{DefaultEditor, error::ReadlineError};
use snap_rs::repl_ops::memsnap::MemSnap;

enum CliArg {
    Json { alloc: String, elem: String },
    Zip { path: String },
}

fn cli() -> CliArg {
    let matches = Command::new("tomi: pyTOrch Memory Inspection tool")
        .arg(
            Arg::new("zip")
                .short('z')
                .long("zip")
                .help("Load snap from a .zip file")
                .action(ArgAction::Set)
                .num_args(1) // Exactly one path
                .value_name("ZIP_PATH")
                .conflicts_with("json"), // Cannot be used with --json
        )
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .help("Load snap from allocations.json and elements.json files")
                .action(ArgAction::Set)
                .num_args(2) // Exactly two paths
                .value_name("JSON_PATHS")
                .conflicts_with("zip"), // Cannot be used with --zip
        )
        // You could also use an ArgGroup for mutual exclusivity, but conflicts_with is more direct here.
        // If you had more complex "either/or" scenarios, ArgGroup would be powerful.
        .get_matches();

    if let Some(zip_paths) = matches.get_many::<String>("zip") {
        let path: Vec<_> = zip_paths.map(|s| s.as_str()).collect();
        CliArg::Zip {
            path: path[0].to_string(),
        }
    } else if let Some(json_paths) = matches.get_many::<String>("json") {
        let paths: Vec<_> = json_paths.map(|s| s.as_str()).collect();

        CliArg::Json {
            alloc: paths[0].to_string(),
            elem: paths[1].to_string(),
        }
    } else {
        eprintln!(
            "No valid arguments provided. Use --zip <PATH> or --json <ALLOC_PATH> <ELEM_PATH>."
        );

        exit(1);
    }
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Off)
        .filter_module("snap_rs", log::LevelFilter::Info)
        .init();

    let snap_opt = match cli() {
        CliArg::Json { alloc, elem } => MemSnap::from_jsons(&alloc, &elem),
        CliArg::Zip { path } => MemSnap::from_zip(&path),
    };

    let mut snap = match snap_opt {
        Ok(snap) => snap,
        Err(err) => {
            eprintln!("Error loading snap: {}", err);
            exit(1);
        }
    };

    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline("tomi> ");
        match readline {
            Ok(line) => {
                let cmd = line.trim().to_string();

                rl.add_history_entry(cmd.as_str())?;

                let output = snap.exec(cmd);

                match output {
                    Ok(out) => println!("{}", out),
                    Err(e) => match e.to_string().as_str() {
                        "!quit" => {
                            println!("Bye!");
                            break;
                        }
                        err => {
                            eprintln!("Error: {}", err);
                        }
                    },
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C starts a new line
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Bye!");
                break;
            }
            Err(err) => {
                eprintln!("Unhandled error encountered: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}
