use crate::instructions::*;

pub fn compile(program: &Vec<Instruction>) -> Vec<u16> {
    let mut vec = Vec::new();
    program.iter().for_each(|instr| vec.append(&mut instr.to_binary()));
    vec
}
