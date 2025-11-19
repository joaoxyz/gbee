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

// macro_rules! decode_operand_u8_mut {
//     ($self:expr, $n:expr) => {
//         match $n {
//             0b000 => &mut $self.b,
//             0b001 => &mut $self.c,
//             0b010 => &mut $self.d,
//             0b011 => &mut $self.e,
//             0b100 => &mut $self.h,
//             0b101 => &mut $self.l,
//             0b110 => &mut $self.ram[CPU::join_u8($self.h, $self.l) as usize],
//             0b111 => &mut $self.a,
//             n => panic!("Cannot match DST operand for LD, {n:b}"),
//         }
//     };
// }

impl CPU {
    pub fn load_rom(&mut self, rom: &[u8], pos: usize) {
        self.ram[pos..rom.len()].copy_from_slice(rom);
    }

    fn join_u8(high: u8, low: u8) -> u16 {
        ((high as u16) << 8) | low as u16
    }

    fn split_u16(value: u16) -> (u8, u8) {
        ((value >> 8) as u8, (value & 0x00FF) as u8)
    }

    pub fn fetch(&mut self) -> u8 {
        let byte = self.ram[self.pc as usize];
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn fetch_u16(&mut self) -> u16 {
        let low = self.ram[self.pc as usize];
        let high = self.ram[(self.pc+1) as usize];
        self.pc = self.pc.wrapping_add(2);

        Self::join_u8(high, low)
    }

    pub fn decode(&mut self, opcode: u8) {
        match opcode {
            // nop
            0x00 => self.inst_nop(),
            0x10 => {self.fetch();}, // stop
            0x76 => todo!(), // halt
            0xCB => {
                // CB prefixed instuctions
                let new_opcode = self.fetch();
                self.decode_prefixed_cb(new_opcode);
            },
            0x02 => {
                let offset = Self::join_u8(self.b, self.c);
                self.inst_load_to_memory(offset);
            },
            0x12 => {
                let offset = Self::join_u8(self.d, self.e);
                self.inst_load_to_memory(offset);
            },
            // TODO: move HL increment and decrement ops to instruction code
            0x22 => {
                let offset = Self::join_u8(self.h, self.l);
                self.inst_load_to_memory(offset);
                let offset = offset.wrapping_add(1);
                (self.h, self.l) = Self::split_u16(offset);
            },
            0x32 => {
                let offset = Self::join_u8(self.h, self.l);
                self.inst_load_to_memory(offset);
                let offset = offset.wrapping_sub(1);
                (self.h, self.l) = Self::split_u16(offset);
            },
            0x0A => {
                let offset = Self::join_u8(self.b, self.c);
                self.inst_load_from_memory(offset);
            },
            0x1A => {
                let offset = Self::join_u8(self.d, self.e);
                self.inst_load_from_memory(offset);
            },
            // TODO: move HL increment and decrement ops to instruction code
            0x2A => {
                let offset = Self::join_u8(self.h, self.l);
                self.inst_load_from_memory(offset);
                let offset = offset.wrapping_add(1);
                (self.h, self.l) = Self::split_u16(offset);
            },
            0x3A => {
                let offset = Self::join_u8(self.h, self.l);
                self.inst_load_from_memory(offset);
                let offset = offset.wrapping_sub(1);
                (self.h, self.l) = Self::split_u16(offset);
            },
            0xEA => {
                let offset = self.fetch_u16();
                self.inst_load_to_memory(offset);
            },
            0xFA => {
                let offset = self.fetch_u16();
                self.inst_load_from_memory(offset);
            },
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {

            }
            0x27 => self.inst_daa(),
            0x37 => self.inst_scf(),
            // 16-bit register increment
            0x03 => self.inst_inc_bc(),
            0x13 => self.inst_inc_de(),
            0x23 => self.inst_inc_hl(),
            0x33 => self.inst_inc_sp(),
            // 16-bit register decrement
            0x0B => self.inst_dec_bc(),
            0x1B => self.inst_dec_de(),
            0x2B => self.inst_dec_hl(),
            0x3B => self.inst_dec_sp(),
            0x2F => self.inst_cpl(),
            0x3F => self.inst_ccf(),
            // ALU operations with 8-bit immediates
            0xC6 => {
                let immediate = self.fetch();
                self.inst_add(immediate);
            },
            0xD6 => {
                let immediate = self.fetch();
                self.inst_sub(immediate);
            },
            0xE6 => {
                let immediate = self.fetch();
                self.inst_and(immediate);
            },
            0xF6 => {
                let immediate = self.fetch();
                self.inst_or(immediate);
            },
            0xCE => {
                let immediate = self.fetch();
                self.inst_add_carry(immediate);
            },
            0xDE => {
                let immediate = self.fetch();
                self.inst_sub_carry(immediate);
            },
            0xEE => {
                let immediate = self.fetch();
                self.inst_xor(immediate);
            },
            0xFE => {
                let immediate = self.fetch();
                self.inst_compare(immediate);
            },
            // 16-bit loads
            0x01 => {
                let immediate = self.fetch_u16();
                self.inst_load_bc(immediate);
            },
            0x11 => {
                let immediate = self.fetch_u16();
                self.inst_load_de(immediate);
            },
            0x21 => {
                let immediate = self.fetch_u16();
                self.inst_load_hl(immediate);
            },
            0x31 => {
                let immediate = self.fetch_u16();
                self.inst_load_sp(immediate);
            },
            0x40..=0x7F => {
                // reg to reg 8-bit loads
                let src = opcode & 0b00000111;
                let dst = (opcode & 0b00111000) >> 3;
                let (src, dst) = (self.decode_operand_u8(src), self.decode_operand_u8_mut(dst));
                Self::inst_load(dst, src);
            },
            0x80..=0x87 => {
                // add
                let rhs_value = opcode & 0b00000111;
                let rhs_value = self.decode_operand_u8(rhs_value);
                self.inst_add(rhs_value);
            },
            0x88..=0x8F => {
                // add with carry
                let rhs_value = opcode & 0b00000111;
                let rhs_value = self.decode_operand_u8(rhs_value);
                self.inst_add_carry(rhs_value);
            },
            0x90..=0x97 => {
                // sub
                let rhs_value = opcode & 0b00000111;
                let rhs_value = self.decode_operand_u8(rhs_value);
                self.inst_sub(rhs_value);
            },
            0x98..=0x9F => {
                // sub with carry
                let rhs_value = opcode & 0b00000111;
                let rhs_value = self.decode_operand_u8(rhs_value);
                self.inst_sub_carry(rhs_value);
            },
            0xA0..=0xA7 => {
                // and
                let rhs_value = opcode & 0b00000111;
                let rhs_value = self.decode_operand_u8(rhs_value);
                self.inst_and(rhs_value);
            },
            0xA8..=0xAF => {
                // xor
                let rhs_value = opcode & 0b00000111;
                let rhs_value = self.decode_operand_u8(rhs_value);
                self.inst_xor(rhs_value);
            },
            0xB0..=0xB7 => {
                // or
                let rhs_value = opcode & 0b00000111;
                let rhs_value = self.decode_operand_u8(rhs_value);
                self.inst_or(rhs_value);
            },
            0xB8..=0xBF => {
                // compare
                let rhs_value = opcode & 0b00000111;
                let rhs_value = self.decode_operand_u8(rhs_value);
                self.inst_compare(rhs_value);
            },
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB..=0xED | 0xF4 | 0xFC | 0xFD => println!("Unused opcode: {opcode:#04X}"), // unused opcodes, listed for match sanity check
            _ => eprintln!("Unknown opcode: {opcode:#04X}"),
        }
    }

    // Decode opcodes prefixed with $CB
    fn decode_prefixed_cb(&mut self, opcode: u8) {
        match opcode {
            0x00..=0x07 => {
                // rlc
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = self.decode_operand_u8_mut(rhs_value);
                Self::inst_rlc(&mut self.f, rhs_value);
            },
            0x08..=0x0F => {
                // rrc
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = self.decode_operand_u8_mut(rhs_value);
                Self::inst_rrc(&mut self.f, rhs_value);
            },
            0x10..=0x17 => {
                // rl
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = self.decode_operand_u8_mut(rhs_value);
                Self::inst_rl(&mut self.f, rhs_value);
            },
            0x18..=0x1F => {
                // rr
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = self.decode_operand_u8_mut(rhs_value);
                Self::inst_rr(&mut self.f, rhs_value);
            },
            0x20..=0x27 => {
                // sla
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = self.decode_operand_u8_mut(rhs_value);
                Self::inst_sla(&mut self.f, rhs_value);
            },
            0x28..=0x2F => {
                // sra
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = self.decode_operand_u8_mut(rhs_value);
                Self::inst_sra(&mut self.f, rhs_value);
            },
            0x30..=0x37 => {
                // swap
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = self.decode_operand_u8_mut(rhs_value);
                Self::inst_swap(&mut self.f, rhs_value);
            },
            0x38..=0x3F => {
                // srl
                let rhs_value = opcode & 0b0000_0111;
                // let rhs_value = decode_operand_u8_mut!(self, rhs_value);
                let rhs_value = self.decode_operand_u8_mut(rhs_value);
                Self::inst_srl(&mut self.f, rhs_value);
            },
            0x40..=0x7F => {
                // bit
                // let reg = opcode & 
                // Self::inst_bit(&mut self.f, reg, test_bit);
            },
            0x80..=0xBF => {
                // res
                todo!();
            },
            0xC0..=0xFF => {
                // set
                todo!();
            },
        }
    }

    fn decode_operand_enum(n: u8) -> Register {
        match n {
            0b000 => Register::B,
            0b001 => Register::C,
            0b010 => Register::D,
            0b011 => Register::E,
            0b100 => Register::H,
            0b101 => Register::L,
            0b110 => Register::FromHL,
            0b111 => Register::A,
            _ => panic!("Cannot match operand for ALU_OP A, {n:b}"),
        }
    }

    fn decode_operand_enum2(&mut self, n: Register) -> &mut u8 {
        match n {
            Register::B => &mut self.b,
            Register::C => &mut self.c,
            Register::D => &mut self.d,
            Register::E => &mut self.e,
            Register::H => &mut self.h,
            Register::L => &mut self.l,
            Register::FromHL => &mut self.ram[Self::join_u8(self.h, self.l) as usize],
            Register::A => &mut self.a,
        }
    }

    // Operand disassembly tables
    // https://archive.gbdev.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
    fn decode_operand_u8(&self, n: u8) -> u8 {
        match n {
            0b000 => self.b,
            0b001 => self.c,
            0b010 => self.d,
            0b011 => self.e,
            0b100 => self.h,
            0b101 => self.l,
            0b110 => self.ram[Self::join_u8(self.h, self.l) as usize],
            0b111 => self.a,
            _ => panic!("Cannot match operand for ALU_OP A, {n:b}"),
        }
    }

    fn decode_operand_u8_mut(&mut self, n: u8) -> &mut u8 {
        match n {
            0b000 => &mut self.b,
            0b001 => &mut self.c,
            0b010 => &mut self.d,
            0b011 => &mut self.e,
            0b100 => &mut self.h,
            0b101 => &mut self.l,
            0b110 => &mut self.ram[Self::join_u8(self.h, self.l) as usize],
            0b111 => &mut self.a,
            _ => panic!("Cannot match DST operand for LD, {n:b}"),
        }
    }

    fn decode_operand_u16(&self, n: u8) -> u8 {
        todo!();
        match n {
            0b000 => self.b,
            0b001 => self.c,
            0b010 => self.d,
            0b011 => self.e,
            0b100 => self.h,
            0b101 => self.l,
            0b110 => self.ram[Self::join_u8(self.h, self.l) as usize],
            0b111 => self.a,
            _ => panic!("Cannot match operand for ALU_OP A, {n:b}"),
        }
    }

    fn decode_operand_u16_ref(&mut self, n: u8) -> &mut u8 {
        todo!();
        match n {
            0b000 => &mut self.b,
            0b001 => &mut self.c,
            0b010 => &mut self.d,
            0b011 => &mut self.e,
            0b100 => &mut self.h,
            0b101 => &mut self.l,
            0b110 => &mut self.ram[Self::join_u8(self.h, self.l) as usize],
            0b111 => &mut self.a,
            _ => panic!("Cannot match DST operand for LD, {n:b}"),
        }
    }

    fn inst_nop(&self) {}

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
        if carry0 || carry1 {
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

        // if any of the two subs underflow we should set carry
        // according to opcode table SBC A, A does not affect the carry flag, look this up later
        if carry0 || carry1 {
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

        if result == 0 {
            flags |= Flags::Zero;
        }

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

    fn inst_load(dst: &mut u8, src: u8) {
        *dst = src;
    }

    fn inst_inc_bc(&mut self) {
        let result = Self::join_u8(self.b, self.c).wrapping_add(1);

        (self.b, self.c) = Self::split_u16(result);
    }

    fn inst_inc_de(&mut self) {
        let result = Self::join_u8(self.d, self.e).wrapping_add(1);

        (self.d, self.e) = Self::split_u16(result);
    }

    fn inst_inc_hl(&mut self) {
        let result = Self::join_u8(self.h, self.l).wrapping_add(1);

        (self.h, self.l) = Self::split_u16(result);
    }

    fn inst_inc_sp(&mut self) {
        self.sp = self.sp.wrapping_add(1);
    }

    fn inst_dec_bc(&mut self) {
        let result = Self::join_u8(self.b, self.c).wrapping_sub(1);

        (self.b, self.c) = Self::split_u16(result);
    }

    fn inst_dec_de(&mut self) {
        let result = Self::join_u8(self.d, self.e).wrapping_sub(1);

        (self.d, self.e) = Self::split_u16(result);
    }

    fn inst_dec_hl(&mut self) {
        let result = Self::join_u8(self.h, self.l).wrapping_add(1);

        (self.h, self.l) = Self::split_u16(result);
    }

    fn inst_dec_sp(&mut self) {
        self.sp = self.sp.wrapping_sub(1);
    }

    fn inst_scf(&mut self) {
        let mut flags = 0u8;

        flags |= self.f & Flags::Zero;
        flags |= Flags::Carry;

        self.f = flags;
    }

    fn inst_daa(&mut self) {
        let mut flags = 0u8;
        let subtract = self.f & Flags::Subtraction;
        let half_carry = self.f & Flags::HalfCarry;
        let carry = self.f & Flags::Carry;

        let result: u8;
        let new_carry: bool;
        let mut adjustment = 0u8;
        if subtract > 0 {
            if half_carry > 0 {
                adjustment = adjustment.wrapping_add(0x6);
            }

            if carry > 0 {
                adjustment = adjustment.wrapping_add(0x60);
            }

            (result, new_carry) = self.a.borrowing_sub(adjustment, false);
        } else {
            if half_carry > 0 || ((self.a & 0xF) > 0x9) {
                adjustment = adjustment.wrapping_add(0x6);
            }

            if carry > 0 || self.a > 0x99 {
                adjustment = adjustment.wrapping_add(0x60);
            }

            (result, new_carry) = self.a.carrying_add(adjustment, false);
        }

        if result == 0 {
            flags |= Flags::Zero;
        }

        flags |= subtract;

        if new_carry {
            flags |= Flags::Carry;
        }

        self.a = result;
        self.f = flags;
    }

    fn inst_cpl(&mut self) {
        let mut flags = 0u8;

        flags |= self.f & Flags::Zero;
        flags |= Flags::Subtraction;
        flags |= Flags::HalfCarry;
        flags |= self.f & Flags::Carry;

        self.a = !self.a;
        self.f = flags;
    }

    fn inst_ccf(&mut self) {
        let mut flags = 0u8;

        flags |= self.f & Flags::Zero;

        if (self.f & Flags::Carry) == 0 {
            flags |= Flags::Carry
        }

        self.f = flags;
    }

    fn inst_load_bc(&mut self, value: u16) {
        (self.b, self.c) = Self::split_u16(value);
    }

    fn inst_load_de(&mut self, value: u16) {
        (self.d, self.e) = Self::split_u16(value);
    }

    fn inst_load_hl(&mut self, value: u16) {
        (self.h, self.l) = Self::split_u16(value);
    }

    fn inst_load_sp(&mut self, value: u16) {
        self.sp = value;
    }

    fn inst_load_to_memory(&mut self, offset: u16) {
        self.ram[offset as usize] = self.a;
    }

    fn inst_load_from_memory(&mut self, offset: u16) {
        self.a = self.ram[offset as usize];
    }

    fn inst_rrca(&mut self) {
        let mut flags = 0u8;
        let shifted_bit = self.a & 1;

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        self.a = (self.a >> 1) & (shifted_bit << 7);
        self.f = flags;
    }

    fn inst_rra(&mut self) {
        let mut flags = 0u8;
        let shifted_bit = self.a & 1;
        let old_carry = ((self.f & Flags::Carry) > 0) as u8 ;

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        self.a = (self.a >> 1) & (old_carry << 7);
        self.f = flags;
    }

    fn inst_rlc(flag_reg: &mut u8, reg: &mut u8) {
        let mut flags = 0u8;
        let shifted_bit = (*reg & 0b1000_0000) >> 7;
        let result = (*reg << 1) & (shifted_bit);

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        *reg = result;
        *flag_reg = flags;
    }

    fn inst_rrc(flag_reg: &mut u8, reg: &mut u8) {
        let mut flags = 0u8;
        let shifted_bit = *reg & 1;
        let result = (shifted_bit << 7) & (*reg >> 1);

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        *reg = result;
        *flag_reg = flags;
    }

    fn inst_rl(flag_reg: &mut u8, reg: &mut u8) {
        let mut flags = 0u8;
        let shifted_bit = (*reg & 0b1000_0000) >> 7;
        let old_carry = ((*flag_reg & Flags::Carry) > 0) as u8;
        let result = (*reg << 1) & old_carry;

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        *reg = result;
        *flag_reg = flags;
    }

    fn inst_rr(flag_reg: &mut u8, reg: &mut u8) {
        let mut flags = 0u8;
        let shifted_bit = *reg & 1;
        let old_carry = ((*flag_reg & Flags::Carry) > 0) as u8;
        let result = (old_carry << 7) & (*reg >> 1);

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        *reg = result;
        *flag_reg = flags;
    }

    fn inst_sla(flag_reg: &mut u8, reg: &mut u8) {
        let mut flags = 0u8;
        let shifted_bit = (*reg & 0b1000_0000) >> 7;
        let result = *reg << 1;

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        *reg = result;
        *flag_reg = flags;
    }

    fn inst_sra(flag_reg: &mut u8, reg: &mut u8) {
        let mut flags = 0u8;
        let shifted_bit = *reg & 1;
        let bit_7 = *reg & 0b1000_0000;
        let result = (*reg >> 1) & bit_7; // keep most significant bit

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        *reg = result;
        *flag_reg = flags;
    }

    fn inst_swap(flag_reg: &mut u8, reg: &mut u8) {
        let mut flags = 0u8;
        let upper = (*reg & 0xF0) >> 4;
        let lower = *reg & 0x0F;
        let result = (lower << 4) & upper;

        if result == 0 {
            flags |= Flags::Zero;
        }

        *reg = result;
        *flag_reg = flags;
    }

    fn inst_srl(flag_reg: &mut u8, reg: &mut u8) {
        let mut flags = 0u8;
        let shifted_bit = *reg & 1;
        let result = *reg >> 1;

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        *reg = result;
        *flag_reg = flags;
    }

    fn inst_bit(&mut self, regN: Register, test_bit: u8) {
        let mut flags = 0u8;

        let reg = self.decode_operand_enum2(regN);
        // todo: check if logic is right
        if ((*reg) & (1 << test_bit)) > 0 {
            flags |= Flags::Zero;
        }

        flags |= Flags::HalfCarry;

        self.f = flags;
    }
}

#[repr(u8)]
#[derive(Debug)]
enum Flags {
    Zero = 0b1000_0000,
    Subtraction = 0b0100_0000,
    HalfCarry = 0b0010_0000,
    Carry = 0b0001_0000,
}

enum Register {
    B,
    C,
    D,
    E,
    H,
    L,
    FromHL,
    A,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_u8() {
        assert_eq!(CPU::join_u8(0xAB, 0xCD), 0xABCD);
    }

    #[test]
    fn test_split_u16() {
        assert_eq!(CPU::split_u16(0xABCD), (0xAB, 0xCD));
    }

    #[test]
    fn test_inst_add() {
        let mut cpu: CPU = Default::default();

        cpu.a = 0x0A;
        cpu.inst_add(0x05);

        assert_eq!(cpu.a, 0x0Au8.wrapping_add(0x05));
        assert_eq!(cpu.f, 0);
    }

    #[test]
    fn test_inst_add_set_carry_flag() {
        let mut cpu: CPU = Default::default();

        cpu.a = 0xFE;
        cpu.inst_add(0x3B);

        assert_eq!(cpu.a, 0xFEu8.wrapping_add(0x3B));
        assert_eq!(cpu.f & Flags::Carry, Flags::Carry as u8);
    }

    #[test]
    fn test_inst_add_set_half_carry_flag() {
        let mut cpu: CPU = Default::default();

        cpu.a = 0x4F;
        cpu.inst_add(0x13);

        assert_eq!(cpu.a, 0x4Fu8.wrapping_add(0x13));
        assert_eq!(cpu.f & Flags::HalfCarry, Flags::HalfCarry as u8);
    }

    #[test]
    fn test_inst_add_set_zero_flag() {
        let mut cpu: CPU = Default::default();

        cpu.a = 0xFF;
        cpu.inst_add(0x1);

        assert_eq!(cpu.a, 0xFFu8.wrapping_add(0x1));
        assert_eq!(cpu.f & Flags::Zero, Flags::Zero as u8);
    }
}
