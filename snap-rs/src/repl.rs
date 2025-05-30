use crate::{repl_ops::memsnap::MemSnap, utils::format_bytes};
use thiserror::Error;

// define a quit error
#[derive(Debug, Error)]
pub struct Quit;

impl std::fmt::Display for Quit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!quit")
    }
}

// TODO: refactor Quit to a kind of error and handle specially in repl_main.rs

#[derive(Debug, PartialEq)]
pub enum TopkOption {
    Global,
    Timestamp(u32),
    GlobalVerbose,
    TimestampVerbose(u32),
}

pub fn parse_topk_option(options: &[&str]) -> anyhow::Result<TopkOption> {
    match options.len() {
        0 => Ok(TopkOption::Global),
        1 => {
            let s = options[0];
            if s == "v" || s == "verbose" {
                Ok(TopkOption::GlobalVerbose)
            } else if s.starts_with('@') {
                if let Ok(ts) = s[1..].parse::<u32>() {
                    Ok(TopkOption::Timestamp(ts))
                } else {
                    Err(anyhow::anyhow!("Invalid timestamp format: {}", s))
                }
            } else {
                Err(anyhow::anyhow!("Unrecognized single element: {}", s))
            }
        }
        2 => {
            let s1 = options[0];
            let s2 = options[1];

            let s1_is_verbose = s1 == "v" || s1 == "verbose";
            let s2_is_timestamp = s2.starts_with('@');

            // Case: one verbose, one timestamp
            if s1_is_verbose && s2_is_timestamp {
                if let Ok(ts) = s2[1..].parse::<u32>() {
                    Ok(TopkOption::TimestampVerbose(ts))
                } else {
                    Err(anyhow::anyhow!("Invalid timestamp format: {}", s2))
                }
            } else {
                Err(anyhow::anyhow!(
                    "Unrecognized two elements: {:?} (expected [verbose] [timestamp])",
                    options
                ))
            }
        }
        _ => Err(anyhow::anyhow!(
            "Input slice has an unsupported number of elements: {}",
            options.len()
        )),
    }
}

impl MemSnap {
    /// Input: trimmed command string
    /// Return the output string as an ExecResult
    pub fn exec(&mut self, cmd: String) -> anyhow::Result<String> {
        // 1. Split at whitespace: first part as command, rest as arguments.
        // If the command string is empty after trimming, return success with empty info.
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];
        let args = parts.get(1).map_or("", |s| s.trim()); // Get arguments, trim, or empty string if no args

        if command.is_empty() {
            return Ok("".into());
        }

        // 2. Handle the commands based on the parsed command and arguments.
        match command {
            "sql" => {
                if args.is_empty() {
                    return Err(anyhow::anyhow!("SQL error: query is empty"));
                }
                self.exec_sql(args)
            }
            "sqlbuild" => {
                if !args.is_empty() {
                    return Err(anyhow::anyhow!(
                        "`sqlbuild` command does not take arguments.".to_string(),
                    ));
                }
                self.build_sqlite()?;
                Ok("Build Sqlite OK".into())
            }
            "byte" => match args.parse::<u64>() {
                Ok(bytes) => Ok(format_bytes(bytes).into()),
                Err(e) => Err(anyhow::anyhow!(
                    "Invalid byte value (expected uint64): {}",
                    e
                )),
            },
            "timeline" => {
                if args.is_empty() {
                    return Err(anyhow::anyhow!(
                        "`timeline` command requires a path argument.".to_string(),
                    ));
                }
                self.plot_timeline(args)?;
                Ok(format!("Plot saved to {}", args).into())
            }
            "peak" => {
                // split args by every whitespace
                let argv = args.split_whitespace().collect::<Vec<&str>>();
                // if no index is specified, inspect the last allocation
                if argv.len() != 1 && argv.len() != 2 {
                    return Err(anyhow::anyhow!("`peak` command takes [k] as argument."));
                }

                // parse as usize
                let k = argv[0].parse::<usize>()?;
                match argv.len() {
                    1 => Ok("Index, sorted descending by allocation size: ".to_owned()
                        + &self
                            .peak_topk(k)?
                            .iter()
                            .map(|&i| i.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")),
                    2 => {
                        if argv[1] == "verbose" || argv[1] == "v" {
                            Ok(self
                                .peak_topk(k)?
                                .iter()
                                .enumerate()
                                // rank: ranking sorted by size descending
                                .map(|(rank, &i)| format!("#{}\n{}", rank, self.allocations[i]))
                                .collect::<Vec<_>>()
                                .join("\n\n"))
                        } else {
                            Err(anyhow::anyhow!(
                                "Invalid option: {}, expected `verbose` or `v`",
                                argv[1]
                            ))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            "top" => {
                // split args by every whitespace
                let argv = args.split_whitespace().collect::<Vec<&str>>();
                // if no index is specified, inspect the last allocation
                if argv.len() == 0 || argv.len() > 3 {
                    return Err(anyhow::anyhow!(
                        "`top` command takes [k] and optional [verbose] [@timestamp] as argument."
                    ));
                }
                // try to parse the index as a number
                let k = argv[0].parse::<usize>()?;

                if k >= self.allocations.len() {
                    // return Err(anyhow::anyhow!(format!(
                    //     "Index out of bounds: {} >= {}",
                    //     k,
                    //     self.allocations.len()
                    // ));

                    return Err(anyhow::anyhow!(
                        "Index out of bounds: {} >= {}",
                        k,
                        self.allocations.len()
                    ));
                }

                let options = &argv[1..];
                let topk_options = parse_topk_option(options)?;
                match topk_options {
                    TopkOption::Global => Ok("Index, sorted descending by allocation size: "
                        .to_owned()
                        + &self
                            // NOTE: global topK
                            .global_topk(k)?
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")),
                    TopkOption::Timestamp(timestamp) => {
                        Ok("Index, sorted descending by allocation size: ".to_owned()
                            + &self
                                // NOTE: timestamp topK
                                .timestamp_topk(timestamp, k)?
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(", "))
                    }
                    TopkOption::GlobalVerbose => Ok(self
                        // NOTE: global topK
                        .global_topk(k)?
                        .iter()
                        .enumerate()
                        // rank: ranking sorted by size descending
                        .map(|(rank, &i)| format!("#{}\n{}", rank, self.allocations[i]))
                        .collect::<Vec<_>>()
                        .join("\n\n")),
                    TopkOption::TimestampVerbose(timestamp) => Ok(self
                        // NOTE: timestamp topK
                        .timestamp_topk(timestamp, k)?
                        .iter()
                        .enumerate()
                        // rank: ranking sorted by size descending
                        .map(|(rank, &i)| format!("#{}\n{}", rank, self.allocations[i]))
                        .collect::<Vec<_>>()
                        .join("\n\n")),
                }
            }
            "i" | "inspect" => {
                // split args by every whitespace
                let argv = args.split_whitespace().collect::<Vec<&str>>();
                // if no index is specified, inspect the last allocation
                if argv.len() == 0 {
                    return Err(anyhow::anyhow!(
                        "`inspect` command requires at least an index argument.".to_string(),
                    ));
                }
                // try to parse the index as a number
                let index = match argv[0].parse::<usize>() {
                    Ok(index) => index,
                    Err(e) => {
                        // if the index is not a number, return an error
                        return Err(anyhow::anyhow!(format!("Invalid index value: {}", e)));
                    }
                };

                // check if the index is within the bounds of the allocations
                if index >= self.allocations.len() {
                    return Err(anyhow::anyhow!(
                        "Index out of bounds: {} >= {}",
                        index,
                        self.allocations.len()
                    ));
                }

                let options = &argv[1..];

                if options.is_empty() {
                    // if no options are specified, just print the allocation details
                    Ok(self.allocations[index].to_string())
                } else {
                    // TODO: implement other options
                    // Err(anyhow::anyhow!(format)
                    Err(anyhow::anyhow!(
                        "Unsupported option: [{}]",
                        options.join(" ")
                    ))
                }
            }
            "help" => {
                if !args.is_empty() {
                    return Err(anyhow::anyhow!(
                        "`help` command does not take arguments.".to_string(),
                    ));
                }
                Ok(
                    r#"Available commands:
  help                    - Display this help message.
  i | inspect <index>     - Inspect an allocation at the specified index.
  sql <query>             - Execute an SQL query against the loaded data (require build sql database first).
  sqlbuild                - Build the in-memory sqlite database from current data.
  byte <value>            - Format a byte value (e.g., '1024' -> '1.0 KiB').
  timeline <path>         - Plot a timeline graph and save it to the specified path.
  q | quit                - Exit the application."#
                        .to_string(),
                )
            }
            "q" | "quit" => Err(Quit.into()),
            _ => Err(anyhow::anyhow!(
                "Unsupported command: '{}'. Type 'help' for available commands.",
                cmd
            )),
        }
    }
}
