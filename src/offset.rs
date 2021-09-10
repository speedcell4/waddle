use std::fs::OpenOptions;
use std::intrinsics::transmute;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

pub type Offset = u64;
pub type Offsets = Vec<Offset>;

pub fn collect_offsets(path: &Path) -> io::Result<Offsets> {
    let mut offsets: Vec<_> = OpenOptions::new().read(true).open(path)?.bytes()
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

pub fn dump_offsets(offsets: &Offsets, path: &Path) -> io::Result<()> {
    let mut writer = OpenOptions::new().create(true).write(true).open(path)?;

    for offset in offsets {
        let bytes = unsafe { transmute::<Offset, [u8; 8]>(*offset) };
        writer.write(&bytes)?;
    }

    Ok(())
}

pub fn load_offsets(path: &Path) -> io::Result<Offsets> {
    let mut buf: [u8; 8] = [0; 8];
    let mut reader = OpenOptions::new().read(true).open(path)?;

    let mut offsets: Offsets = Vec::new();
    while let Ok(..) = reader.read_exact(&mut buf) {
        let offset = unsafe {
            transmute::<[u8; 8], Offset>(buf)
        };
        offsets.push(offset);
    }

    Ok(offsets)
}
