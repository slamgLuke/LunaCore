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

fn main() {
    let mut cpu = CPU::new();
    cpu.debug = true;

    let program = [
        "0001101000000010", // mov t0, !2
        "0001110001000010", // shl t1, t0, !2
        "0000001010001000", // sub t2, t1, t0
        "0100010000101000", // push t0
        "0100010010101000", // push t2
        "0110010000101111", // push !-1
        "0001001100101010", // sub bp, sp, !2
        "0100111011101000", // popb t3
        "0100111000101000", // popb t0
        "0000100011011011", // xor t3, t3, t3
        "0011101001000000", // mov t1, !0xffe0
        "1111111111100000",
        "0010000000000111", // add t0, t0, !-1
        "0101000000001000", // sav t0, [t1+0]
        "0011101010000000", // mov t2, !0x123d
        "0001001000111101",
        "0011110010010000", // shl t2, t2, !15
        "0000000000001111",
        "0011111010010000", // shr t2, t2, !15
        "0000000000001111",
        "0001001111111001", // sub in, in, !1
        "1001111111110101", // jnot -11 (add t0, t0, !-1)
        "1001110000000000", // jmp 0 (skips next)
        "1001110111100010", // jmp -30
        "0001101000000000", // mov t0, !0
        "1000001111100010", // jnz -30
        "0001101001000000", // mov t1, !0
        "0001000101101001", // add pc, pc, !1 (skips next)
        "1010000000000000", // jmpz 0xf999
        "1111100110011001",
        "0010101011000000", // mov t3, !-8
        "0100100011100000", // savb t3, [bp+0]
    ];

    cpu.imem.load_binary_str(program.join("").as_str());
    cpu.debug();
    println!();

    let n_wide = 4;
    let program_len = program.len() as i32 - n_wide + 1;
    let extra_instructions = -3;

    for _ in 0..(program_len + extra_instructions) {
        cpu.fetch();
        cpu.decode_and_execute();
        cpu.debug();
        cpu.next_cycle();
    }

    cpu.dmem.print_memory(0xFFE0, 0xFFFF);
}
