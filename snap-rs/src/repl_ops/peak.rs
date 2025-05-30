use super::memsnap::MemSnap;

impl MemSnap {
    pub fn peak_topk(&mut self, k: usize) -> Result<Vec<usize>, anyhow::Error> {
        if k >= self.allocations.len() {
            return Err(anyhow::anyhow!(format!(
                "k is out of bounds: expected 0 ~ {}, got {}",
                self.allocations.len(),
                k
            )));
        }

        match &self.peak_sorted_sizes {
            Some(indices_sorted_by_peak) => Ok(indices_sorted_by_peak[..k].to_vec()),
            None => {
                log::info!("Sorting by peak globally");
                // create topk vector
                let mut peaks = self
                    .allocations
                    .iter()
                    .enumerate()
                    .map(|(i, alloc)| (i, alloc.peak_mem))
                    .collect::<Vec<(usize, u64)>>();

                // NOTE: sort descending
                peaks.sort_by(|(i1, peak1), (i2, peak2)| peak2.cmp(peak1));

                let indices_sorted_by_peak: Vec<usize> =
                    peaks.into_iter().map(|(index, peak)| index).collect();

                self.peak_sorted_sizes = Some(indices_sorted_by_peak.clone());

                dbg!(&indices_sorted_by_peak);

                Ok(indices_sorted_by_peak[..k].to_vec())
            }
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

        let top17 = memsnap.peak_topk(17).unwrap();

        dbg!(&top17);
        for i in top17 {
            println!(
                "Peak at allocation at index {}: {}",
                i,
                format_bytes(memsnap.allocations[i].peak_mem)
            );
        }
    }
}
