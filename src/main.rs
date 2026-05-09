use std::fs::File;
use std::fs::copy;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

fn check_header(mut f: &File) -> bool {
    const HEADER: [u8;5] = *b"PATCH";
    let mut buffer = [0u8;5];
    f.read_exact( &mut buffer ).unwrap();
    HEADER == buffer
}

fn read_record(mut patch: &File, mut rom: &File) -> bool {
    let mut buffer = [0u8;5];
    let mut rle_buffer = [0u8;3];
    if !patch.read_exact( &mut buffer ).is_ok() {
        return false;
    }
    let offset = (buffer[0] as u32) << 16 | (buffer[1] as u32) << 8 | (buffer[2] as u32);
    let size = (buffer[3] as u16) << 8 | (buffer[4] as u16);

    rom.seek( SeekFrom::Start(offset as u64) ).unwrap();
    let mut l = vec![0; size as usize];
    if size == 0 {
        // run length encoded
        patch.read_exact( &mut rle_buffer ).unwrap();
        let size = (rle_buffer[0] as u16) << 8 | (rle_buffer[1] as u16);
        l = vec![rle_buffer[2]; size as usize];
    }
    else {
        patch.read_exact(l.as_mut_slice()).unwrap();
    }
    rom.write(l.as_slice()).unwrap();
    true
}

fn main() {
    let mut patchfile = File::open("testfiles/patch.ips").unwrap();
    if false == check_header(&mut patchfile) {
        println!("Patch header not found");
        return;
    }
    let in_rom = Path::new("testfiles/base_rom");
    let out_rom = Path::new("testfiles/patched_rom");
    let out_rom_file = File::create(out_rom).unwrap();
    copy(in_rom,out_rom).unwrap();

    while read_record(&patchfile,&out_rom_file) {}

    out_rom_file.sync_all().unwrap();

}
