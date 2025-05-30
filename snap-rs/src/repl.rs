use crate::{repl_ops::memsnap::MemSnap, utils::format_bytes};

pub enum ExecResult {
    Success(String),
    Error(String),
    Quit,
}

impl Into<ExecResult> for String {
    fn into(self) -> ExecResult {
        ExecResult::Success(self)
    }
}

impl MemSnap {
    /// Input: trimmed command string
    /// Return the output string as an ExecResult
    pub fn exec(&mut self, cmd: String) -> ExecResult {
        // 1. Split at whitespace: first part as command, rest as arguments.
        // If the command string is empty after trimming, return success with empty info.
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];
        let args = parts.get(1).map_or("", |s| s.trim()); // Get arguments, trim, or empty string if no args

        if command.is_empty() {
            return ExecResult::Success("".to_string());
        }

        // 2. Handle the commands based on the parsed command and arguments.
        match command {
            "sql" => {
                if args.is_empty() {
                    return ExecResult::Error("SQL error: query is empty".to_string());
                }
                match self.exec_sql(args) {
                    Ok(out) => out.into(),
                    Err(e) => ExecResult::Error(format!("SQL error: {}", e)),
                }
            }
            "sqlbuild" => {
                if !args.is_empty() {
                    return ExecResult::Error(
                        "`sqlbuild` command does not take arguments.".to_string(),
                    );
                }
                match self.build_sqlite() {
                    Ok(_) => "Build Sqlite OK".to_string().into(),
                    Err(e) => ExecResult::Error(format!("Building Sqlite error: {}", e)),
                }
            }
            "byte" => match args.parse::<u64>() {
                Ok(bytes) => format_bytes(bytes).into(),
                Err(e) => ExecResult::Error(format!("Invalid byte value (expected uint64): {}", e)),
            },
            "timeline" => {
                if args.is_empty() {
                    return ExecResult::Error(
                        "`timeline` command requires a path argument.".to_string(),
                    );
                }
                match self.plot_timeline(args) {
                    Ok(_) => format!("Plot saved to {}", args).into(),
                    Err(e) => ExecResult::Error(format!("Plotting error: {}", e)),
                }
            }
            "i" | "inspect" => {
                // split args by every whitespace
                let argv = args.split_whitespace().collect::<Vec<&str>>();
                // if no index is specified, inspect the last allocation
                if argv.len() == 0 {
                    return ExecResult::Error(
                        "`inspect` command requires at least an index argument.".to_string(),
                    );
                }
                // try to parse the index as a number
                let index = match argv[0].parse::<usize>() {
                    Ok(index) => index,
                    Err(e) => {
                        // if the index is not a number, return an error
                        return ExecResult::Error(format!("Invalid index value: {}", e));
                    }
                };

                // check if the index is within the bounds of the allocations
                if index >= self.allocations.len() {
                    return ExecResult::Error(format!(
                        "Index out of bounds: {} >= {}",
                        index,
                        self.allocations.len()
                    ));
                }

                let options = &argv[1..];

                if options.is_empty() {
                    // if no options are specified, just print the allocation details
                    self.allocations[index].to_string().into()
                } else {
                    // TODO: implement other options
                    ExecResult::Error(format!("Unsupported option: [{}]", options.join(" ")))
                }
            }
            "help" => {
                if !args.is_empty() {
                    return ExecResult::Error(
                        "`help` command does not take arguments.".to_string(),
                    );
                }
                ExecResult::Success(
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
            "q" | "quit" => ExecResult::Quit,
            _ => ExecResult::Error(format!(
                "Unsupported command: '{}'. Type 'help' for available commands.",
                command
            )),
        }
    }
}
