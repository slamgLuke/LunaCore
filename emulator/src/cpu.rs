use crate::components::*;

pub struct CPU {
    pub regs: RegFile,
    pub imem: WordROM,
    pub dmem: ByteRAM,
    pub cond_unit: CondUnit,
    pub pc: u16,

    // datapath
    alu_flags: Flags,
    instr: [u16; 2],
    wide: bool,
    pc_overwritten: bool,

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
            pc_overwritten: false,

            run: true,
            debug: true,
        }
    }

    pub fn debug(&self) {
        println!("pc: {:04x}", self.pc);
        println!("Instr: {:04x}", self.instr[0]);
        println!("{:?}", self.regs);
        println!("{:?}", self.cond_unit.flags);
        println!();
    }

    pub fn fetch(&mut self) {
        self.instr = self.imem.read(self.pc);
    }

    pub fn decode_and_execute(&mut self) {
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
                    _ => panic!("Special instructions not implemented yet!"),
                }
            }
            0b10 => self.branch(),
            _ => panic!(),
        }

        if self.debug && self.wide {
            println!("Wide Immediate used: {:04x}", self.instr[1]);
        }
    }

    pub fn next_cycle(&mut self) {
        // advance pc and reset booleans
        if !self.pc_overwritten {
            self.pc += self.wide as u16 + 1;
        }
        self.wide = false;
        self.pc_overwritten = false;
        self.alu_flags = Flags::new();
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
        if self.debug {
            println!("Executing DP");
        }
        let imm = get_bits(self.instr[0], 13, 12);
        self.wide = imm == 0b11;
        self.regs.pc = self.pc + 2 + self.wide as u16;

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
                println!("DP Result stored in PC");
            }
        }
    }

    fn sav(&mut self) {
        if self.debug {
            println!("Executing SAV");
        }
        let imm = get_bits(self.instr[0], 13, 12);
        self.wide = imm == 0b11;
        self.regs.pc = self.pc + 2 + self.wide as u16;

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
        if self.debug {
            println!("Executing LOD");
        }
        let imm = get_bits(self.instr[0], 13, 12);
        self.wide = imm == 0b11;
        self.regs.pc = self.pc + 2 + self.wide as u16;

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

        let addr = self.alu(src_a, src_b, 0b000);

        let read_data = self.dmem.read(addr, b);
        self.regs.write(td, read_data);
    }

    fn push(&mut self) {
        if self.debug {
            println!("Executing PUSH");
        }
        let imm = get_bits(self.instr[0], 13, 12);
        self.wide = imm == 0b11;
        self.regs.pc = self.pc + 2 + self.wide as u16;

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
        if self.debug {
            println!("Executing POP");
        }
        let imm = get_bits(self.instr[0], 13, 12);
        self.wide = imm == 0b11;
        self.regs.pc = self.pc + 2 + self.wide as u16;

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
    }

    fn branch(&mut self) {
        if self.debug {
            println!("Executing Branch");
        }

        let w = get_bit(self.instr[0], 13);
        self.wide = w == 0b1;
        self.regs.pc = self.pc + 2 + self.wide as u16;

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
                true => println!("Branch Taken"),
                false => println!("Branch Not Taken"),
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
        assert_eq!(cpu.alu(0b0011, 1, 0b101), 0b0110); // LSL
        assert_eq!(cpu.alu(0b1100, 1, 0b110), 0b0110); // LSR
    }
}
