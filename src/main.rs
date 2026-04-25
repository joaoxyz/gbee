mod cpu;

use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let boot_rom = fs::read("roms/dmg_boot.bin")?;

    let mut cpu: cpu::Cpu = Default::default();
    cpu.load_rom(&boot_rom, 0);

    loop {
        let a = cpu.fetch();
        cpu.decode(a);
        println!("{cpu:?}");
    }
}
