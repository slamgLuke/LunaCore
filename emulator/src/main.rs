#[allow(dead_code)]
mod components;
#[allow(dead_code)]
#[allow(overflowing_literals)]
#[allow(arithmetic_overflow)]
mod cpu;

use crate::cpu::*;

fn main() {
    let mut cpu = CPU::new();
    let program = [
        "0001101000010000", // mov t0, !2
        "0001110000010001", // shl t1, t0, !2
        "0000001001000010", // sub t2, t1, t0
        "0100010101000000", // push t0
        "0100010101000010", // push t2
        "0110010101111000", // push -1
        "0100111101000011", // popb t3
        "0100111101000000", // popb t0
        "0000100011011011", // xor t3, t3, t3
        "0011101000000001", // mov t1, !0xffe0
        "1111111111100000",
        "0010000000111000", // add t0, t0, !-1
        "0101000001000000", // sav t0, [t1+0]
        "0011101000000010", // mov t2, !0x123d
        "0001001000111101",
        "0011110010000010", // shl t2, t2, !15
        "0000000000001111",
        "0011111010000010", // shr t2, t2, !15
        "0000000000001111",
    ];

    cpu.imem.load_binary_str(program.join("").as_str());
    cpu.debug();
    println!();

    let n_wide = 4;
    let instr_len = program.len() - n_wide;

    for i in 0..instr_len {
        println!("instruction: {}", program[i]);
        cpu.fetch();
        cpu.decode_and_execute();
        cpu.next_cycle();
        cpu.debug();
    }

    cpu.dmem.print_memory(0xFFE0, 0xFFFF);
}
