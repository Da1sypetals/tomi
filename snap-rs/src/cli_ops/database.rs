use super::memsnap::MemSnap;
use crate::allocation::Frame;
use rusqlite::Connection;

#[derive(Debug)]
pub struct AllocationDbRow {
    pub index: usize,
    pub size: u64,
    pub callstack: String,
    pub peak_mem: u64,
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
            }
        });

        let database = Connection::open_in_memory()?;

        database.execute("CREATE TABLE allocations (idx INTEGER PRIMARY KEY, size INTEGER, callstack TEXT, peak_mem INTEGER)", ())?;

        for row in rows {
            database.execute(
                "INSERT INTO allocations (idx, size, callstack, peak_mem) VALUES (?, ?, ?, ?)",
                (&row.index, &row.size, &row.callstack, &row.peak_mem),
            )?;
        }

        let mut stmt =
            database.prepare("SELECT idx, size, callstack, peak_mem FROM allocations")?;

        let person_iter = stmt
            .query_map([], |row| {
                Ok(AllocationDbRow {
                    index: row.get(0)?,
                    size: row.get(1)?,
                    callstack: row.get(2)?,
                    peak_mem: row.get(3)?,
                })
            })?
            .collect::<Vec<_>>();

        dbg!(person_iter);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{cli_ops::memsnap::MemSnap, load::load_allocations, utils::format_bytes};

    #[test]
    fn test_peak() {
        // These paths should point to your actual JSON files
        // For demonstration, you'd create dummy files named allocations.json and elements.json
        // in a 'snapshots' directory relative to where you run the executable.
        let alloc_path = "../snapshots/allocations.json";
        let elements_path = "../snapshots/elements.json";

        let allocations = load_allocations(alloc_path, elements_path).unwrap();

        let mut memsnap = MemSnap::new(allocations);

        memsnap.build_sqlite().unwrap();
    }
}
