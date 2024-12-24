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

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
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
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Missing Filename\nUsage: {} <binary_file>", args[0]);
        std::process::exit(1);
    }

    let input_filename = &args[1];

    let mut cpu = CPU::new();

    let binary = load_binary_file(input_filename).expect("Invalid binary file");
    let program_len = binary.len();
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

    // Enter interactive mode
    interactive_mode(&mut cpu, program_len);
}

fn interactive_mode(cpu: &mut CPU, program_len: usize) -> i32 {
    let mut breakpoints = Vec::new();
    let mut cycle_count = 0;

    loop {
        print!("(emulator) > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let command = input.trim();

        match command {
            "r" | "run" => {
                // run until the program ends or a breakpoint is hit
                while (cpu.pc as usize) < program_len && !breakpoints.contains(&cpu.pc) {
                    cpu.fetch();
                    cpu.decode();
                    cpu.debug_instruction();
                    cpu.execute();
                    cpu.next_cycle();
                    cycle_count += 1;
                }
                if (cpu.pc as usize) >= program_len {
                    println!("PC out of bounds. Program Halted.");
                } else {
                    println!("Hit breakpoint at PC: 0x{:04x}", cpu.pc);
                }
            }
            "s" | "step" => {
                // execute a single instruction
                if (cpu.pc as usize) >= program_len {
                    println!("PC out of bounds. Program Halted.");
                }
                cpu.fetch();
                cpu.decode();
                cpu.debug_instruction();
                cpu.execute();
                cpu.next_cycle();
                cycle_count += 1;
            }
            "state" => cpu.debug_state(),

            "memory" => {
                // Prompt for memory range
                print!("Enter memory range (start end): ");
                io::stdout().flush().unwrap();
                let mut range_input = String::new();
                io::stdin().read_line(&mut range_input).unwrap();
                let parts: Vec<_> = range_input.trim().split_whitespace().collect();

                if parts.len() == 2 {
                    let parse_value = |s: &str| {
                        if s.starts_with("0x") {
                            u16::from_str_radix(&s[2..], 16) // Parse as hexadecimal
                        } else {
                            s.parse::<u16>() // Parse as decimal
                        }
                    };

                    if let (Ok(start), Ok(end)) = (parse_value(parts[0]), parse_value(parts[1])) {
                        cpu.dmem.print_memory(start, end);
                    } else {
                        println!("Invalid memory range.");
                    }
                } else {
                    println!("Usage: start end");
                }
            }
            cmd if cmd.starts_with("break") => {
                // Add a breakpoint
                if let Some(pc_str) = cmd.strip_prefix("break ") {
                    let parse_value = |s: &str| {
                        if s.starts_with("0x") {
                            u16::from_str_radix(&s[2..], 16) // Parse as hexadecimal
                        } else {
                            s.parse::<u16>() // Parse as decimal
                        }
                    };

                    if let Ok(pc) = parse_value(pc_str) {
                        breakpoints.push(pc);
                        println!("Breakpoint added at 0x{:04X}", pc);
                    } else {
                        println!("Invalid breakpoint address.");
                    }
                } else {
                    println!("Usage: break <address>");
                }
            }

            "help" => {
                println!("Commands:");
                println!("  run  (r)        - Run the program until completion or breakpoint");
                println!("  step (s)        - Execute the next instruction");
                println!("  state           - Print the CPU state");
                println!("  memory          - Print memory contents (requires range)");
                println!("  break <pc>      - Add a breakpoint at the given PC address");
                println!("  help            - Show this help message");
                println!("  quit            - Exit the emulator");
            }
            "quit" => {
                println!("Exiting emulator.");
                break;
            }
            _ => println!("Unknown command. Type 'help' for a list of commands."),
        }
    }

    cycle_count
}
