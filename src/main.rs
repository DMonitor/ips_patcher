use std::path::Path;

mod ips;

fn main() {
    let patch_file = Path::new("testfiles/patch.ips");
    let in_rom = Path::new("testfiles/base_rom");
    let out_rom = Path::new("testfiles/patched_rom");

    let mut ipspatch = ips::IPSPatch::new(patch_file);

    ipspatch.apply(in_rom, out_rom).unwrap();
}
