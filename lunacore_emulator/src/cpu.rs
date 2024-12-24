use crate::components::*;

pub struct CPU {
    pub regs: RegFile,
    pub imem: WordROM,
    pub dmem: ByteRAM,
    pub cond_unit: CondUnit,
    pub pc: u16,

    // datapath
    pub alu_flags: Flags,
    pub instr: [u16; 2],
    pub wide: bool,
    pub next_wide: bool,
    pub pc_overwritten: bool,

    pub run: bool,
    pub debug: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            regs: RegFile::new(),
            imem: WordROM::new(),
            dmem: ByteRAM::new(),
            cond_unit: CondUnit::new(),
            pc: 0,

            alu_flags: Flags::new(),
            instr: [0, 0],
            wide: false,
            next_wide: false,
            pc_overwritten: false,

            run: true,
            debug: false,
        }
    }

    pub fn debug_state(&self) {
        println!("");
        println!("{:?}", self.regs);
        println!("{:?}", self.cond_unit.flags);
        println!();
    }

    pub fn debug_instruction(&self) {
        let op = get_bits(self.instr[0], 15, 14);

        let instr_str = match op {
            // DP
            0b00 => {
                let imm = get_bits(self.instr[0], 13, 12);
                let cmd = get_bits(self.instr[0], 11, 9);
                let td = get_bits(self.instr[0], 8, 6);
                let tn = get_bits(self.instr[0], 5, 3);
                let src2 = get_bits(self.instr[0], 2, 0);


                let src2_string = match imm {
                    0b00 => reg_to_string(src2),
                    0b01 => format!("!{}", src2),
                    0b10 => format!("!{}", imm_extend(src2, 3, 1) as i16),
                    0b11 => format!("!0x{:04x}", self.instr[1]),
                    _ => panic!(),
                };

                match cmd {
                    0b000 => format!("ADD  {}, {}, {}", reg_to_string(td), reg_to_string(tn), src2_string),
                    0b001 => format!("SUB  {}, {}, {}", reg_to_string(td), reg_to_string(tn), src2_string),
                    0b010 => format!("AND  {}, {}, {}", reg_to_string(td), reg_to_string(tn), src2_string),
                    0b011 => format!("OR   {}, {}, {} ", reg_to_string(td), reg_to_string(tn), src2_string),
                    0b100 => format!("XOR  {}, {}, {}", reg_to_string(td), reg_to_string(tn), src2_string),
                    0b101 => format!("MOV  {}, {}", reg_to_string(td), src2_string),
                    0b110 => format!("SHL  {}, {}, {}", reg_to_string(td), reg_to_string(tn), src2_string),
                    0b111 => format!("SHR  {}, {}, {}", reg_to_string(td), reg_to_string(tn), src2_string),
                    _ => panic!(),
                }
            }
            // MEM
            0b01 => {
                let imm = get_bits(self.instr[0], 13, 12);
                let b = get_bit(self.instr[0], 11);
                let sl = get_bits(self.instr[0], 10, 9);
                let td = get_bits(self.instr[0], 8, 6);
                let tn = get_bits(self.instr[0], 5, 3);
                let src2 = get_bits(self.instr[0], 2, 0);

                let src2_string = match imm {
                    0b00 => reg_to_string(src2) + "]",
                    0b01 => format!("{}]", src2),
                    0b10 => format!("{}]", imm_extend(src2, 3, 1) as i16),
                    0b11 => format!("0x{:04x}]", self.instr[1]),
                    _ => panic!(),
                };

                let push_src = match imm {
                    0b00 => reg_to_string(td) + "",
                    0b01 => format!("{}", src2),
                    0b10 => format!("{}", imm_extend(src2, 3, 1) as i16),
                    0b11 => format!("0x{:04x}", self.instr[1]),
                    _ => panic!(),
                };

                let b_str = match b {
                    0 => " ",
                    1 => "B",
                    _ => panic!(),
                };

                match sl {
                    0b00 => format!("SAV{} {}, [{} + {}", b_str, reg_to_string(td), reg_to_string(tn), src2_string),
                    0b01 => format!("LOD{} {}, [{} + {}", b_str, reg_to_string(td), reg_to_string(tn), src2_string),
                    0b10 => format!("PUSH{} {}", b_str, push_src),
                    0b11 => format!("POP{} {}             ", b_str, reg_to_string(td)),
                    _ => panic!(),
                }
            }
            // BRANCH
            0b10 => {
                let w = get_bit(self.instr[0], 13);
                let cond = get_bits(self.instr[0], 12, 9);
                let offset = get_bits(self.instr[0], 8, 0);

                let offset_string = match w {
                    0 => format!("{}        => [0x{:04x}]", sign_extend(offset, 9) as i16, self.regs.pc + sign_extend(offset, 9)),
                    1 => format!("0x{:04x}    => [0x{:04x}]", self.instr[1], self.regs.pc + self.instr[1]),
                    _ => panic!(),
                };

                let cond_str = match cond {
                    0b0000 => "Z  ",
                    0b0001 => "NZ ",
                    0b0010 => "LT ",
                    0b0011 => "LE ",
                    0b0100 => "GT ",
                    0b0101 => "GE ",
                    0b0110 => "ULT",
                    0b0111 => "ULE",
                    0b1000 => "UGT",
                    0b1001 => "UGE",
                    0b1010 => "MI ",
                    0b1011 => "PL ",
                    0b1100 => "VS ",
                    0b1101 => "VC ",
                    0b1110 => "MP ",
                    0b1111 => "NV ",
                    _ => panic!(),
                };

                format!("J{} {}", cond_str, offset_string)
            }
            _ => panic!(),
        };

        println!("PC=0x{:04x}: {}", self.pc, instr_str);
    }

    pub fn fetch(&mut self) {
        self.instr = self.imem.read(self.pc);
    }

    pub fn decode(&mut self) {
        // wide and next_wide logic
        self.wide = self.is_wide(self.instr[0]);
        self.next_wide = self.is_wide(self.instr[1]) && !self.wide;
        if self.debug && self.wide {
            print!("W ");
        }

        // reg file pc
        self.regs.pc = self.pc + 2 + (self.wide || self.next_wide) as u16;
    }

    pub fn execute(&mut self) {
        let op = get_bits(self.instr[0], 15, 14);

        match op {
            0b00 => self.dp(),
            0b01 => {
                let sl = get_bits(self.instr[0], 10, 9);
                match sl {
                    0b00 => self.sav(),
                    0b01 => self.lod(),
                    0b10 => self.push(),
                    0b11 => self.pop(),
                    _ => panic!("Op=11 instructions not implemented yet!"),
                }
            }
            0b10 => self.branch(),
            _ => panic!(),
        }
    }

    pub fn next_cycle(&mut self) {
        // advance pc and reset booleans
        if !self.pc_overwritten {
            self.pc += self.wide as u16 + 1;
        }
        self.wide = false;
        self.next_wide = false;
        self.pc_overwritten = false;
        self.alu_flags = Flags::new();
    }

    fn is_wide(&self, instr: u16) -> bool {
        let instr_op = get_bits(instr, 15, 14);
        match instr_op {
            0b00..=0b01 => {
                let next_imm = get_bits(instr, 13, 12);
                next_imm == 0b11
            }
            0b10 => {
                let w = get_bit(instr, 13);
                w == 1
            }
            _ => false,
        }
    }

    fn alu(&mut self, a: u16, b: u16, aluop: u16) -> u16 {
        let result: u32 = match aluop {
            0b000 => a as u32 + b as u32,        // Addition
            0b001 => a as u32 + (!b as u32 + 1), // Subtraction: a - b = a + (~b + 1)
            0b010 => a as u32 & b as u32,        // AND
            0b011 => a as u32 | b as u32,        // OR
            0b100 => a as u32 ^ b as u32,        // XOR
            0b101 => b as u32,                   // MOV
            0b110 => (a as u32) << (b & 15),     // Logical shift left
            0b111 => (a as u32) >> (b & 15),     // Logical shift right
            _ => panic!(),
        };

        self.alu_flags.n = ((result as u16) >> 15) == 1; // Negative flag: MSB of result
        self.alu_flags.z = (result as u16) == 0; // Zero flag: result is zero
        self.alu_flags.c = match aluop {
            0b000 => result > 0xFFFF,                          // Carry on addition
            0b001 => a < b,                                    // Borrow (carry) on subtraction
            0b101 => (a as u32) & (1 << (16 - (b & 15))) != 0, // Carry out on left shift
            _ => false,                                        // Carry is irrelevant for other ops
        };
        self.alu_flags.v = match aluop {
            0b000 => {
                let sign_a = (a >> 15) & 1;
                let sign_b = (b >> 15) & 1;
                let sign_result = ((result as u16) >> 15) & 1;
                sign_a == sign_b && sign_result != sign_a // Same sign for a and b, result has a different sign
            }
            0b001 => {
                let sign_a = (a >> 15) & 1;
                let sign_b = (b >> 15) & 1;
                let sign_result = ((result as u16) >> 15) & 1;
                sign_a != sign_b && sign_result != sign_a // Opposite sign for a and b, result has a different sign than a
            }
            _ => false, // Overflow is irrelevant for other ops
        };

        result as u16
    }

    fn dp(&mut self) {
        let imm = get_bits(self.instr[0], 13, 12);
        let cmd = get_bits(self.instr[0], 11, 9);

        let td = get_bits(self.instr[0], 8, 6);
        let tn = get_bits(self.instr[0], 5, 3);
        let src2 = get_bits(self.instr[0], 2, 0);

        let src_a = self.regs.read(tn);
        let src_b = match imm {
            0b00 => self.regs.read(src2),
            0b01 => imm_extend(src2, 3, 0),
            0b10 => imm_extend(src2, 3, 1),
            0b11 => self.instr[1],
            _ => panic!(),
        };

        let result = self.alu(src_a, src_b, cmd);

        // write flags and results
        self.cond_unit.write_flags(&self.alu_flags);
        self.regs.write(td, result);

        if td == 0b110 {
            self.pc = result;
            self.pc_overwritten = true;

            if self.debug {
                print!("DP Result stored in PC ");
            }
        }
    }

    fn sav(&mut self) {
        let imm = get_bits(self.instr[0], 13, 12);
        let b = get_bit(self.instr[0], 11);

        let td = get_bits(self.instr[0], 8, 6);
        let tn = get_bits(self.instr[0], 5, 3);
        let src2 = get_bits(self.instr[0], 2, 0);

        let src_a = self.regs.read(tn);
        let src_b = match imm {
            0b00 => self.regs.read(src2),
            0b01 => imm_extend(src2, 3, 0),
            0b10 => imm_extend(src2, 3, 1),
            0b11 => self.instr[1],
            _ => panic!(),
        };

        let result = self.alu(src_a, src_b, 0b000);
        let addr = result;

        let write_data = self.regs.read(td);
        self.dmem.write(addr, write_data, b);
    }

    fn lod(&mut self) {
        let imm = get_bits(self.instr[0], 13, 12);
        let b = get_bit(self.instr[0], 11);

        let td = get_bits(self.instr[0], 8, 6);
        let tn = get_bits(self.instr[0], 5, 3);
        let src2 = get_bits(self.instr[0], 2, 0);

        let src_a = self.regs.read(tn);
        let src_b = match imm {
            0b00 => self.regs.read(src2),
            0b01 => imm_extend(src2, 3, 0),
            0b10 => imm_extend(src2, 3, 1),
            0b11 => self.instr[1],
            _ => panic!(),
        };

        let result = self.alu(src_a, src_b, 0b000);
        let addr = result;

        let read_data = self.dmem.read(addr, b);
        self.regs.write(td, read_data);

        if td == 0b110 {
            self.pc = read_data;
            self.pc_overwritten = true;

            if self.debug {
                print!("LOD Result stored in PC ");
            }
        }
    }

    fn push(&mut self) {
        let imm = get_bits(self.instr[0], 13, 12);
        let b = get_bit(self.instr[0], 11);

        let td = get_bits(self.instr[0], 8, 6);
        let tn = get_bits(self.instr[0], 5, 3);
        assert_eq!(tn, 0b101); // rn = sp
        let src2 = get_bits(self.instr[0], 2, 0);

        let sp = self.regs.read(tn);
        let src_b: u16 = match b {
            0 => 2,
            1 => 1,
            _ => panic!(),
        };

        let result = self.alu(sp, src_b, 0b001);
        let addr = result;
        let new_sp = result;

        let write_data = match imm {
            0b00 => self.regs.read(td),
            0b01 => imm_extend(src2, 3, 0),
            0b10 => imm_extend(src2, 3, 1),
            0b11 => self.instr[1],
            _ => panic!(),
        };
        self.dmem.write(addr, write_data, b);
        self.regs.write(tn, new_sp);
    }

    fn pop(&mut self) {
        let b = get_bit(self.instr[0], 11);

        let td = get_bits(self.instr[0], 8, 6);
        let tn = get_bits(self.instr[0], 5, 3);
        assert_eq!(tn, 0b101); // rn = sp
        let _src2 = get_bits(self.instr[0], 2, 0); // src2 not used in pop

        let sp = self.regs.read(tn);
        let src_b: u16 = match b {
            0 => 2,
            1 => 1,
            _ => panic!(),
        };

        let result = self.alu(sp, src_b, 0b000);
        let addr = sp;
        let new_sp = result;

        let read_data = self.dmem.read(addr, b);
        self.regs.write(td, read_data);
        self.regs.write(tn, new_sp);

        if td == 0b110 {
            self.pc = read_data;
            self.pc_overwritten = true;

            if self.debug {
                print!("POP Result stored in PC ");
            }
        }
    }

    fn branch(&mut self) {
        let cond = get_bits(self.instr[0], 12, 9);
        let imm9 = get_bits(self.instr[0], 8, 0);

        let offset = match self.wide {
            true => self.instr[1],
            false => sign_extend(imm9, 9),
        };

        // do jmp if check = true
        if self.cond_unit.check(cond) {
            let result = self.alu(self.regs.pc, offset, 0b000);
            self.pc = result;
            self.pc_overwritten = true;
        }

        if self.debug {
            match self.pc_overwritten {
                true => print!("Branch Taken to 0x{:04x} ", self.pc),
                false => print!("Branch Not Taken "),
            }
        }
    }
}

// binary utils
fn imm_extend(data: u16, len: u16, ext_value: u16) -> u16 {
    if ext_value == 0 {
        data
    } else if ext_value == 1 {
        let bitmask: u16 = ((1 << len) - 1) ^ u16::MAX;
        data | bitmask
    } else {
        panic!()
    }
}

fn sign_extend(data: u16, len: u16) -> u16 {
    let sign = data >> (len - 1);
    imm_extend(data, len, sign)
}

fn get_bit(data: u16, pos: u16) -> u16 {
    (data >> pos) & 1
}

fn get_bits(data: u16, end: u16, start: u16) -> u16 {
    let shifted_data: u16 = data >> start;
    let bitmask: u16 = (1 << (end - start + 1)) - 1;
    shifted_data & bitmask
}

#[cfg(test)]
#[allow(overflowing_literals)]
#[allow(arithmetic_overflow)]
mod tests {
    use super::*;

    #[test]
    fn test_imm_extend() {
        assert_eq!(imm_extend(0b1011, 4, 0), 0b00000000_00001011);
        assert_eq!(imm_extend(0b000, 3, 1), 0b11111111_11111000);
    }

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(0b011, 3), 0b00000000_00000011);
        assert_eq!(sign_extend(0b101, 3), 0b11111111_11111101);
    }

    #[test]
    fn test_get_bit() {
        assert_eq!(get_bit(0b1010_1010, 3), 1);
        assert_eq!(get_bit(0b1010_1010, 2), 0);
        assert_eq!(get_bit(0b0000_0010, 1), 1);
        assert_eq!(get_bit(0b1111_1110, 0), 0);
    }

    #[test]
    fn test_get_bits() {
        assert_eq!(get_bits(0b1111_0000, 7, 4), 0b1111);
    }

    #[test]
    fn test_alu_add() {
        let mut cpu = CPU::new();
        let result = cpu.alu(5, 3, 0b000); // ADD
        assert_eq!(result, 8);
        assert!(!cpu.alu_flags.n);
        assert!(!cpu.alu_flags.z);
        assert!(!cpu.alu_flags.c);
        assert!(!cpu.alu_flags.v);

        let result = cpu.alu(0, 0, 0b000); // ADD
        assert_eq!(result, 0);
        assert!(!cpu.alu_flags.n);
        assert!(cpu.alu_flags.z);
        assert!(!cpu.alu_flags.c);
        assert!(!cpu.alu_flags.v);

        let result = cpu.alu(-3i16 as u16, 4, 0b000);
        assert_eq!(result, 1);
        assert!(!cpu.alu_flags.n);
        assert!(!cpu.alu_flags.z);
        assert!(cpu.alu_flags.c);
        assert!(!cpu.alu_flags.v);

        let result = cpu.alu(i16::MAX as u16, 1, 0b000);
        assert_eq!(result, i16::MIN as u16);
        assert!(cpu.alu_flags.n);
        assert!(!cpu.alu_flags.z);
        assert!(!cpu.alu_flags.c);
        assert!(cpu.alu_flags.v);

        let result = cpu.alu(-1i16 as u16, 1, 0b000); // ADD
        assert_eq!(result, 0);
        assert!(!cpu.alu_flags.n); // Not Negative
        assert!(cpu.alu_flags.z); // Zero
        assert!(cpu.alu_flags.c); // Carry
        assert!(!cpu.alu_flags.v); // Not Overflow (a and b have different signs in add)
    }

    #[test]
    fn test_alu_sub() {
        let mut cpu = CPU::new();
        let result = cpu.alu(5, 3, 0b001); // SUB
        assert_eq!(result, 2);
        assert!(!cpu.alu_flags.n);
        assert!(!cpu.alu_flags.z);
    }

    #[test]
    fn test_alu_logic_ops() {
        let mut cpu = CPU::new();
        assert_eq!(cpu.alu(0b1100, 0b1010, 0b010), 0b1000); // AND
        assert_eq!(cpu.alu(0b1100, 0b1010, 0b011), 0b1110); // OR
        assert_eq!(cpu.alu(0b1100, 0b1010, 0b100), 0b0110); // XOR
    }

    #[test]
    fn test_alu_shift_ops() {
        let mut cpu = CPU::new();
        assert_eq!(cpu.alu(0b0011, 1, 0b110), 0b0110); // SHL
        assert_eq!(cpu.alu(0b1100, 1, 0b111), 0b0110); // SHR
    }
}
