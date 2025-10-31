use core::panic;

#[derive(Debug)]
pub struct CPU {
    ram: [u8; 0xFFFF],

    // registers
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            ram: [0; 0xFFFF],
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }
}

impl CPU {
    pub fn load_rom(&mut self, rom: &[u8]) {
        self.ram[..rom.len()].copy_from_slice(rom);
    }

    fn get_low(reg: u16) -> u8 {
        (reg & 0x00FF) as u8
    }

    fn get_high(reg: u16) -> u8 {
        (reg >> 8) as u8
    }

    fn join_u8(high: u8, low: u8) -> u16 {
        ((high as u16) << 8) | low as u16
    }

    pub fn fetch(&mut self) -> u8 {
        let byte = self.ram[self.pc as usize];
        self.pc += 1;
        byte
    }

    pub fn decode(&mut self, opcode: u8) {
        match opcode {
            0x00 => (), // nop
            0x76 => (), // halt
            0xCB => {
                // CB prefixed instuctions
                let new_opcode = self.fetch();
                self.decode_prefixed(new_opcode);
            },
            0x40..=0x7F => (), // load
            0x80..=0x87 => {
                // add
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu(register);
                self.inst_add(rhs_value);
            },
            0x88..=0x8F => (), // add with carry
            0x90..=0x97 => {
                // sub
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu(register);
                self.inst_sub(rhs_value);
            },
            0x98..=0x9F => (), // sub with carry
            0xA0..=0xA7 => {
                // and
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu(register);
                self.inst_and(rhs_value);
            },
            0xA8..=0xAF => {
                // xor
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu(register);
                self.inst_xor(rhs_value);
            },
            0xB0..=0xB7 => {
                // or
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu(register);
                self.inst_or(rhs_value);
            },
            0xB8..=0xBF => {
                // compare
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu(register);
                self.inst_compare(rhs_value);
            },
            _ => eprintln!("Unknown opcode: {opcode:#04X}"),
        }
    }

    fn decode_prefixed(&mut self, opcode: u8) {
        todo!()
    }

    fn decode_alu(&self, n: u8) -> u8 {
        match n {
            0b000 => self.b,
            0b001 => self.c,
            0b010 => self.d,
            0b011 => self.e,
            0b100 => self.h,
            0b101 => self.l,
            0b110 => self.ram[CPU::join_u8(self.h, self.l) as usize],
            0b111 => self.a,
            n => panic!("Cannot match RHS operand for ADD A, {n:b}"),
        }
    }

    // TODO: Set flags
    fn inst_add(&mut self, value: u8) {
        let mut flags = 0 as u8;
        flags |= Flags::Zero as u8;

        self.a += value;
        self.set_flags(flags);
    }

    fn inst_sub(&mut self, value: u8) {
        self.a -= value;
    }

    fn inst_and(&mut self, value: u8) {
        self.a &= value;
    }

    fn inst_xor(&mut self, value: u8) {
        self.a ^= value;
    }

    fn inst_or(&mut self, value: u8) {
        let result = self.a | value;

        let mut flags = 0 as u8;
        if result == 0 {
            flags |= Flags::Zero as u8;
        }

        self.a = result;
        self.set_flags(flags);
    }

    fn inst_compare(&mut self, value: u8) {
        let result = self.a - value;
    }

    fn inst_ld(dst: &mut u8, src: u8) {
        *dst = src;
    }

    fn set_flags(&mut self, flags: u8) {
        self.f |= flags;
    }
}

#[repr(u8)]
enum Flags {
    Zero = 0b10000000,
    Subtraction = 0b01000000,
    HalfCarry = 0b00100000,
    Carry = 0b00010000,
}
