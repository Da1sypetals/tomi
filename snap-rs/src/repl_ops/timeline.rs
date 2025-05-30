use super::memsnap::MemSnap;
use std::collections::BTreeMap;

impl MemSnap {
    /// Returns (timestamp -> memory occupied) mapping as a Vec
    pub fn timeline(&mut self) -> Vec<(u32, u64)> {
        let mut timeline = BTreeMap::new();

        for alloc in &self.allocations {
            for (&timestamp, &offset) in alloc.timesteps.iter().zip(alloc.offsets.iter()) {
                let mem = offset + alloc.size;
                // if timeline.get(timestamp) 1) has no value, or 2) has value smaller than mem, set to mem
                timeline
                    .entry(timestamp)
                    // if has value, AND is applied, take max
                    .and_modify(|current_mem| *current_mem = mem.max(*current_mem))
                    // if has NO value, OR is applied, insert
                    .or_insert(mem);
            }
        }

        timeline.into_iter().collect()
    }
}
