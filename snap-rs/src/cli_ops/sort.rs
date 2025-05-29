use super::memsnap::MemSnap;

impl MemSnap {
    pub fn global_topk(&mut self, k: usize) -> Result<Vec<usize>, anyhow::Error> {
        if k >= self.allocations.len() {
            return Err(anyhow::anyhow!(format!(
                "k is out of bounds: expected 0 ~ {}, got {}",
                self.allocations.len(),
                k
            )));
        }

        match &self.global_sorted_sizes {
            Some(indices_sorted_by_size) => Ok(indices_sorted_by_size[..k].to_vec()),
            None => {
                // create topk vector
                let mut sizes = self
                    .allocations
                    .iter()
                    .enumerate()
                    .map(|(i, alloc)| (i, alloc.size))
                    .collect::<Vec<(usize, u64)>>();

                // NOTE: sort descending
                sizes.sort_by(|(i1, size1), (i2, size2)| size2.cmp(size1));

                let indices_sorted_by_size: Vec<usize> =
                    sizes.into_iter().map(|(index, size)| index).collect();

                self.global_sorted_sizes = Some(indices_sorted_by_size.clone());

                Ok(indices_sorted_by_size[..k].to_vec())
            }
        }
    }

    pub fn timestamp_topk(
        &mut self,
        timestamp: u32,
        k: usize,
    ) -> Result<Vec<usize>, anyhow::Error> {
        if k >= self.allocations.len() {
            return Err(anyhow::anyhow!(format!(
                "k is out of bounds: expected 0 ~ {}, got {}",
                self.allocations.len(),
                k
            )));
        }

        let nearest_timestamp_index = match self.timestamps.binary_search(&timestamp) {
            Ok(i) => i,
            Err(i) => i,
        };
        let nearest_timestamp = self.timestamps[nearest_timestamp_index];

        match self.timestamp_sorted_sizes.get(&nearest_timestamp) {
            Some(indices_sorted_by_size) => {
                println!("Hit {}", nearest_timestamp);
                Ok(indices_sorted_by_size[..k].to_vec())
            }
            None => {
                // create topk vector
                let mut sizes = self
                    .allocations
                    .iter()
                    .enumerate() // first enumerate, make sure index does not change
                    .filter_map(|(i, alloc)| {
                        if alloc.is_alive_at(nearest_timestamp) {
                            Some((i, alloc.size))
                        } else {
                            None
                        }
                    }) // check if this alloc is alive at timestamp
                    .collect::<Vec<(usize, u64)>>();

                // NOTE: sort descending
                sizes.sort_by(|(i1, size1), (i2, size2)| size2.cmp(size1));

                let indices_sorted_by_size: Vec<usize> =
                    sizes.into_iter().map(|(index, size)| index).collect();

                self.timestamp_sorted_sizes
                    .insert(nearest_timestamp, indices_sorted_by_size.clone());

                Ok(indices_sorted_by_size[..k].to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{cli_ops::memsnap::MemSnap, load::load_allocations, utils::format_bytes};

    #[test]
    fn test_global() {
        // These paths should point to your actual JSON files
        // For demonstration, you'd create dummy files named allocations.json and elements.json
        // in a 'snapshots' directory relative to where you run the executable.
        let alloc_path = "../snapshots/allocations.json";
        let elements_path = "../snapshots/elements.json";

        let allocations = load_allocations(alloc_path, elements_path).unwrap();

        let mut memsnap = MemSnap::new(allocations);

        let top5 = memsnap.global_topk(5).unwrap();

        dbg!(&top5);
        for i in top5 {
            println!(
                "Allocation at index {}: {}",
                i,
                format_bytes(memsnap.allocations[i].size)
            );
        }
    }

    #[test]
    fn test_timestamp() {
        // These paths should point to your actual JSON files
        // For demonstration, you'd create dummy files named allocations.json and elements.json
        // in a 'snapshots' directory relative to where you run the executable.
        let alloc_path = "../snapshots/allocations.json";
        let elements_path = "../snapshots/elements.json";

        let allocations = load_allocations(alloc_path, elements_path).unwrap();

        let mut memsnap = MemSnap::new(allocations);

        let top3 = memsnap.timestamp_topk(24, 3).unwrap();
        let top3 = memsnap.timestamp_topk(25, 3).unwrap(); // hit 26
        let top3 = memsnap.timestamp_topk(24, 3).unwrap(); // hit 26
        let top3 = memsnap.timestamp_topk(25, 3).unwrap(); // hit 26

        dbg!(&top3);
        for i in top3 {
            println!(
                "Allocation at index {}: {}",
                i,
                format_bytes(memsnap.allocations[i].size)
            );
        }
    }
}
