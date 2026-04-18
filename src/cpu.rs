#[repr(u8)]
#[derive(Debug)]
enum Flags {
    Zero = 0b1000_0000,
    Subtraction = 0b0100_0000,
    HalfCarry = 0b0010_0000,
    Carry = 0b0001_0000,
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

#[derive(Clone, Copy, PartialEq)]
enum Register {
    B,
    C,
    D,
    E,
    H,
    L,
    FromHL, // address pointed by HL
    A,
}

#[derive(Clone, Copy)]
enum Register16 {
    BC,
    DE,
    HL,
    AF,
    SP,
}

#[derive(Clone, Copy)]
enum Address {
    BC,
    DE,
    HL,
    HLI,
    HLD,
    Immediate,
}

#[derive(Clone, Copy)]
struct Immediate;

impl PartialEq<Register> for Immediate {
    fn eq(&self, _: &Register) -> bool {
        false
    }
}

enum Condition {
    Z,
    NZ,
    C,
    NC,
}

trait Read<T: Copy> {
    fn read(&mut self, target: T) -> u8;
}

trait ReadU16<T: Copy> {
    fn read_u16(&mut self, target: T) -> u16;
}

trait Write<T: Copy> {
    fn write(&mut self, target: T, data: u8);
}

trait WriteU16<T: Copy> {
    fn write_u16(&mut self, target: T, data: u16);
}

trait Eval {
    fn eval(&self, condtion: Condition) -> bool;
}

#[derive(Debug)]
pub struct Cpu {
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

impl Default for Cpu {
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

impl Read<Register> for Cpu {
    fn read(&mut self, target: Register) -> u8 {
        match target {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::FromHL => self.ram[u16::from_le_bytes([self.l, self.h]) as usize],
            Register::A => self.a,
        }
    }
}

impl ReadU16<Register16> for Cpu {
    fn read_u16(&mut self, target: Register16) -> u16 {
        match target {
            Register16::BC => u16::from_le_bytes([self.c, self.b]),
            Register16::DE => u16::from_le_bytes([self.e, self.d]),
            Register16::HL => u16::from_le_bytes([self.l, self.h]),
            Register16::AF => u16::from_le_bytes([self.f, self.a]),
            Register16::SP => self.sp,
        }
    }
}

impl Read<Address> for Cpu {
    fn read(&mut self, target: Address) -> u8 {
        match target {
            Address::BC => self.ram[u16::from_le_bytes([self.c, self.b]) as usize],
            Address::DE => self.ram[u16::from_le_bytes([self.e, self.d]) as usize],
            Address::HL => self.ram[u16::from_le_bytes([self.l, self.h]) as usize],
            Address::HLI => {
                let offset = u16::from_le_bytes([self.l, self.h]);
                let value = self.ram[offset as usize];
                let offset = offset.wrapping_add(1);
                [self.l, self.h] = u16::to_le_bytes(offset);
                value
            }
            Address::HLD => {
                let offset = u16::from_le_bytes([self.l, self.h]);
                let value = self.ram[offset as usize];
                let offset = offset.wrapping_sub(1);
                [self.l, self.h] = u16::to_le_bytes(offset);
                value
            },
            Address::Immediate => {
                let offset = self.fetch_u16();
                self.ram[offset as usize]
            }
        }
    }
}

impl Read<Immediate> for Cpu {
    fn read(&mut self, _: Immediate) -> u8 {
        self.fetch()
    }
}

impl ReadU16<Immediate> for Cpu {
    fn read_u16(&mut self, _: Immediate) -> u16 {
        self.fetch_u16()
    }
}

impl Write<Register> for Cpu {
    fn write(&mut self, target: Register, value: u8) {
        match target {
            Register::B => self.b = value,
            Register::C => self.c = value,
            Register::D => self.d = value,
            Register::E => self.e = value,
            Register::H => self.h = value,
            Register::L => self.l = value,
            Register::FromHL => self.ram[u16::from_le_bytes([self.l, self.h]) as usize] = value,
            Register::A => self.a = value,
        }
    }
}

impl WriteU16<Register16> for Cpu {
    fn write_u16(&mut self, target: Register16, value: u16) {
        match target {
            Register16::BC => [self.c, self.b] = u16::to_le_bytes(value),
            Register16::DE => [self.e, self.d] = u16::to_le_bytes(value),
            Register16::HL => [self.l, self.h] = u16::to_le_bytes(value),
            Register16::AF => [self.f, self.a] = u16::to_le_bytes(value),
            Register16::SP => self.sp = value,
        }
    }
}

impl Write<Address> for Cpu {
    fn write(&mut self, target: Address, value: u8) {
        match target {
            Address::BC => self.ram[u16::from_le_bytes([self.c, self.b]) as usize] = value,
            Address::DE => self.ram[u16::from_le_bytes([self.e, self.d]) as usize] = value,
            Address::HL => self.ram[u16::from_le_bytes([self.l, self.h]) as usize] = value,
            Address::HLI => {
                let offset = u16::from_le_bytes([self.l, self.h]);
                self.ram[offset as usize] = value;
                let offset = offset.wrapping_add(1);
                [self.l, self.h] = u16::to_le_bytes(offset);
            }
            Address::HLD => {
                let offset = u16::from_le_bytes([self.l, self.h]);
                self.ram[offset as usize] = value;
                let offset = offset.wrapping_sub(1);
                [self.l, self.h] = u16::to_le_bytes(offset);
            },
            Address::Immediate => {
                let offset = self.fetch_u16();
                self.ram[offset as usize] = value;
            }
        }
    }
}

impl Eval for Cpu {
    fn eval(&self, condition: Condition) -> bool {
        match condition {
            Condition::Z => (self.f & Flags::Zero) != 0,
            Condition::NZ => (self.f & Flags::Zero) == 0,
            Condition::C => (self.f & Flags::Carry) != 0,
            Condition::NC => (self.f & Flags::Carry) == 0,
        }
    }
}

impl Cpu {
    pub fn load_rom(&mut self, rom: &[u8], pos: usize) {
        self.ram[pos..rom.len()].copy_from_slice(rom);
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

        u16::from_le_bytes([low, high])
    }

    pub fn decode(&mut self, opcode: u8) {
        match opcode {
            0x00 => self.inst_nop(), // nop
            0x10 => {self.fetch();}, // stop
            0x76 => todo!(), // halt
            0xCB => {
                // CB prefixed instuctions
                let new_opcode = self.fetch();
                self.decode_prefixed_cb(new_opcode);
            },
            // relative jumps
            0x20 => self.inst_jr(Some(Condition::NZ)),
            0x30 => self.inst_jr(Some(Condition::NC)),
            0x18 => self.inst_jr(None),
            0x28 => self.inst_jr(Some(Condition::Z)),
            0x38 => self.inst_jr(Some(Condition::C)),
            // 8-bit loads
            0x02 => self.inst_ld(Address::BC, Register::A),
            0x12 => self.inst_ld(Address::DE, Register::A),
            0x22 => self.inst_ld(Address::HLI, Register::A),
            0x32 => self.inst_ld(Address::HLD, Register::A),
            0x0A => self.inst_ld(Register::A, Address::BC),
            0x1A => self.inst_ld(Register::A, Address::DE),
            0x2A => self.inst_ld(Register::A, Address::HLI),
            0x3A => self.inst_ld(Register::A, Address::HLD),
            0xEA => self.inst_ld(Address::Immediate, Register::A),
            0xFA => self.inst_ld(Register::A, Address::Immediate),
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                let dst = Self::extract_operand((opcode & 0b0011_1000) >> 3);
                self.inst_ld(dst, Immediate);
            },
            0x27 => self.inst_daa(),
            0x37 => self.inst_scf(),
            // 16-bit register increment
            0x03 => self.inst_inc_u16(Register16::BC),
            0x13 => self.inst_inc_u16(Register16::DE),
            0x23 => self.inst_inc_u16(Register16::HL),
            0x33 => self.inst_inc_u16(Register16::SP),
            // 16-bit register decrement
            0x0B => self.inst_dec_u16(Register16::BC),
            0x1B => self.inst_dec_u16(Register16::DE),
            0x2B => self.inst_dec_u16(Register16::HL),
            0x3B => self.inst_dec_u16(Register16::SP),
            0x2F => self.inst_cpl(),
            0x3F => self.inst_ccf(),
            // ALU operations with 8-bit immediates
            0xC6 => self.inst_add(Immediate),
            0xD6 => self.inst_sub(Immediate),
            0xE6 => self.inst_and(Immediate),
            0xF6 => self.inst_or(Immediate),
            0xCE => self.inst_adc(Immediate),
            0xDE => self.inst_sbc(Immediate),
            0xEE => self.inst_xor(Immediate),
            0xFE => self.inst_cp(Immediate),
            0x0F => self.inst_rrca(),
            0x1F => self.inst_rra(),
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB..=0xED | 0xF4 | 0xFC | 0xFD => println!("Unused opcode: {opcode:#04X}"),
            // 16-bit loads
            0x01 => self.inst_ld_u16(Register16::BC, Immediate),
            0x11 => self.inst_ld_u16(Register16::DE, Immediate),
            0x21 => self.inst_ld_u16(Register16::HL, Immediate),
            0x31 => self.inst_ld_u16(Register16::SP, Immediate),
            0x40..=0x7F => {
                // reg to reg 8-bit loads
                let dst = (opcode & 0b00111000) >> 3;
                let src = opcode & 0b00000111;
                let (dst, src) = (Self::extract_operand(dst), Self::extract_operand(src));
                self.inst_ld(dst, src);
            },
            0x80..=0x87 => {
                // add
                let rhs_value = opcode & 0b00000111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_add(rhs_value);
            },
            0x88..=0x8F => {
                // add with carry
                let rhs_value = opcode & 0b00000111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_adc(rhs_value);
            },
            0x90..=0x97 => {
                // sub
                let rhs_value = opcode & 0b00000111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_sub(rhs_value);
            },
            0x98..=0x9F => {
                // sub with carry
                let rhs_value = opcode & 0b00000111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_sbc(rhs_value);
            },
            0xA0..=0xA7 => {
                // and
                let rhs_value = opcode & 0b00000111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_and(rhs_value);
            },
            0xA8..=0xAF => {
                // xor
                let rhs_value = opcode & 0b00000111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_xor(rhs_value);
            },
            0xB0..=0xB7 => {
                // or
                let rhs_value = opcode & 0b00000111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_or(rhs_value);
            },
            0xB8..=0xBF => {
                // compare
                let rhs_value = opcode & 0b00000111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_cp(rhs_value);
            },
            _ => todo!()
        }
    }

    // Decode opcodes prefixed with $CB
    fn decode_prefixed_cb(&mut self, opcode: u8) {
        match opcode {
            0x00..=0x07 => {
                // rlc
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_rlc(rhs_value);
            },
            0x08..=0x0F => {
                // rrc
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_rrc(rhs_value);
            },
            0x10..=0x17 => {
                // rl
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_rl(rhs_value);
            },
            0x18..=0x1F => {
                // rr
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_rr(rhs_value);
            },
            0x20..=0x27 => {
                // sla
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_sla(rhs_value);
            },
            0x28..=0x2F => {
                // sra
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_sra(rhs_value);
            },
            0x30..=0x37 => {
                // swap
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_swap(rhs_value);
            },
            0x38..=0x3F => {
                // srl
                let rhs_value = opcode & 0b0000_0111;
                let rhs_value = Self::extract_operand(rhs_value);
                self.inst_srl(rhs_value);
            },
            0x40..=0x7F => {
                // bit
                let register = Self::extract_operand(opcode & 0b0000_0111);
                let test_bit = (opcode & 0b0011_1000) >> 3;
                self.inst_bit(register, test_bit);
            },
            0x80..=0xBF => {
                // res
                let register = Self::extract_operand(opcode & 0b0000_0111);
                let test_bit = (opcode & 0b0011_1000) >> 3;
                self.inst_res(register, test_bit);
            },
            0xC0..=0xFF => {
                // set
                let register = Self::extract_operand(opcode & 0b0000_0111);
                let test_bit = (opcode & 0b0011_1000) >> 3;
                self.inst_set(register, test_bit);
            },
        }
    }

    // Operand disassembly tables
    // https://archive.gbdev.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
    fn extract_operand(n: u8) -> Register {
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

    fn inst_nop(&self) {}

    fn inst_add<T: Copy>(&mut self, operand: T) where Self: Read<T> {
        let mut flags = 0u8;

        let value = self.read(operand);
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

    fn inst_adc<T: Copy>(&mut self, operand: T) where Self: Read<T> {
        let mut flags = 0u8;

        let value = self.read(operand);
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

    fn inst_sub<T: Copy>(&mut self, operand: T) where Self: Read<T> {
        let mut flags = 0u8;

        let value = self.read(operand);
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

    fn inst_sbc<T: Copy + PartialEq<Register>>(&mut self, operand: T) where Self: Read<T> {
        let mut flags = 0u8;

        let value = self.read(operand);
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
        // "SBC A, A" does not affect the carry flag
        if operand != Register::A && (carry0 || carry1) {
            flags |= Flags::Carry;
        }

        self.a = result;
        self.f = flags;
    }

    fn inst_and<T: Copy>(&mut self, operand: T) where Self: Read<T> {
        let mut flags = 0u8;

        let value = self.read(operand);
        let result = self.a & value;

        if result == 0 {
            flags |= Flags::Zero;
        }

        flags |= Flags::HalfCarry;

        self.a = result;
        self.f = flags;
    }

    fn inst_xor<T: Copy>(&mut self, operand: T) where Self: Read<T> {
        let mut flags = 0u8;

        let value = self.read(operand);
        let result = self.a ^ value;

        if result == 0 {
            flags |= Flags::Zero;
        }

        self.a = result;
        self.f = flags;
    }

    fn inst_or<T: Copy>(&mut self, operand: T) where Self: Read<T> {
        let mut flags = 0u8;

        let value = self.read(operand);
        let result = self.a | value;

        if result == 0 {
            flags |= Flags::Zero;
        }

        self.a = result;
        self.f = flags;
    }

    fn inst_cp<T: Copy>(&mut self, operand: T) where Self: Read<T> {
        let mut flags = 0u8;

        let value = self.read(operand);
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

    fn inst_ld<T: Copy, U: Copy>(&mut self, dst: T, src: U) where Self: Write<T> + Read<U> {
        let value = self.read(src);
        self.write(dst, value);
    }

    fn inst_ld_u16<T: Copy, U: Copy>(&mut self, dst: T, src: U) where Self: WriteU16<T> + ReadU16<U> {
        let value = self.read_u16(src);
        self.write_u16(dst, value);
    }

    fn inst_inc_u16<T: Copy>(&mut self, target: T) where Self: WriteU16<T> + ReadU16<T> {
        let value = self.read_u16(target).wrapping_add(1);
        self.write_u16(target, value);
    }

    fn inst_dec_u16<T: Copy>(&mut self, target: T) where Self: WriteU16<T> + ReadU16<T> {
        let value = self.read_u16(target).wrapping_sub(1);
        self.write_u16(target, value);
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

    // fn inst_load_bc(&mut self, value: u16) {
    //     (self.b, self.c) = split_u16(value);
    // }

    // fn inst_load_de(&mut self, value: u16) {
    //     (self.d, self.e) = split_u16(value);
    // }

    // fn inst_load_hl(&mut self, value: u16) {
    //     (self.h, self.l) = split_u16(value);
    // }

    // fn inst_load_sp(&mut self, value: u16) {
    //     self.sp = value;
    // }

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
        let old_carry = ((self.f & Flags::Carry) > 0) as u8;

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        self.a = (self.a >> 1) & (old_carry << 7);
        self.f = flags;
    }

    fn inst_rlc(&mut self, reg: Register) {
        let mut flags = 0u8;

        let old_value = self.read(reg);
        let shifted_bit = (old_value & 0b1000_0000) >> 7;
        let result = (old_value << 1) & (shifted_bit);

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        self.write(reg, result);
        self.f = flags;
    }

    fn inst_rrc(&mut self, reg: Register) {
        let mut flags = 0u8;

        let old_value = self.read(reg);
        let shifted_bit = old_value & 1;
        let result = (shifted_bit << 7) & (old_value >> 1);

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        self.write(reg, result);
        self.f = flags;
    }

    fn inst_rl(&mut self, reg: Register) {
        let mut flags = 0u8;

        let old_value = self.read(reg);
        let shifted_bit = (old_value & 0b1000_0000) >> 7;
        let old_carry = ((self.f & Flags::Carry) > 0) as u8;
        let result = (old_value << 1) & old_carry;

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        self.write(reg, result);
        self.f = flags;
    }

    fn inst_rr(&mut self, reg: Register) {
        let mut flags = 0u8;

        let old_value = self.read(reg);
        let shifted_bit = old_value & 1;
        let old_carry = ((self.f & Flags::Carry) > 0) as u8;
        let result = (old_carry << 7) & (old_value >> 1);

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        self.write(reg, result);
        self.f = flags;
    }

    fn inst_sla(&mut self, reg: Register) {
        let mut flags = 0u8;

        let old_value = self.read(reg);
        let shifted_bit = (old_value & 0b1000_0000) >> 7;
        let result = old_value << 1;

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        self.write(reg, result);
        self.f = flags;
    }

    fn inst_sra(&mut self, reg: Register) {
        let mut flags = 0u8;

        let old_value = self.read(reg);
        let shifted_bit = old_value & 1;
        let bit_7 = old_value & 0b1000_0000;
        let result = (old_value >> 1) & bit_7; // keep most significant bit

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        self.write(reg, result);
        self.f = flags;
    }

    fn inst_swap(&mut self, reg: Register) {
        let mut flags = 0u8;

        let old_value = self.read(reg);
        let upper = (old_value & 0xF0) >> 4;
        let lower = old_value & 0x0F;
        let result = (lower << 4) & upper;

        if result == 0 {
            flags |= Flags::Zero;
        }

        self.write(reg, result);
        self.f = flags;
    }

    fn inst_srl(&mut self, reg: Register) {
        let mut flags = 0u8;

        let old_value = self.read(reg);
        let shifted_bit = old_value & 1;
        let result = old_value >> 1;

        if result == 0 {
            flags |= Flags::Zero;
        }

        if shifted_bit == 1 {
            flags |= Flags::Carry;
        }

        self.write(reg, result);
        self.f = flags;
    }

    fn inst_bit(&mut self, reg: Register, test_bit: u8) {
        // keep carry flag
        let mut flags = self.f & Flags::Carry;

        let value = self.read(reg);
        if (value & (1 << test_bit)) == 0 {
            flags |= Flags::Zero;
        }

        flags |= Flags::HalfCarry;

        self.f = flags;
    }

    fn inst_res(&mut self, reg: Register, test_bit: u8) {
        let value = self.read(reg);
        let result = value & !(1 << test_bit);
        self.write(reg, result);
    }

    fn inst_set(&mut self, reg: Register, test_bit: u8) {
        let value = self.read(reg);
        let result = value | (1 << test_bit);
        self.write(reg, result);
    }

    fn inst_jr(&mut self, condition_code: Option<Condition>) {
        let offset = self.fetch() as i8; // signed offset
        match condition_code {
            Some(cond) => {
                if self.eval(cond) {
                    // hacky solution, TODO fix this
                    self.pc = (self.pc as i16).wrapping_add(offset as i16) as u16;
                }
            },
            None => self.pc = (self.pc as i16).wrapping_add(offset as i16) as u16,
        }
    }
}
