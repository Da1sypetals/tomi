use log::{info, trace};
use rusqlite::Connection;

use crate::{allocation::Allocation, load::load_allocations};
use std::collections::BTreeMap;

pub type AllocationIndex = usize;

/// Options are lazily created
pub struct MemSnap {
    pub allocations: Vec<Allocation>,

    pub timestamps: Vec<u32>, // all timestamps that something happens, sorted ascending

    pub global_sorted_sizes: Option<Vec<AllocationIndex>>, // indices, sorted descending

    pub timestamp_sorted_sizes: BTreeMap<u32, Vec<AllocationIndex>>, // timestamp -> indices, sorted descending

    pub peak_sorted_sizes: Option<Vec<AllocationIndex>>,

    pub database: Option<Connection>, // database connection to sqlite
}

impl MemSnap {
    pub fn new(allocations: Vec<Allocation>) -> Self {
        info!("Sorting timestamps...");
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
            database: None,
        }
    }

    pub fn from_paths(alloc_path: &str, elements_path: &str) -> Result<Self, anyhow::Error> {
        pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Trace)
            .init();

        info!("Loading allocations...");
        let allocations = load_allocations(alloc_path, elements_path)?;

        Ok(Self::new(allocations))
    }
}
