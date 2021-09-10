use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::mem::transmute;
use std::path::{Path, PathBuf};

use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;

use offset::{Offset, Offsets};

mod offset;

struct CopyLine {
    buf: Vec<u8>,
}

impl CopyLine {
    fn new() -> CopyLine {
        CopyLine {
            buf: Vec::new(),
        }
    }

    fn copy_line(&mut self, offsets: &mut Offsets,
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
        let mut offsets = offset::collect_offsets(path).unwrap();
        offsets.shuffle(&mut rng);

        let offset_path = path.with_extension(".offsets").to_owned();
        offset::dump_offsets(&offsets, offset_path.as_path());

        (path, offset_path)
    }).collect()
}

fn shuffle_files(map: &HashMap<&Path, PathBuf>, paths: &Vec<&Path>, size: usize) {
    let mut rng = thread_rng();
    let mut copy_line = CopyLine::new();

    let mut iter = map.iter();
    let mut readers: Vec<_> = (0..size).map(|_| iter.next().unwrap()).map(|(path, offset_path)| {
        let file = OpenOptions::new().read(true).open(path).unwrap();
        let offsets = offset::load_offsets(offset_path.as_path()).unwrap();
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
                let offsets = offset::load_offsets(offset_path.as_path()).unwrap();
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
