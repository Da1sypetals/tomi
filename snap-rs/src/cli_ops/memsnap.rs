use crate::snapshot::Allocation;
use std::collections::BTreeMap;

pub type AllocationIndex = usize;

/// Options are lazily created
pub struct MemSnap {
    pub allocations: Vec<Allocation>,

    pub timestamps: Vec<u32>, // all timestamps that something happens, sorted ascending

    pub global_sorted_sizes: Option<Vec<AllocationIndex>>, // indices, sorted descending

    pub timestamp_sorted_sizes: BTreeMap<u32, Vec<AllocationIndex>>, // timestamp -> indices, sorted descending

    pub peak_sorted_sizes: Option<Vec<AllocationIndex>>,
    // database: SqLite database optional
}

impl MemSnap {
    pub fn new(allocations: Vec<Allocation>) -> Self {
        let mut timestamps: Vec<u32> = Vec::new();

        for alloc in &allocations {
            timestamps.extend(alloc.timesteps.iter());
        }

        timestamps.sort();
        timestamps.dedup();

        // dbg!(&timestamps);

        MemSnap {
            allocations,
            timestamps: timestamps,
            global_sorted_sizes: None,
            timestamp_sorted_sizes: BTreeMap::new(),
            peak_sorted_sizes: None,
        }
    }
}
