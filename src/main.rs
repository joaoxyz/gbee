mod cpu;

use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let boot_rom = fs::read("dmg_boot.bin")?;

    let mut cpu: cpu::Cpu = Default::default();
    cpu.load_rom(&boot_rom, 0);

    for _i in 0..0xFFF {
        let a = cpu.fetch();
        println!("{a:#X?}");
        cpu.decode(a);
    }

    println!("{cpu:#?}");
    Ok(())
}
