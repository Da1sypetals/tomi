use crate::snapshot::{Allocation, ElementData, RawAllocationData};
use serde::Deserialize;
use std::error::Error;
use std::fs;

pub fn load_allocations(
    alloc_path: &str,
    elements_path: &str,
) -> Result<Vec<Allocation>, Box<dyn Error>> {
    // Read and parse allocations.json
    let alloc_content = fs::read_to_string(alloc_path)
        .map_err(|e| format!("Failed to read allocations file '{}': {}", alloc_path, e))?;
    let raw_allocs: Vec<RawAllocationData> = serde_json::from_str(&alloc_content).map_err(|e| {
        format!(
            "Failed to parse allocations JSON from '{}': {}",
            alloc_path, e
        )
    })?;

    // Read and parse elements.json
    let elements_content = fs::read_to_string(elements_path)
        .map_err(|e| format!("Failed to read elements file '{}': {}", elements_path, e))?;
    // elements.json is a list, where each item has a "frames" key.
    let elements_data: Vec<ElementData> = serde_json::from_str(&elements_content).map_err(|e| {
        format!(
            "Failed to parse elements JSON from '{}': {}",
            elements_path, e
        )
    })?;

    // Check if the number of allocations matches the number of element data (callstacks)
    if raw_allocs.len() != elements_data.len() {
        return Err(format!(
            "Mismatch in the number of entries: {} allocations vs {} elements",
            raw_allocs.len(),
            elements_data.len()
        )
        .into());
    }

    // Combine the data from raw_allocs and elements_data (callstacks)
    let allocations: Vec<Allocation> = raw_allocs
        .into_iter()
        .zip(elements_data.into_iter())
        .map(|(raw_alloc, element_data)| Allocation {
            timesteps: raw_alloc.timesteps,
            offsets: raw_alloc.offsets,
            size: raw_alloc.size,
            callstack: element_data.frames, // element_data.frames is Vec<Frame>
        })
        .collect();

    Ok(allocations)
}

#[cfg(test)]
mod tests {
    use super::load_allocations;

    #[test]
    fn test_basic() {
        // These paths should point to your actual JSON files
        // For demonstration, you'd create dummy files named allocations.json and elements.json
        // in a 'snapshots' directory relative to where you run the executable.
        let alloc_path = "../snapshots/allocations.json";
        let elements_path = "../snapshots/elements.json";

        match load_allocations(alloc_path, elements_path) {
            Ok(allocations) => {
                if allocations.is_empty() {
                    println!("No allocations were loaded.");
                } else {
                    println!("Successfully loaded {} allocations:", allocations.len());

                    if let Some(first_alloc) = allocations.first() {
                        println!("{:#?}", first_alloc);
                    }
                    // println!("{:#?}", allocations);
                }
            }
            Err(e) => {
                eprintln!("Error loading allocations: {}", e);
            }
        }
    }
}
