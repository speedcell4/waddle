use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

use rand::prelude::SliceRandom;
use rand::thread_rng;

fn collect_offsets(path: &Path) -> Result<Vec<u64>, io::Error> {
    let mut offsets: Vec<_> = File::open(path)?.bytes()
        .zip(0u64..)
        .filter_map(|(byte, index)| {
            if let Ok(b) = byte {
                if b == b'\n' {
                    return Some(index + 1);
                }
            }
            None
        })
        .collect();
    offsets.push(0);

    Ok(offsets)
}


fn main() {
    let offsets = collect_offsets(
        Path::new("data/example1.txt")
    );


    println!("offsets => {:?}", offsets);
}
