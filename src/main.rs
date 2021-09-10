use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::mem::transmute;
use std::path::{Path, PathBuf};

use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;

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

fn shuffle_files(map: &HashMap<&Path, PathBuf>, paths: &Vec<&Path>, size: usize) {
    let mut rng = thread_rng();
    let mut copy_line = CopyLine::new();

    let mut iter = map.iter();
    let mut readers: Vec<_> = (0..size).map(|_| iter.next().unwrap()).map(|(path, offset_path)| {
        let file = OpenOptions::new().read(true).open(path).unwrap();
        let offsets = load_offsets(offset_path.as_path()).unwrap();
        (BufReader::new(file), offsets)
    }).collect();

    let mut writers: Vec<_> = paths.iter().map(|path| {
        let file = OpenOptions::new().create(true).write(true).open(path).unwrap();
        BufWriter::new(file)
    }).collect();

    while !readers.is_empty() {
        let index1 = rng.gen_range(0..readers.len());
        let index2 = rng.gen_range(0..writers.len());

        let (reader, offsets) = readers.get_mut(index1).unwrap();
        let writer = writers.get_mut(index2).unwrap();

        copy_line.copy_line(offsets, reader, writer);
        if offsets.len() == 0 {
            readers.remove(index1);
            if let Some((path, offset_path)) = iter.next() {
                let file = OpenOptions::new().read(true).open(path).unwrap();
                let offsets = load_offsets(offset_path.as_path()).unwrap();
                readers.push((BufReader::new(file), offsets));
            }
        }
    }
}

fn main() {
    let path1 = vec![Path::new("data/example1.txt")];
    let path2 = vec![Path::new("data/example1.out.txt")];

    let map = build_offsets(&path1);
    shuffle_files(&map, &path2, 1);
}
