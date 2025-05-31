use super::memsnap::MemSnap;
use crate::allocation::Frame;
use rusqlite::Connection;
use std::fmt::Write;

#[derive(Debug)]
pub struct AllocationDbRow {
    pub index: usize,
    pub size: u64,
    pub callstack: String,
    pub peak_mem: u64,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
}

pub fn format_callstack(frames: &[Frame]) -> String {
    frames
        .iter()
        .map(|frame| format!("{}:{}:{}", frame.filename, frame.line, frame.name))
        .collect::<Vec<String>>()
        .join("\n")
}

impl MemSnap {
    /// 可能被sql用到的字段：
    /// id: index int
    /// size: int
    /// callstack: varchar
    /// peak_mem: int
    pub fn build_sqlite(&mut self) -> Result<(), anyhow::Error> {
        let rows = self.allocations.iter().enumerate().map(|(index, alloc)| {
            let callstack = format_callstack(&alloc.callstack);

            AllocationDbRow {
                index,
                size: alloc.size,
                callstack,
                peak_mem: alloc.peak_mem,
                start_timestamp: alloc.timesteps[0],
                end_timestamp: *alloc.timesteps.last().unwrap(),
            }
        });

        let database = Connection::open_in_memory()?;

        {
            log::info!("Creating allocations table");
            database.execute(
                "CREATE TABLE allocations (
idx INTEGER PRIMARY KEY, 
size INTEGER, 
callstack TEXT, 
peak_mem INTEGER,
start_timestamp INTEGER,
end_timestamp INTEGER
)",
                (),
            )?;

            log::info!("Inserting rows into allocations table");
            for row in rows {
                database.execute(
                    "INSERT INTO allocations (idx, size, callstack, peak_mem, start_timestamp, end_timestamp) VALUES (?, ?, ?, ?, ?, ?)",
                    (
                        &row.index,
                        &row.size,
                        &row.callstack,
                        &row.peak_mem,
                        &row.start_timestamp,
                        &row.end_timestamp,
                    ),
                )?;
            }
        }

        self.database = Some(database);

        Ok(())
    }

    pub fn exec_sql(&mut self, sql: &str) -> Result<String, anyhow::Error> {
        match &self.database {
            Some(database) => {
                log::info!("Executing SQL query");

                let mut stmt = database.prepare(sql)?;
                let num_cols = stmt.column_count();
                let column_names: Vec<String> =
                    stmt.column_names().iter().map(|s| s.to_string()).collect();

                let mut output_string = String::new();
                let rows_iter = stmt.query_map([], |row| {
                    let mut row_values = Vec::new();
                    for i in 0..num_cols {
                        let value_str = row
                            .get_ref(i)?
                            .as_str()
                            .map(|s| s.to_string()) // if is text
                            .unwrap_or_else(|_| {
                                // If not text, try to represent it as a string
                                match row.get_ref(i) {
                                    Ok(rusqlite::types::ValueRef::Integer(i)) => i.to_string(),
                                    Ok(rusqlite::types::ValueRef::Real(f)) => f.to_string(),
                                    Ok(rusqlite::types::ValueRef::Blob(b)) => {
                                        format!("<BLOB len={}>", b.len())
                                    }
                                    Ok(rusqlite::types::ValueRef::Null) => String::from("NULL"),
                                    _ => String::from("[UNSUPPORTED TYPE]"),
                                }
                            });
                        row_values.push(value_str);
                    }
                    Ok(row_values)
                })?;

                log::info!("Merging results");
                output_string.push_str("\n===== SQL Query Results =====\n");
                for (idx, row_result) in rows_iter.enumerate() {
                    let row_values = row_result?;
                    output_string.push_str(&format!("\n===== Row {} =====\n", idx));
                    for (col_name, row_value) in column_names.iter().zip(row_values) {
                        output_string.push_str(&format!("column [{}] : {}\n", col_name, row_value));
                    }
                }

                Ok(output_string)
            }
            None => Err(anyhow::anyhow!("Please build sqlite database first!")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        load::{load_allocations, read_snap_from_jsons},
        repl_ops::memsnap::MemSnap,
        utils::format_bytes,
    };

    #[test]
    fn test_peak() {
        // These paths should point to your actual JSON files
        // For demonstration, you'd create dummy files named allocations.json and elements.json
        // in a 'snapshots' directory relative to where you run the executable.
        let alloc_path = "../snapshots/allocations.json";
        let elements_path = "../snapshots/elements.json";

        let allocations =
            load_allocations(read_snap_from_jsons(alloc_path, elements_path).unwrap()).unwrap();

        let mut memsnap = MemSnap::new(allocations);

        memsnap.build_sqlite().unwrap();

        match memsnap.exec_sql("SELECT idx, size FROM allocations ORDER BY size DESC LIMIT 10") {
            Ok(out) => println!("{}", out),
            Err(e) => eprintln!("SQL error: {}", e),
        }

        match memsnap.exec_sql("SELECT SUM(size) FROM allocations ORDER BY size LIMIT 4") {
            Ok(out) => println!("{}", out),
            Err(e) => eprintln!("SQL error: {}", e),
        }

        match memsnap.exec_sql("SELECT SUM(aaa) FROM allocations ORDER BY size LIMIT 4") {
            Ok(out) => println!("{}", out),
            Err(e) => eprintln!("SQL error: {}", e),
        }
    }

    #[test]
    fn test_big() {
        // These paths should point to your actual JSON files
        // For demonstration, you'd create dummy files named allocations.json and elements.json
        // in a 'snapshots' directory relative to where you run the executable.
        let alloc_path = "../snapshots/large/transformer_allocations.json";
        let elements_path = "../snapshots/large/transformer_elements.json";

        // start timer

        let mut memsnap = MemSnap::from_jsons(alloc_path, elements_path).unwrap();

        memsnap.build_sqlite().unwrap();

        match memsnap.exec_sql("SELECT idx, size FROM allocations ORDER BY size DESC LIMIT 10") {
            Ok(out) => println!("{}", out),
            Err(e) => eprintln!("SQL error: {}", e),
        }

        match memsnap.exec_sql(
            "SELECT idx AS num_rows
FROM allocations
ORDER BY size DESC
LIMIT 120;",
        ) {
            Ok(out) => println!("{}", out),
            Err(e) => eprintln!("SQL error: {}", e),
        }

        match memsnap.exec_sql("SELECT SUM(aaa) FROM allocations ORDER BY size LIMIT 4") {
            Ok(out) => println!("{}", out),
            Err(e) => eprintln!("SQL error: {}", e),
        }

        match memsnap.exec_sql(
            "SELECT SUM(size) AS total_size, COUNT(*) AS num_rows
FROM allocations
WHERE callstack LIKE '%torch%';",
        ) {
            Ok(out) => println!("{}", out),
            Err(e) => eprintln!("SQL error: {}", e),
        }
    }
}
