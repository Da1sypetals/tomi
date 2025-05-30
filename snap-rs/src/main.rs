// use snap_rs::repl_ops::memsnap::MemSnap;

// fn main() {
//     let zip_path = "../snapshots/large/transformer.zip";

//     match MemSnap::from_zip(zip_path) {
//         Ok(snap) => {
//             dbg!(snap.allocations.len());
//         }
//         Err(e) => {
//             eprintln!("Error loading allocations: {}", e);
//         }
//     }
// }

use std::time::Instant;

const NUM_STRINGS: usize = 10000;
const STRING_LENGTH: usize = 100000;

// Helper function to create a string of a given length
fn create_long_string(length: usize) -> String {
    "a".repeat(length)
}

fn main() {
    println!("Benchmarking string merging operations...");
    println!("Number of strings: {}", NUM_STRINGS);
    println!("Length of each string: {}", STRING_LENGTH);
    println!(
        "Total expected length: {} bytes",
        NUM_STRINGS * STRING_LENGTH
    );

    // Prepare the strings once for all benchmarks
    let strings_to_merge: Vec<String> = (0..NUM_STRINGS)
        .map(|_| create_long_string(STRING_LENGTH))
        .collect();

    // Convert to slices for `join` and `collect`
    let string_slices: Vec<&str> = strings_to_merge.iter().map(|s| s.as_str()).collect();

    // --- Benchmark 1: push_str with pre-allocated capacity ---
    let start_time = Instant::now();
    let mut result_push_str_capacity = String::with_capacity(NUM_STRINGS * STRING_LENGTH);
    for s in strings_to_merge.iter() {
        result_push_str_capacity.push_str(s);
    }
    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    println!("\npush_str (with capacity): {} ms", duration.as_millis());
    // Use black_box to prevent the compiler from optimizing away the result
    // (This is a simple way to achieve a similar effect to `test::black_box` for std-only)
    // In a real scenario, you might print or use the result to ensure it's not optimized out.
    // Here, we just declare it as a mutable variable, which is often enough.
    let _ = result_push_str_capacity;

    // --- Benchmark 2: push_str without pre-allocated capacity ---
    let start_time = Instant::now();
    let mut result_push_str_no_capacity = String::new();
    for s in strings_to_merge.iter() {
        result_push_str_no_capacity.push_str(s);
    }
    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    println!("push_str (no capacity):   {} ms", duration.as_millis());
    let _ = result_push_str_no_capacity;

    // --- Benchmark 3: join ---
    let start_time = Instant::now();
    let result_join = string_slices.join(",");
    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    println!("join:                     {} ms", duration.as_millis());
    let _ = result_join;

    // --- Benchmark 4: collect::<String>() ---
    let start_time = Instant::now();
    let result_collect: String = string_slices.iter().copied().collect();
    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    println!("collect::<String>():      {} ms", duration.as_millis());
    let _ = result_collect;

    println!("\nNote: Times can vary between runs due to system load and other factors.");
    println!(
        "For more precise and reliable benchmarking, consider using `cargo bench` with `criterion`."
    );
}
