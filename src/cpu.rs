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
    pub fn load_rom(&mut self, rom: &[u8], pos: usize) {
        self.ram[pos..rom.len()].copy_from_slice(rom);
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
            0x00 => todo!(), // nop
            0x76 => todo!(), // halt
            0xCB => {
                // CB prefixed instuctions
                let new_opcode = self.fetch();
                self.decode_prefixed(new_opcode);
            },
            0x40..=0x7F => todo!(), // load
            0x80..=0x87 => {
                // add
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu_operand(register);
                self.inst_add(rhs_value);
            },
            0x88..=0x8F => {
                // add with carry
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu_operand(register);
                self.inst_add_carry(rhs_value);
            }, 
            0x90..=0x97 => {
                // sub
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu_operand(register);
                self.inst_sub(rhs_value);
            },
            0x98..=0x9F => {
                // sub with carry
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu_operand(register);
                self.inst_sub_carry(rhs_value);
            }, 
            0xA0..=0xA7 => {
                // and
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu_operand(register);
                self.inst_and(rhs_value);
            },
            0xA8..=0xAF => {
                // xor
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu_operand(register);
                self.inst_xor(rhs_value);
            },
            0xB0..=0xB7 => {
                // or
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu_operand(register);
                self.inst_or(rhs_value);
            },
            0xB8..=0xBF => {
                // compare
                let register = opcode & 0b00000111;
                let rhs_value = self.decode_alu_operand(register);
                self.inst_compare(rhs_value);
            },
            _ => eprintln!("Unknown opcode: {opcode:#04X}"),
        }
    }

    // Decode opcodes prefixed with $CB
    fn decode_prefixed(&mut self, opcode: u8) {
        todo!()
    }

    fn decode_alu_operand(&self, n: u8) -> u8 {
        // Disassembly table for 8-bit registers
        // https://archive.gbdev.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
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

    fn decode_load_operands(&self, x: u8, y: u8) -> (&u8, u8) {
        let src = match x { 
            _ => todo!()
        };

        let dst = match y {
            _ => todo!()
        };

        (dst, src)
    }

    fn inst_add(&mut self, value: u8) {
        let mut flags: u8 = 0;
        let (result, carry) = self.a.carrying_add(value, false);

        if result == 0 {
            flags |= Flags::Zero;
        }

        let half_carry = ((self.a & 0b1111) + (value & 0b1111)) > 0b1111;
        if half_carry {
            flags |= Flags::HalfCarry;
        }

        if carry {
            flags |= Flags::Carry;
        }

        self.a = result;
        self.f = flags;
    }

    fn inst_add_carry(&mut self, value: u8) {
        let mut flags: u8 = 0;
        let current_carry_flag = ((self.f & Flags::Carry) > 0) as u8;
        let (result, carry0) = self.a.carrying_add(value, false);
        let (result, carry1) = result.carrying_add(current_carry_flag, carry0);

        if result == 0 {
            flags |= Flags::Zero;
        }

        let half_carry = ((self.a & 0b1111) + (value & 0b1111) + (current_carry_flag)) > 0b1111;
        if half_carry {
            flags |= Flags::HalfCarry;
        }

        // if any of the two adds overflow we should set carry
        let carry = carry0 | carry1;
        if carry {
            flags |= Flags::Carry;
        }

        self.a = result;
        self.f = flags;
    }

    fn inst_sub(&mut self, value: u8) {
        let mut flags: u8 = 0;
        let (result, carry) = self.a.borrowing_sub(value, false);

        if result == 0 {
            flags |= Flags::Zero;
        }

        flags |= Flags::Subtraction;

        let half_carry = (self.a & 0b1111) < (value & 0b1111);
        if half_carry {
            flags |= Flags::HalfCarry;
        }

        if carry {
            flags |= Flags::Carry;
        }

        self.a = result;
        self.f = flags;
    }

    fn inst_sub_carry(&mut self, value: u8) {
        let mut flags: u8 = 0;
        let current_carry_flag = ((self.f & Flags::Carry) > 0) as u8;
        let (result, carry0) = self.a.borrowing_sub(value, false);
        let (result, carry1) = result.borrowing_sub(current_carry_flag, carry0);

        if result == 0 {
            flags |= Flags::Zero;
        }

        let half_carry = (self.a & 0b1111) < ((value & 0b1111) + (current_carry_flag));
        if half_carry {
            flags |= Flags::HalfCarry;
        }

        // if any of the two subs overflow we should set carry
        // according to opcode table SBC A, A does not affect the carry flag, look this up later
        let carry = carry0 | carry1;
        if carry {
            flags |= Flags::Carry;
        }

        self.a = result;
        self.f = flags;
    }

    fn inst_and(&mut self, value: u8) {
        let mut flags: u8 = 0;
        let result = self.a & value;

        if result == 0 {
            flags |= Flags::Zero;
        }

        flags |= Flags::HalfCarry;

        self.a = result;
        self.f = flags;
    }

    fn inst_xor(&mut self, value: u8) {
        let mut flags: u8 = 0;
        let result = self.a ^ value;

        flags |= Flags::Zero;

        self.a = result;
        self.f = flags;
    }

    fn inst_or(&mut self, value: u8) {
        let mut flags: u8 = 0;
        let result = self.a | value;

        if result == 0 {
            flags |= Flags::Zero;
        }

        self.a = result;
        self.f = flags;
    }

    fn inst_compare(&mut self, value: u8) {
        let mut flags: u8 = 0;
        let (result, carry) = self.a.borrowing_sub(value, false);

        if result == 0 {
            flags |= Flags::Zero;
        }

        flags |= Flags::Subtraction;

        let half_carry = (self.a & 0b1111) < (value & 0b1111);
        if half_carry {
            flags |= Flags::HalfCarry;
        }

        if carry {
            flags |= Flags::Carry;
        }

        self.f = flags;
    }

    fn inst_ld(dst: &mut u8, src: u8) {
        *dst = src;
        todo!()
    }
}

#[repr(u8)]
enum Flags {
    Zero = 0b10000000,
    Subtraction = 0b01000000,
    HalfCarry = 0b00100000,
    Carry = 0b00010000,
}

impl std::ops::BitOrAssign<Flags> for u8 {
    fn bitor_assign(&mut self, rhs: Flags) {
        *self |= rhs as u8;
    }
}

impl std::ops::BitAnd<Flags> for u8 {
    type Output = Self;
    fn bitand(self, rhs: Flags) -> Self::Output {
        self & rhs as u8
    }
}
