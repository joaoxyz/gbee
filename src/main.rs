mod cpu;

use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let boot_rom = fs::read("dmg_boot.bin")?;

    let mut cpu: cpu::CPU = Default::default();
    cpu.load_rom(&boot_rom, 0);

    for i in 0..200 {
        let a = cpu.fetch();
        cpu.decode(a);
    } 

    Ok(())
}
