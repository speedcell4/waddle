use std::collections::HashMap;
use std::env::temp_dir;
use std::fs::{File, OpenOptions, read};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Error, Read, Seek, SeekFrom, Write};
use std::mem::transmute;
use std::panic::panic_any;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

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
        .create(true)
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

struct CopyLine {
    buf: Vec<u8>,
}

impl CopyLine {
    fn new() -> CopyLine {
        CopyLine {
            buf: Vec::new(),
        }
    }

    fn copy_line(&mut self, offsets: &mut Vec<u64>,
                 src: &mut BufReader<File>,
                 tgt: &mut BufWriter<File>) -> std::io::Result<()> {
        let offset = offsets.pop().unwrap();
        src.seek(SeekFrom::Start(offset))?;

        self.buf.clear();
        src.read_until(b'\n', &mut self.buf)?;
        tgt.write(self.buf.as_slice())?;
        Ok(())
    }
}

fn build_offsets<'a>(paths: &Vec<&'a Path>) -> HashMap<&'a Path, PathBuf> {
    let mut rng = thread_rng();

    paths.iter().map(|&path| {
        let mut offsets = collect_offsets(path).unwrap();
        offsets.shuffle(&mut rng);

        let offset_path = path.with_extension(".offsets").to_owned();
        dump_offsets(&offsets, offset_path.as_path());

        (path, offset_path)
    }).collect()
}

fn main() -> Result<(), Error> {
    let paths = vec![
        Path::new("data/example1.txt"),
    ];
    println!("build_offsets(&paths) => {:?}", build_offsets(&paths));

    Ok(())
}
