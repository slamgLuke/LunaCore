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
        "0001000000010000", // add t0, t0, !2
        "0001101000010001", // shl t1, t0, !2
        "0000001001000010", // sub t2, t1, t0
        "0100010101000000", // push t0
        "0100010101000010", // push t2
        "0110010101111000", // push -1
    ];

    cpu.imem.load_binary_str(program.join("").as_str());
    cpu.debug();

    for instr in program {
        println!("instruction: {}", instr);
        cpu.fetch();
        cpu.decode_and_execute();
        cpu.next_cycle();
        cpu.debug();
    }

    cpu.dmem.print_memory(0xFFF0, 0xFFFF);
}
