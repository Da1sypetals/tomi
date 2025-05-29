use crate::snapshot::Allocation;
use std::collections::{BTreeMap, BTreeSet};

pub type AllocationIndex = usize;

/// Options are lazily created
pub struct MemSnap {
    trace: Vec<Allocation>,

    timestamps: BTreeSet<usize>, // all timestamps that something happens

    global_topk: Option<Vec<AllocationIndex>>,

    timestamp_topk: Option<BTreeMap<u32, AllocationIndex>>, // timestamp -> index
}
