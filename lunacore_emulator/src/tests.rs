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
    cpu.decode();
	cpu.execute();
    assert_eq!(cpu.regs.t[0], 2); // mov 2
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.regs.pc, 2);
    cpu.next_cycle();

    // shl t1, t0, !2
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    assert_eq!(cpu.regs.t[1], 8); // 2 << 2
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.regs.pc, 3);
    cpu.next_cycle();

    // sub t2, t1, t0
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    assert_eq!(cpu.regs.t[2], 6); // 8 - 2
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.regs.pc, 4);
    cpu.next_cycle();

    // push t0
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    assert_eq!(cpu.regs.sp, 0x0000 - 2);
    assert_eq!(cpu.dmem.read(0x0000 - 2, 0), 2); // push 2
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.regs.pc, 5);
    cpu.next_cycle();

    // push t2
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    assert_eq!(cpu.regs.sp, 0x0000 - 4);
    assert_eq!(cpu.dmem.read(0x0000 - 4, 0), 6); // push 6
    assert_eq!(cpu.pc, 4);
    assert_eq!(cpu.regs.pc, 6);
    cpu.next_cycle();

    // push !-1
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    assert_eq!(cpu.regs.sp, 0x0000 - 6);
    assert_eq!(cpu.dmem.read(0x0000 - 6, 0), -1i16 as u16); // push -1
    assert_eq!(cpu.pc, 5);
    assert_eq!(cpu.regs.pc, 7);
    cpu.next_cycle();

    // sub bp, sp, !2
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    assert_eq!(cpu.regs.bp, cpu.regs.sp - 2); // sp - 2
    assert_eq!(cpu.pc, 6);
    assert_eq!(cpu.regs.pc, 8);
    cpu.next_cycle();

    // popb t3
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    assert_eq!(cpu.regs.sp, 0x0000 - 5);
    assert_eq!(cpu.regs.t[3], 0xff); // popb t3
    assert_eq!(cpu.pc, 7);
    assert_eq!(cpu.regs.pc, 9);
    cpu.next_cycle();

    // popb t0
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    assert_eq!(cpu.regs.sp, 0x0000 - 4);
    assert_eq!(cpu.regs.t[0], 0xff); // popb t0
    assert_eq!(cpu.pc, 8);
    assert_eq!(cpu.regs.pc, 10);
    cpu.next_cycle();

    // xor t3, t3, t3
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    assert_eq!(cpu.regs.t[3], 0); // xor t3, t3
    assert_eq!(cpu.pc, 9);
    assert_eq!(cpu.regs.pc, 12);
    cpu.next_cycle();

    // mov t1, !0xffe0
    // W
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    assert!(cpu.wide);
    assert_eq!(cpu.regs.t[1], 0xffe0); // !0xffe0
    assert_eq!(cpu.pc, 10);
    assert_eq!(cpu.regs.pc, 13); // + 2 + W
    cpu.next_cycle();

    // add t0, t0, !-1
    cpu.fetch();
    cpu.decode();
	cpu.execute();
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
    cpu.decode();
	cpu.execute();
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.regs.pc, 3);
    assert_eq!(cpu.regs.sp, 0x0000 - 2); // push pc 
    assert_eq!(cpu.dmem.data[(0x0000u16 - 2) as usize], 3);
    cpu.next_cycle();

    // jmp !4 (wide)
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    assert!(cpu.wide);
    assert_eq!(cpu.pc, 8);
    cpu.next_cycle();

    cpu.fetch();
    cpu.decode();
	cpu.execute();
    cpu.next_cycle();
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    cpu.next_cycle();
    cpu.fetch();
    cpu.decode();
	cpu.execute();
    cpu.next_cycle();

    assert_eq!(cpu.regs.t, [0, 0, 0xffff, 0xffff]);
    assert_eq!(cpu.pc, 3); // should not be in 2, which is a wide immediate

    cpu.fetch();
    cpu.decode();
	cpu.execute();

    assert_eq!(cpu.regs.t, [0xffff, 0, 0xffff, 0xffff]);
}

// test of problematic case: wide imm jump with small offset
#[test]
fn lunacore_test_wide_imm_jmp() {
    /*
    0:  jmp label (wide)
    2:  mov t0, !0x9999

    label:
    4:  mov t1, !0x8888 
    */

    let program = [
    // main:
        "1011110000000000",
        "0000000000000001", // (4) - (0+2+w)  =>  label_pc - (pc+2+w)
        "0011101000000000",
        "1001100110011001",
    // label:
        "0011101001000000",
        "1000100010001000",
    ];

    let mut cpu = CPU::new();
    cpu.imem.load_binary_str(program.join("").as_str());
    cpu.debug = true;

    for _ in 0..2 {
        cpu.fetch();
        cpu.debug_instruction();
        cpu.decode();
	cpu.execute();
        cpu.next_cycle();
    }

    assert_eq!(cpu.regs.t, [0, 0x8888, 0, 0]);
}


// testing a function that returns the sum of naturals up to n
#[test]
fn lunacore_test_natural_sum() {
    /*
    main:
        push !104
        push pc
        jmp natural_sum (+1)
        mov t3, t0
        jmp 0x1234

    natural_sum:
        mov t0, !1
        mov t1, !1
        lod t2, [sp + 2]
    while:
        cmp t1, t2      (sub in, t1, t2)
        jge endwhile    (+2)
        add t1, t1, !1 
        add t0, t0, t1
        jmp while       (-6)

    endwhile:
        ret             (pop pc)

    */

    let program = [
    // main:
        "0111010000101000", "0000000001101000",
        "0100010110101000",
        "1001110", "000000001",
        "0000101011000000",
        "1011110000000000", "0001001000110100",

    // natural_sum:
        "0000101000000001",
        "0000101001000001",
        "0101001010101010",

    // while:
        "0000001111001010",
        "1000101", "000000010",
        "0001000001001001",
        "0000000000000001",
        "1001110", "111111010",
    // endwhile:
        "0100011110101000",
    ];

    let mut cpu = CPU::new();
    cpu.debug = true;
    cpu.imem.load_binary_str(program.join("").as_str());

    for _ in 0..1000 {
        cpu.fetch();
        cpu.debug_instruction();
        cpu.decode();
	cpu.execute();
        cpu.next_cycle();
    }

    assert_eq!(cpu.regs.t[3], 104*105/2);
}


// testing multiplication by software
#[test]
fn lunacore_test_multiplication() {
    /*
    main:
        pushb !213
        pushb !71
        push pc
        jmp mul_8b (+2)
        add sp, sp, !2

        mov t3, t0
        jmp 0x1234

    mul_8b:
        mov t0, !0
        lodb t1, [sp + 2]
        lodb t2, [sp + 3]
        
    mul_loop:
        tst t2, !1
        jz skip_add (+0)
        add t0, t0, t1

    skip_add:
        shl t1, t1, !1
        shr t2, t2, !1
        jnz mul_loop (-7)

        ret             (pop pc)

    */

    let program = [
    // main:
        "0111110000101000", "0000000011010101",
        "0111110000101000", "0000000001000111",
        "0100010110101000",
        "1001110", "000000010",
        "0001000101101010",

        "0000101011000000",
        "1011110000000000", "0001001000110100",
    
    // mul_8b:
        "0001101000000000",
        "0101101001101010",
        "0101101010101011",
    // mul_loop:
        "0001010111010001",
        "1000000", "000000000",
        "0000000000000001",
    // skip_add:
        "0001110001001001",
        "0001111010010001",
        "1000001", "111111001",

        "0100011110101000",
    ];

    let mut cpu = CPU::new();
    cpu.debug = true;
    cpu.imem.load_binary_str(program.join("").as_str());

    for _ in 0..56 {
        cpu.fetch();
        cpu.debug_instruction();
        cpu.decode();
	    cpu.execute();
        // cpu.debug_state();
        // cpu.dmem.print_memory(0xfffc, 0xffff);
        cpu.next_cycle();
    }

    assert_eq!(cpu.regs.t[3], 213*71);
    assert_eq!(cpu.regs.sp, 0x0000);

}
