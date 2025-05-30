use snap_rs::repl_ops::memsnap::MemSnap;

fn main() {
    let zip_path = "../snapshots/large/transformer.zip";

    match MemSnap::from_zip(zip_path) {
        Ok(snap) => {
            dbg!(snap.allocations.len());
        }
        Err(e) => {
            eprintln!("Error loading allocations: {}", e);
        }
    }
}
