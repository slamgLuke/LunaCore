use crate::cpu::*;


// test of general dp instructions
#[test]
fn lunacore_test_basic_dp() {
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
    ];

    cpu.imem.load_binary_str(program.join("").as_str());
    assert_eq!(cpu.pc, 0);

    // mov t0, !2
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.regs.t[0], 2); // mov 2
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.regs.pc, 2);
    cpu.next_cycle();

    // shl t1, t0, !2
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.regs.t[1], 8); // 2 << 2
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.regs.pc, 3);
    cpu.next_cycle();

    // sub t2, t1, t0
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.regs.t[2], 6); // 8 - 2
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.regs.pc, 4);
    cpu.next_cycle();

    // push t0
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.regs.sp, 0x0000 - 2);
    assert_eq!(cpu.dmem.read(0x0000 - 2, 0), 2); // push 2
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.regs.pc, 5);
    cpu.next_cycle();

    // push t2
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.regs.sp, 0x0000 - 4);
    assert_eq!(cpu.dmem.read(0x0000 - 4, 0), 6); // push 6
    assert_eq!(cpu.pc, 4);
    assert_eq!(cpu.regs.pc, 6);
    cpu.next_cycle();

    // push !-1
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.regs.sp, 0x0000 - 6);
    assert_eq!(cpu.dmem.read(0x0000 - 6, 0), -1i16 as u16); // push -1
    assert_eq!(cpu.pc, 5);
    assert_eq!(cpu.regs.pc, 7);
    cpu.next_cycle();

    // sub bp, sp, !2
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.regs.bp, cpu.regs.sp - 2); // sp - 2
    assert_eq!(cpu.pc, 6);
    assert_eq!(cpu.regs.pc, 8);
    cpu.next_cycle();

    // popb t3
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.regs.sp, 0x0000 - 5);
    assert_eq!(cpu.regs.t[3], 0xff); // popb t3
    assert_eq!(cpu.pc, 7);
    assert_eq!(cpu.regs.pc, 9);
    cpu.next_cycle();

    // popb t0
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.regs.sp, 0x0000 - 4);
    assert_eq!(cpu.regs.t[0], 0xff); // popb t0
    assert_eq!(cpu.pc, 8);
    assert_eq!(cpu.regs.pc, 10);
    cpu.next_cycle();

    // xor t3, t3, t3
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.regs.t[3], 0); // xor t3, t3
    assert_eq!(cpu.pc, 9);
    assert_eq!(cpu.regs.pc, 12);
    cpu.next_cycle();

    // mov t1, !0xffe0
    // W
    cpu.fetch();
    cpu.decode_and_execute();
    assert!(cpu.wide);
    assert_eq!(cpu.regs.t[1], 0xffe0); // !0xffe0
    assert_eq!(cpu.pc, 10);
    assert_eq!(cpu.regs.pc, 13); // + 2 + W
    cpu.next_cycle();

    // add t0, t0, !-1
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.regs.t[0], 0xfe); // t0 - 1
    assert_eq!(cpu.pc, 12);
    assert_eq!(cpu.regs.pc, 14);
    cpu.next_cycle();
}



// test of problematic case: pushing pc to stack before a wide instruction
#[test]
fn lunacore_test_push_pc_wide() {
    let mut cpu = CPU::new();
    cpu.debug = true;

    let program = [
        "0100010110101000", // push pc       (3 has to be pushed)
        "1011110000000000", // jmp !4 (wide)
        "0000000000000100",
        "0010101000000111", // mov t0, !-1
        "1111111111111111",
        "1111111111111111",
        "1111111111111111",
        "0010101001000111", // mov t1, !-1
        "0010101010000111", // mov t2, !-1
        "0010101011000111", // mov t3, !-1
        "0100011110101000", // ret (pop pc)
    ];

    cpu.imem.load_binary_str(program.join("").as_str());
    assert_eq!(cpu.pc, 0);

    // push pc
    cpu.fetch();
    cpu.decode_and_execute();
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.regs.pc, 3);
    assert_eq!(cpu.regs.sp, 0x0000 - 2); // push pc 
    assert_eq!(cpu.dmem.data[(0x0000u16 - 2) as usize], 3);
    cpu.next_cycle();

    // jmp !4 (wide)
    cpu.fetch();
    cpu.decode_and_execute();
    assert!(cpu.wide);
    assert_eq!(cpu.pc, 8);
    cpu.next_cycle();

    cpu.fetch();
    cpu.decode_and_execute();
    cpu.next_cycle();
    cpu.fetch();
    cpu.decode_and_execute();
    cpu.next_cycle();
    cpu.fetch();
    cpu.decode_and_execute();
    cpu.next_cycle();

    assert_eq!(cpu.regs.t, [0, 0, 0xffff, 0xffff]);
    assert_eq!(cpu.pc, 3); // should not be in 2, which is a wide immediate

    cpu.fetch();
    cpu.decode_and_execute();

    assert_eq!(cpu.regs.t, [0xffff, 0, 0xffff, 0xffff]);
}
