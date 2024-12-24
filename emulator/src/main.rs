#[allow(dead_code)]
mod components;
#[allow(dead_code)]
#[allow(overflowing_literals)]
#[allow(arithmetic_overflow)]
mod cpu;
#[cfg(test)]
#[allow(overflowing_literals)]
#[allow(arithmetic_overflow)]
mod tests;

use crate::cpu::*;

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub fn load_binary_file(file_path: &str) -> io::Result<Vec<u16>> {
    let path = Path::new(file_path);
    let mut file = File::open(&path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if buffer.len() % 2 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File size is not a multiple of 2 bytes",
        ));
    }

    // convert the byte buffer into u16 values
    let mut instructions = Vec::new();
    for chunk in buffer.chunks(2) {
        // interpret two bytes as a little-endian u16
        let word = u16::from_le_bytes([chunk[0], chunk[1]]);
        instructions.push(word);
    }

    Ok(instructions)
}


fn main() {
    let mut cpu = CPU::new();
    cpu.debug = true;

    let input_filename = "sort.lunaexe";
    let binary = load_binary_file(input_filename).expect("Invalid binary file");
    cpu.imem.load_binary(&binary[..]);

    let data = [
        "00000101", // 5
        "00000001", // 1
        "00000011", // 3
        "00001000", // 8
        "00000010", // 2
        "00000110", // 6
        "00000100", // 4
        "00000111", // 7
        "00001001", // 9
        "00000000", // 0
    ];
    cpu.dmem.load_binary_str(data.join("").as_str());
    
    for _ in 0..1000 {
        cpu.fetch();
        cpu.debug_instruction();
        //cpu.debug_state();
        cpu.decode_and_execute();

        cpu.next_cycle();
    }

    cpu.debug_state();
    // cpu.dmem.print_memory(0xFFE0, 0xFFFF);
    cpu.dmem.print_memory(0x0000, 0x000F);
}
