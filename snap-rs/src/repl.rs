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
    /// Input: trimmed command
    /// Return the output string
    pub fn exec(&mut self, cmd: String) -> ExecResult {
        // TODO: refact or
        if cmd.starts_with("sql ") {
            let sql_command = &cmd[3..];
            match self.exec_sql(&sql_command) {
                Ok(out) => out.into(),
                Err(e) => ExecResult::Error(format!("SQL error: {}", e).into()),
            }
        } else if cmd == "sqlbuild" {
            match self.build_sqlite() {
                Ok(_) => "Build Sqlite OK".to_string().into(),
                Err(e) => ExecResult::Error(format!("Building Sqlite error: {}", e)),
            }
        } else if cmd.starts_with("byte ") {
            let byte_str = cmd[5..].trim(); // Get string after "byte " and trim whitespace
            match byte_str.parse::<u64>() {
                Ok(bytes) => format_bytes(bytes).into(),
                Err(e) => ExecResult::Error(format!("Invalid byte value (expected uint64): {}", e)),
            }
        } else if cmd == "q" || cmd == "quit" {
            ExecResult::Quit
        } else {
            ExecResult::Error("Unsupported...".to_string())
        }
    }
}
