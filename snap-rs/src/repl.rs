use crate::repl_ops::memsnap::MemSnap;

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
        } else if cmd == "q" || cmd == "quit" {
            ExecResult::Quit
        } else {
            ExecResult::Error("Unsupported...".to_string())
        }
    }
}
