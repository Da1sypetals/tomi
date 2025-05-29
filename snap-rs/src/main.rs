use snap_rs::load::load_allocations;

fn main() {
    let alloc_path = "../snapshots/allocations.json";
    let elements_path = "../snapshots/elements.json";

    match load_allocations(alloc_path, elements_path) {
        Ok(allocations) => {
            if allocations.is_empty() {
                println!("No allocations were loaded.");
            } else {
                println!("Successfully loaded {} allocations:", allocations.len());

                let alloc = &allocations[3];

                dbg!(&alloc.timesteps);
                dbg!(&alloc.offsets);
                dbg!(&alloc.size);
                dbg!(&alloc.callstack[13]);
                // println!("{:#?}", allocations);
            }
        }
        Err(e) => {
            eprintln!("Error loading allocations: {}", e);
        }
    }
}
