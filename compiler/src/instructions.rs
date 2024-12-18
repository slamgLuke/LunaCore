#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Dp { cmd: u8, td: u8, tn: u8, src2: Src2 },

    Mem { bsl: u8, td: u8, tn: u8, src2: Src2 },

    BranchLabel { cond: u8, label: String },

    BranchOffset { cond: u8, offset: Offset },
}

impl Instruction {
    pub fn to_binary(&self) -> Vec<u16> {
        match self {
            Self::Dp { cmd, td, tn, src2 } => {
                let (imm, src2, wide) = match src2 {
                    Src2::Reg(r) => (0b00, *r as u16, false),
                    Src2::ZeroImm3(i) => (0b01, *i as u16, false),
                    Src2::OneImm3(i) => (0b10, *i as u16, false),
                    Src2::WideImm16(i) => (0b11, *i as u16, true),
                };

                let instr: u16 = (00 << 14) | (imm << 12) | ((*cmd as u16) << 9) | ((*td as u16) << 6) | ((*tn as u16) << 3);

                if !wide {
                    vec![(instr | (src2 & 0b111))]
                } else {
                    vec![instr, src2]
                }
            }
            Self::Mem { bsl, td, tn, src2 } => {
                let (imm, src2, wide) = match src2 {
                    Src2::Reg(r) => (0b00, *r as u16, false),
                    Src2::ZeroImm3(i) => (0b01, *i as u16, false),
                    Src2::OneImm3(i) => (0b10, *i as u16, false),
                    Src2::WideImm16(i) => (0b11, *i as u16, true),
                };

                let instr: u16 = (01 << 14) | (imm << 12) | ((*bsl as u16) << 9) | ((*td as u16) << 6) | ((*tn as u16) << 3);

                if !wide {
                    vec![(instr | (src2 & 0b111))]
                } else {
                    vec![instr, src2]
                }
            }
            Self::BranchOffset { cond, offset } => {
                let (w, offset, wide) = match offset {
                    Offset::SignImm9(i) => (0, *i as u16, false),
                    Offset::WideImm16(i) => (1, *i as u16, true),
                };

                let instr = (10 << 14) | (w << 13) | ((*cond as u16) << 9);

                if !wide {
                    vec![(instr | offset & 0x1f)]
                } else {
                    vec![instr, offset]
                }
            }
            Self::BranchLabel { .. } => {
                panic!("Trying to convert BranchLabel type to binary, when it has to be converted to BranchOffset beforehand");
            }
        }
    }

    pub fn is_wide(&self) -> bool {
        if matches!(
            self,
            Instruction::Dp {
                src2: Src2::WideImm16(_),
                ..
            } | Instruction::Mem {
                src2: Src2::WideImm16(_),
                ..
            } | Instruction::BranchOffset {
                offset: Offset::WideImm16(_),
                ..
            } | Instruction::BranchLabel { .. } // Assumming every branch label is wide
        ) {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Src2 {
    Reg(u8),
    ZeroImm3(u8),
    OneImm3(i8),
    WideImm16(i16),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Offset {
    SignImm9(i16),
    WideImm16(i16),
}
