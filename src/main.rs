use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;

use offset::Offsets;

mod offset;

struct LineWriter {
    buf: Vec<u8>,
}

impl LineWriter {
    fn new() -> LineWriter {
        LineWriter {
            buf: Vec::new(),
        }
    }

    fn write_line(&mut self,
                  offsets: &mut Offsets,
                  src: &mut BufReader<File>,
                  tgt: &mut BufWriter<File>) -> io::Result<()> {
        let offset = offsets.pop()
            .ok_or(io::Error::new(io::ErrorKind::Other, "offsets is empty"))?;

        src.seek(SeekFrom::Start(offset))?;

        self.buf.clear();
        src.read_until(b'\n', &mut self.buf)?;
        tgt.write(self.buf.as_slice())?;
        Ok(())
    }
}

struct FilesShuffler {
    rng: ThreadRng,
    line_writer: LineWriter,
}

impl FilesShuffler {
    fn new() -> FilesShuffler {
        FilesShuffler {
            rng: thread_rng(),
            line_writer: LineWriter::new(),
        }
    }

    fn build_offsets<'a>(&mut self, src_paths: &Vec<&'a Path>) -> HashMap<&'a Path, PathBuf> {
        src_paths.iter().map(|&src_path| {
            let mut offsets = offset::collect_offsets(src_path).unwrap();
            offsets.shuffle(&mut self.rng);

            let offset_path = src_path.with_extension(".offsets").to_owned();
            offset::dump_offsets(&offsets, offset_path.as_path()).unwrap();

            (src_path, offset_path)
        }).collect()
    }

    fn shuffle_files(&mut self, map: &HashMap<&Path, PathBuf>, paths: &Vec<&Path>, size: usize) -> () {
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
            let index1 = self.rng.gen_range(0..readers.len());
            let index2 = self.rng.gen_range(0..writers.len());

            let (reader, offsets) = readers.get_mut(index1).unwrap();
            let writer = writers.get_mut(index2).unwrap();

            self.line_writer.write_line(offsets, reader, writer).unwrap();
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
}


fn main() {
    let path1 = vec![Path::new("data/example1.txt")];
    let path2 = vec![Path::new("data/example1.out.txt")];

    let mut files_shuffler = FilesShuffler::new();


    let map = files_shuffler.build_offsets(&path1);
    files_shuffler.shuffle_files(&map, &path2, 1);
}
