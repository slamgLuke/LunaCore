use crate::instructions::*;
use std::collections::HashMap;

pub struct Parser {
    pub program: Vec<(u16, Instruction)>,
    pub label_map: HashMap<String, u16>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            label_map: HashMap::new(),
            program: Vec::new(),
        }
    }

    pub fn parse_program(&mut self, input: &str, filename: &str) -> Result<(), String> {
        self.program.clear();
        self.label_map.clear();

        let mut pc: u16 = 0;
        let mut line_number = 0;

        // first pass — assuming every BranchLabel is wide
        for line in input.lines() {
            line_number += 1;
            let mut line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(";") || line.starts_with("//") {
                continue;
            }

            // Handle labels
            if let Some((label, rest)) = line.split_once(':') {
                let label = label.trim().to_string();
                if self.label_map.contains_key(&label) {
                    return Err(format!(
                        "Error in {} line {}\nDuplicate label: {}",
                        filename, line_number, label
                    ));
                }
                self.label_map.insert(label, pc);

                // Parse the instruction after the label
                line = rest.trim();
                if line.is_empty() || line.starts_with(";") || line.starts_with("//") {
                    continue;
                }
            }

            match parse_instruction(line) {
                Ok(instruction) => {
                    let pc_step = if instruction.is_wide() { 2 } else { 1 };
                    self.program.push((pc, instruction));
                    pc += pc_step;
                }
                Err(err) => {
                    return Err(format!(
                        "Error in {} line {}\n{}",
                        filename, line_number, err
                    ));
                }
            }
        }

        // second pass - validating BranchLabel instructions
        for line in input.lines() {
            line_number += 1;
            let mut line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(";") || line.starts_with("//") {
                continue;
            }

            // skip labels
            if let Some((_, rest)) = line.split_once(':') {
                // Parse the instruction after the label
                line = rest.trim();
                if line.is_empty() || line.starts_with(";") || line.starts_with("//") {
                    continue;
                }
            }

            match parse_instruction(line) {
                Ok(instruction) => {
                    if let Instruction::BranchLabel { label, .. } = instruction {
                        match self.label_map.get(&label) {
                            Some(_) => (),
                            None => {
                                return Err(format!(
                                    "Error in {} line {}\nLabel {} not found",
                                    filename, line_number, &label
                                ))
                            }
                        }
                    }
                }
                Err(err) => {
                    return Err(format!(
                        "Error in {} line {}\n{}",
                        filename, line_number, err
                    ));
                }
            }
        }

        // third pass - convert BranchLabel to BranchOffset
        for (pc, instruction) in self.program.iter_mut() {
            if let Instruction::BranchLabel { cond, label } = instruction {
                if let Some(&label_pc) = self.label_map.get(label) {
                    let w = 1; // assuming every BranchLabel is wide
                    let offset = (label_pc as i16) - (*pc as i16 + 2 + w);
                    *instruction = Instruction::BranchOffset {
                        cond: *cond,
                        offset: Offset::WideImm16(offset),
                    }
                } else {
                    return Err(format!("\nLabel '{}' not found", &label));
                }
            }
        }

        Ok(())
    }

    pub fn get_program(&self) -> Vec<Instruction> {
        self.program.clone().into_iter().map(|(_, instr)| instr).collect::<Vec<_>>()
    }
}


fn parse_instruction(line: &str) -> Result<Instruction, String> {
    let line = line.replace(",", " ");
    let line = line.replace("[", " [ ");
    let line = line.replace("]", " ] ");
    let line = line.replace("+", " + ");
    let line = line.to_lowercase();

    let normalized_instruction = line.split_whitespace().collect::<Vec<_>>();

    let opcode = normalized_instruction[0];
    let operands = &normalized_instruction[1..];

    match opcode {
        "add" | "sub" | "and" | "or" | "xor" | "mov" | "shl" | "shr" => parse_dp(opcode, operands),
        "lod" | "lodb" | "sav" | "savb" => parse_mem(opcode, operands),
        "push" | "pushb" => parse_push(operands, opcode == "pushb"),
        "pop" | "popb" => parse_pop(operands, opcode == "popb"),
        "inc" | "dec" | "not" | "cmp" | "tst" | "ret" | "nop" => parse_alias(opcode, operands),
        _ => match opcode.chars().collect::<Vec<_>>()[0] {
            'j' => parse_branch(opcode, operands),
            _ => Err(format!("Invalid opcode {}", opcode)),
        },
    }
}

fn parse_dp(opcode: &str, operands: &[&str]) -> Result<Instruction, String> {
    let cmd = parse_cmd(opcode).unwrap();
    if operands.len() == 3 && cmd != 0b101 {
        let td = parse_register(operands[0])?;
        let tn = parse_register(operands[1])?;
        let src2 = parse_register_or_imm(operands[2])?;
        Ok(Instruction::Dp { cmd, td, tn, src2 })
    }
    // mov
    else if operands.len() == 2 && cmd == 0b101 {
        let td = parse_register(operands[0])?;
        let src2 = parse_register_or_imm(operands[1])?;
        Ok(Instruction::Dp {
            cmd,
            td,
            tn: 0b000,
            src2,
        })
    }
    // Td = Tn
    else if operands.len() == 2 {
        let td = parse_register(operands[0])?;
        let tn = td;
        let src2 = parse_register_or_imm(operands[1])?;
        Ok(Instruction::Dp { cmd, td, tn, src2 })
    } else {
        Err(format!("Invalid operands for {}", opcode))
    }
}

fn parse_mem(opcode: &str, operands: &[&str]) -> Result<Instruction, String> {
    let bsl = match opcode {
        "sav" => 0b000,
        "savb" => 0b100,
        "lod" => 0b001,
        "lodb" => 0b101,
        _ => panic!("Parsing non-existant MEM opcode {}", opcode),
    };

    // sav td [ tn ]
    if operands.len() == 4 {
        let td = parse_register(operands[0])?;
        match_token(operands[1], "[")?;
        let tn = parse_register(operands[2])?;
        match_token(operands[3], "]")?;

        Ok(Instruction::Mem {
            bsl,
            td,
            tn,
            src2: Src2::ZeroImm3(0),
        })
    }
    // sav td [ tn + src2 ]
    else if operands.len() == 6 {
        let td = parse_register(operands[0])?;
        match_token(operands[1], "[")?;
        let tn = parse_register(operands[2])?;
        match_token(operands[3], "+")?;
        let src2 = parse_register_or_imm(operands[4])?;
        match_token(operands[5], "]")?;

        Ok(Instruction::Mem { bsl, td, tn, src2 })
    } else {
        Err(format!("Invalid operands for {}", opcode))
    }
}

fn parse_push(operands: &[&str], byte: bool) -> Result<Instruction, String> {
    if operands.len() == 1 {
        let bsl = if byte { 0b110 } else { 0b010 };
        let write_data = parse_register_or_imm(operands[0])?;

        match write_data {
            Src2::Reg(r) => Ok(Instruction::Mem {
                bsl,
                td: r,
                tn: 0b101,
                src2: Src2::Reg(0b000), // don't care when pushing a reg
            }),
            _ => Ok(Instruction::Mem {
                bsl,
                td: 0b000, // don't care when pushing an immediate
                tn: 0b101,
                src2: write_data,
            }),
        }
    } else {
        Err("Invalid operands for push".into())
    }
}

fn parse_pop(operands: &[&str], byte: bool) -> Result<Instruction, String> {
    if operands.len() == 1 {
        let bsl = if byte { 0b111 } else { 0b011 };
        let td = parse_register(operands[0])?;
        Ok(Instruction::Mem {
            bsl,
            td,
            tn: 0b101,
            src2: Src2::Reg(0b000), // don't care when popping
        })
    } else {
        Err("Invalid operands for pop".into())
    }
}

fn parse_alias(opcode: &str, operands: &[&str]) -> Result<Instruction, String> {
    let mut vec = operands.to_vec();
    match opcode {
        "inc" => {
            vec.push("!1");
            if let Ok(instr) = parse_dp("add", &vec[..]) {
                Ok(instr)
            } else {
                Err("Invalid arguments for inc".into())
            }
        }
        "dec" => {
            vec.push("!1");
            if let Ok(instr) = parse_dp("sub", &vec[..]) {
                Ok(instr)
            } else {
                Err("Invalid arguments for dec".into())
            }
        }
        "not" => {
            vec.push("!-1");
            if let Ok(instr) = parse_dp("xor", &vec[..]) {
                Ok(instr)
            } else {
                Err("Invalid arguments for not".into())
            }
        }
        "cmp" => {
            vec.insert(0, "in");
            if let Ok(instr) = parse_dp("sub", &vec[..]) {
                Ok(instr)
            } else {
                Err("Invalid arguments for cmp".into())
            }
        }
        "tst" => {
            vec.insert(0, "in");
            if let Ok(instr) = parse_dp("and", &vec[..]) {
                Ok(instr)
            } else {
                Err("Invalid arguments for tst".into())
            }
        }
        "ret" => {
            if vec.is_empty() {
                Ok(Instruction::Mem {
                    bsl: 0b011,
                    td: 0b110,
                    tn: 0b101,
                    src2: Src2::Reg(0b000),
                })
            } else {
                Err(format!("Unexpected token '{}' after ret", vec[0]))
            }
        }
        "nop" => {
            if vec.is_empty() {
                Ok(Instruction::BranchOffset {
                    cond: 0b1111,
                    offset: Offset::SignImm9(0),
                })
            } else {
                Err(format!("Unexpected token '{}' after nop", vec[0]))
            }
        }
        _ => Err(format!("Parsing non-existant Alias {}", opcode)),
    }
}

fn parse_branch(opcode: &str, operands: &[&str]) -> Result<Instruction, String> {
    if operands.len() == 1 {
        let label = operands[0].to_string();
        Ok(Instruction::BranchLabel {
            cond: parse_cond(opcode)?,
            label,
        })
    } else {
        Err(format!("Invalid operands for {}", opcode))
    }
}

fn parse_register(token: &str) -> Result<u8, String> {
    match token {
        "t0" => Ok(0b000),
        "t1" => Ok(0b001),
        "t2" => Ok(0b010),
        "t3" => Ok(0b011),
        "bp" => Ok(0b100),
        "sp" => Ok(0b101),
        "pc" => Ok(0b110),
        "in" => Ok(0b111),
        _ => Err(format!("Invalid register {}", token)),
    }
}

fn parse_cmd(token: &str) -> Result<u8, String> {
    match token {
        "add" => Ok(0b000),
        "sub" => Ok(0b001),
        "and" => Ok(0b010),
        "or" => Ok(0b011),
        "xor" => Ok(0b100),
        "mov" => Ok(0b101),
        "shl" => Ok(0b110),
        "shr" => Ok(0b111),
        _ => Err(format!("Parsing non-existant DP cmd: {}", token)),
    }
}

fn parse_register_or_imm(token: &str) -> Result<Src2, String> {
    if let Ok(reg) = parse_register(token) {
        Ok(Src2::Reg(reg))
    } else if let Some(imm_token) = token.strip_prefix('!') {
        // hex imm
        let imm = if imm_token.starts_with("0x") {
            i16::from_str_radix(imm_token.strip_prefix("0x").unwrap(), 16)
                .map_err(|_| format!("Invalid hexadecimal immediate: {}", imm_token))?
        }
        // decimal imm
        else {
            imm_token
                .parse::<i16>()
                .map_err(|_| format!("Invalid decimal immediate: {}", imm_token))?
        };

        if imm >= 0 && imm <= 7 {
            Ok(Src2::ZeroImm3(imm as u8))
        } else if imm >= -8 && imm < 0 {
            Ok(Src2::OneImm3(imm as i8))
        } else {
            Ok(Src2::WideImm16(imm))
        }
    } else {
        Err(format!("Invalid register or immediate: {}", token))
    }
}

fn parse_cond(token: &str) -> Result<u8, String> {
    let cond = &token[1..];
    match cond {
        "z" | "eq" => Ok(0b0000),
        "nz" | "ne" => Ok(0b0001),
        "lt" => Ok(0b0010),
        "le" => Ok(0b0011),
        "gt" => Ok(0b0100),
        "ge" => Ok(0b0101),
        "ult" | "cc" => Ok(0b0110),
        "ule" => Ok(0b0111),
        "ugt" => Ok(0b1000),
        "uge" | "cs" => Ok(0b1001),
        "mi" | "ns" => Ok(0b1010),
        "pl" | "nc" => Ok(0b1011),
        "vs" => Ok(0b1100),
        "vc" => Ok(0b1101),
        "al" | "mp" => Ok(0b1110),
        "nv" => Ok(0b1111),
        _ => Err(format!("Invalid conditional for JMP instruction: {}", cond)),
    }
}

fn match_token(token: &str, pat: &str) -> Result<(), String> {
    if token == pat {
        Ok(())
    } else {
        Err(format!("Expected '{}', found '{}'", pat, token))
    }
}
