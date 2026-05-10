use std::fs::File;
use std::fs::copy;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::path::PathBuf;

pub trait PatchRecord {
    fn apply(&self, file: &mut File, patch_source: &mut File) -> Result<(), std::io::Error>;
    fn size(&self) -> u16;
    fn offset(&self) -> u32;
    fn range(&self) -> (u32, u32);
}

struct RleRecord {
    offset: u32,
    size: u16,
    val: u8,
}

impl PatchRecord for RleRecord {
    fn apply(&self, file: &mut File, _: &mut File) -> Result<(), std::io::Error> {
        file.seek(SeekFrom::Start(self.offset as u64))?;
        file.write_all(vec![self.val; self.size as usize].as_slice())
            .map_err(|e| e.into())
    }

    fn offset(&self) -> u32 {
        self.offset
    }

    fn size(&self) -> u16 {
        self.size
    }

    fn range(&self) -> (u32, u32) {
        (self.offset, self.offset + self.size as u32)
    }
}

struct RawRecord {
    loc: u64,
    offset: u32,
    size: u16,
}

impl RawRecord {
    fn new(offset: u32, loc: u64, size: u16) -> Self {
        Self { offset, loc, size }
    }
}

impl PatchRecord for RawRecord {
    fn apply(&self, file: &mut File, patch_source: &mut File) -> Result<(), std::io::Error> {
        let mut bytes = vec![0; self.size as usize];
        patch_source.seek(SeekFrom::Start(self.loc))?;
        patch_source.read_exact(bytes.as_mut_slice())?;
        file.seek(SeekFrom::Start(self.offset as u64))?;
        file.write_all(bytes.as_slice())?;
        Ok(())
    }

    fn offset(&self) -> u32 {
        self.offset
    }

    fn size(&self) -> u16 {
        self.size
    }

    fn range(&self) -> (u32, u32) {
        (self.offset, self.offset + self.size as u32)
    }
}

struct IPSPatch {
    patch_source: PathBuf,
    records: Vec<Box<dyn PatchRecord>>,
}

impl IPSPatch {
    fn new(patch_source: &Path) -> Self {
        Self {
            patch_source: patch_source.into(),
            records: Vec::<Box<dyn PatchRecord>>::new(),
        }
    }

    fn check_header(&self, mut patch: &File) -> bool {
        const HEADER: [u8; 5] = *b"PATCH";
        let mut buffer = [0u8; 5];
        patch.read_exact(&mut buffer).unwrap();
        HEADER == buffer
    }

    fn read_records(&mut self) {
        let mut patch = File::open(self.patch_source.clone()).unwrap();

        if !self.check_header(&patch) {
            return;
        }

        let mut buffer = [0u8; 5];
        let mut rle_buffer = [0u8; 3];
        patch.seek(SeekFrom::Start(5)).unwrap();

        while patch.read_exact(&mut buffer).is_ok() {
            let offset = (buffer[0] as u32) << 16 | (buffer[1] as u32) << 8 | (buffer[2] as u32);
            let size = (buffer[3] as u16) << 8 | (buffer[4] as u16);
            if size == 0 {
                // run length encoded
                patch.read_exact(&mut rle_buffer).unwrap();
                let size = (rle_buffer[0] as u16) << 8 | (rle_buffer[1] as u16);
                self.records.push(Box::new(RleRecord {
                    offset,
                    size,
                    val: rle_buffer[2],
                }));
            } else {
                let loc = patch.stream_position().unwrap();
                self.records
                    .push(Box::new(RawRecord::new(offset, loc, size)));
                patch.seek(SeekFrom::Current(size as i64)).unwrap();
            }
        }
    }

    fn apply(&mut self, in_rom: &Path, out_rom: &Path) {
        if self.records.is_empty() {
            self.read_records();
        }
        if self.records.is_empty() {
            return;
        }

        let mut patch = File::open(self.patch_source.as_path()).unwrap();
        let mut out_rom_file = File::create(out_rom).unwrap();
        copy(in_rom, out_rom).unwrap();

        for record in &self.records {
            record.apply(&mut out_rom_file, &mut patch).unwrap();
        }
    }
}

fn main() {
    let patch_file = Path::new("testfiles/patch.ips");
    let in_rom = Path::new("testfiles/base_rom");
    let out_rom = Path::new("testfiles/patched_rom");

    let mut ipspatch = IPSPatch::new(patch_file);

    ipspatch.apply(in_rom, out_rom);
}
