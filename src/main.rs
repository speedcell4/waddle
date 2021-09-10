use std::fs::{File, OpenOptions, read};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Error, Read, Seek, SeekFrom, Write};
use std::mem::transmute;
use std::panic::panic_any;
use std::path::Path;
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


fn main() -> Result<(), Error> {
    let mut offsets = collect_offsets(
        Path::new("data/example1.txt")
    )?;


    dump_offsets(&offsets, Path::new("data/nice.txt"))?;
    let miao = load_offsets(Path::new("data/nice.txt"))?;

    let mut reader: BufReader<File> = BufReader::new(OpenOptions::new()
        .read(true)
        .open("data/example1.txt")?);
    let mut writer: BufWriter<File> = BufWriter::new(OpenOptions::new()
        .create(true)
        .write(true)

        .open("data/nice.out.txt")?);

    let mut copy_line = CopyLine::new();

    copy_line.copy_line(&mut offsets, &mut reader, &mut writer)?;

    Ok(())
}
