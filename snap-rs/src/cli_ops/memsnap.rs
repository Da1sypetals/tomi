use crate::snapshot::Allocation;
use std::collections::{BTreeMap, BTreeSet};

pub type AllocationIndex = usize;

/// Options are lazily created
pub struct MemSnap {
    allocations: Vec<Allocation>,

    timestamps: BTreeSet<u32>, // all timestamps that something happens

    global_topk: Option<Vec<AllocationIndex>>,

    timestamp_topk: Option<BTreeMap<u32, AllocationIndex>>, // timestamp -> index

                                                            // database: SqLite database
}

impl MemSnap {
    pub fn new(allocations: Vec<Allocation>) -> Self {
        let mut timestamps: BTreeSet<u32> = BTreeSet::new();

        for alloc in &allocations {
            timestamps.extend(alloc.timesteps.iter());
        }

        MemSnap {
            allocations,
            timestamps: timestamps,
            global_topk: None,
            timestamp_topk: None,
        }
    }
}
