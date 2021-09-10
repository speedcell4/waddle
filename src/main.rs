use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Error, Read, Write};
use std::mem::transmute;
use std::panic::panic_any;
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

fn dump_offsets(offsets: &Vec<u64>, path: &Path) -> Result<(), io::Error> {
    let mut writer = OpenOptions::new()
        .write(true)
        .open(path)?;

    for &offset in offsets {
        let bytes = unsafe { transmute::<u64, [u8; 8]>(offset) };
        writer.write(&bytes)?;
    }

    Ok(())
}

fn load_offsets(path: &Path) -> Result<Vec<u64>, io::Error> {
    let mut reader = OpenOptions::new()
        .read(true)
        .open(path)?;

    let mut ret: Vec<u64> = Vec::new();
    let mut buf: [u8; 8] = [0; 8];

    while let Ok(..) = reader.read_exact(&mut buf) {
        let offset = unsafe {
            transmute::<[u8; 8], u64>(buf)
        };
        ret.push(offset);
    }

    Ok(ret)
}

fn main() -> Result<(), Error> {
    let offsets = collect_offsets(
        Path::new("data/example1.txt")
    )?;


    dump_offsets(&offsets, Path::new("data/nice.txt"))?;

    println!("offsets => {:?}", offsets);

    let miao = load_offsets(Path::new("data/nice.txt"))?;
    println!("miao => {:?}", miao);

    Ok(())
}
