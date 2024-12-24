#[allow(arithmetic_overflow)]
mod compiler;
#[allow(arithmetic_overflow)]
mod instructions;
#[allow(arithmetic_overflow)]
mod parser;

use compiler::compile;
use instructions::*;
use parser::*;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    let input_filename = "sort.luna";
    let output_filename = "sort.lunaexe";

    let input = fs::read_to_string(input_filename).unwrap();
    let mut parser = Parser::new();
    match parser.parse_program(&input, input_filename) {
        Err(err) => {
            eprintln!("{}\nNo file was generated", err);
            return Ok(());
        }
        Ok(_) => (),
    }

    println!("{} instruction parsed:\n{:?}", parser.program.len(), parser.program);
    println!("Labels: {:#?}", parser.label_map);

    let binary = compile(&parser.get_program());
    let mut file = File::create(&Path::new(output_filename))?;
    for word in &binary {
        file.write_all(&word.to_le_bytes())?;
    }

    println!("Executable generated succesfully at: {}", output_filename);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_test() {
        let program = "
                _start: \n
                        mov t0, !5 \n
                        mov t1, !0x1000\n
                        xor t2, t2\n
                        push pc \n
                        JMP loop \n
                        Jmp _end \n
                \n
                loop:   cmp t0, !0 \n
                        jeq endloop \n
                        savB T0, [T1+T2] \n
                        dec t0 \n 
                        inc t2 \n
                        jmp loop \n
                endloop:\n
                        ret \n
                _end: \n
            ";

        let mut parser = Parser::new();
        assert_eq!(parser.parse_program(program, "test"), Ok(()));

        println!("Parsed Program: {:#?}", parser.program);
        println!("Labels: {:#?}", parser.label_map);

        assert_eq!(parser.label_map.len(), 4);
        assert_eq!(parser.label_map.get("_start"), Some(&0));
        assert_eq!(parser.label_map.get("loop"), Some(&9));
        assert_eq!(parser.label_map.get("endloop"), Some(&17));
        assert_eq!(parser.label_map.get("_end"), Some(&18));

        assert_eq!(
            parser.program[4],
            (
                5,
                Instruction::BranchOffset {
                    cond: 0b1110,
                    offset: Offset::WideImm16(1)
                }
            )
        );
    }

    #[test]
    fn parse_and_compile() {
        let program = "
            main:
                push !104
                push pc
                jmp natural_sum
                mov t3, t0
                jmp end

            natural_sum:
                mov t0, !1
                mov t1, !1
                lod t2, [sp + !0x02]
            while:
                cmp t1, t2
                jge endwhile
                add t1, t1, !1 
                nop
                add t0, t0, t1
                jmp while

            endwhile:
                ret

            end:
            ";

        let mut parser = Parser::new();
        assert_eq!(parser.parse_program(program, "test"), Ok(()));

        println!("Parsed Program: {:#?}", parser.program);
        println!("Labels: {:#?}", parser.label_map);

        let binary = compile(&parser.get_program());
        let expected: Vec<u16> = vec![
            // main:
            0b0111010000101000, 104,
            0b0100010110101000,
            0b1011110000000000, 2,
            0b0000101011000000,
            0b1011110000000000, 11,
            // natural_sum:
            0b0001101000000001,
            0b0001101001000001,
            0b0101001010101010,
            // while:
            0b0000001111001010,
            0b1010101000000000, 4,
            0b0001000001001001,
            0b1001111000000000,
            0b0000000000000001,
            0b1011110000000000, -9i16 as u16,
            // endwhile:
            0b0100011110101000,
            // end:
        ];

        assert_eq!(binary, expected);
    }
}
