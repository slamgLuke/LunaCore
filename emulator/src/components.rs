use std::fmt;

pub struct RegFile {
    pub t: [u16; 4], // General-purpose: t0 to t3
    pub bp: u16,     // Base pointer
    pub sp: u16,     // Stack pointer
    pub pc: u16,     // Program counter
    pub input: u16,  // Read-only input register
}

impl RegFile {
    pub fn new() -> Self {
        RegFile {
            t: [0; 4],
            bp: 0,
            sp: 0,
            pc: 0,
            input: 0,
        }
    }

    pub fn read(&self, i: u16) -> u16 {
        match i {
            0..=3 => self.t[i as usize],
            4 => self.bp,
            5 => self.sp,
            6 => self.pc,
            7 => self.input,
            _ => panic!(),
        }
    }

    pub fn write(&mut self, i: u16, data: u16) {
        match i {
            0..=3 => self.t[i as usize] = data,
            4 => self.bp = data,
            5 => self.sp = data,
            6 => self.pc = data,
            7 => (), // input reg is read-only
            _ => panic!(),
        }
    }
}

impl fmt::Debug for RegFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "T: [{:04x}, {:04x}, {:04x}, {:04x}]\nBP: {:04x}, SP: {:04x}\nPC: {:04x}, IN: {:04x}",
            self.t[0], self.t[1], self.t[2], self.t[3], self.bp, self.sp, self.pc, self.input
        )
    }
}

pub const MEMORY_SIZE: usize = 1 << 16; // 2^16 = 65536 for a 16-bit physical address space
pub struct ByteRAM {
    pub data: [u8; MEMORY_SIZE],
}

impl ByteRAM {
    pub fn new() -> Self {
        ByteRAM {
            data: [0; MEMORY_SIZE],
        }
    }

    pub fn read(&self, addr: u16, byte_mode: u16) -> u16 {
        match byte_mode {
            1 => self.data[addr as usize] as u16,
            0 => {
                if !is_word_aligned(addr) {
                    panic!("Error: Misaligned address trying to read a word in memory");
                }
                let msb = self.data[(addr + 1) as usize];
                let lsb = self.data[addr as usize];
                into_word(msb, lsb)
            }
            _ => panic!(),
        }
    }

    pub fn write(&mut self, addr: u16, write_data: u16, byte_mode: u16) {
        match byte_mode {
            1 => self.data[addr as usize] = write_data as u8,
            0 => {
                if !is_word_aligned(addr) {
                    panic!("Error: Misaligned address trying to write a word to memory");
                }
                self.data[(addr + 1) as usize] = get_msb(write_data);
                self.data[addr as usize] = get_lsb(write_data);
            }
            _ => panic!(),
        }
    }

    pub fn load_binary(&mut self, binary_data: &[u8]) {
        assert!(
            binary_data.len() <= MEMORY_SIZE,
            "Input binary is larger than memory!"
        );
        self.data.copy_from_slice(&binary_data[0..MEMORY_SIZE]);
    }

    pub fn load_binary_str(&mut self, binary_string: &str) {
        assert!(
            binary_string.len() % 16 == 0,
            "Binary string must be 16-bit aligned."
        );

        let mut idx = 0;
        for chunk in binary_string.as_bytes().chunks(16) {
            if idx < MEMORY_SIZE {
                let bin_str = std::str::from_utf8(chunk).unwrap();
                let byte = u8::from_str_radix(bin_str, 2).unwrap();
                self.data[idx] = byte;
                idx += 1;
            }
        }
    }

    pub fn print_memory(&self, start: u16, end: u16) {
        println!("       MSB LSB");
        for i in (start..end).step_by(2) {
            println!(
                "0x{:04x}: {:02x} {:02x}",
                i,
                self.data[(i + 1) as usize],
                self.data[i as usize]
            );
        }
    }
}

pub struct WordROM {
    pub data: [u16; MEMORY_SIZE],
}

impl WordROM {
    pub fn new() -> Self {
        WordROM {
            data: [0; MEMORY_SIZE],
        }
    }

    pub fn read(&self, addr: u16) -> [u16; 2] {
        [self.data[addr as usize], self.data[(addr + 1) as usize]]
    }

    pub fn load_binary(&mut self, binary_data: &[u16]) {
        assert!(
            binary_data.len() <= MEMORY_SIZE,
            "Input binary is larger than memory!"
        );
        self.data.copy_from_slice(&binary_data[0..MEMORY_SIZE]);
    }

    pub fn load_binary_str(&mut self, binary_string: &str) {
        assert!(
            binary_string.len() % 16 == 0,
            "Binary string must be 16-bit aligned."
        );

        let mut idx = 0;
        for chunk in binary_string.as_bytes().chunks(16) {
            if idx < MEMORY_SIZE {
                let bin_str = std::str::from_utf8(chunk).unwrap();
                let word = u16::from_str_radix(bin_str, 2).unwrap();
                self.data[idx] = word;
                idx += 1;
            }
        }
    }

    pub fn print_memory(&self, start: u16, end: u16) {
        for i in start..end {
            println!("{:04x} : {:04x}", i, self.data[i as usize]);
        }
    }
}

pub struct Flags {
    pub n: bool,
    pub z: bool,
    pub c: bool,
    pub v: bool,
}

impl Flags {
    pub fn new() -> Self {
        Flags {
            n: false,
            z: false,
            c: false,
            v: false,
        }
    }
}

impl fmt::Debug for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (n, z, c, v) = (self.n as u8, self.z as u8, self.c as u8, self.v as u8);
        write!(f, "NZCV: {}{}{}{}", n, z, c, v)
    }
}

pub struct CondUnit {
    pub flags: Flags,
}

impl CondUnit {
    pub fn new() -> Self {
        CondUnit {
            flags: Flags::new(),
        }
    }

    pub fn write_flags(&mut self, wflags: &Flags) {
        self.flags.n = wflags.n;
        self.flags.z = wflags.z;
        self.flags.c = wflags.c;
        self.flags.v = wflags.v;
    }

    pub fn check(&self, cond: u16) -> bool {
        let (n, z, c, v) = (self.flags.n, self.flags.z, self.flags.c, self.flags.v);
        match cond {
            0b0000 => z,
            0b0001 => !z,
            0b0010 => n != z,
            0b0011 => z || (n != v),
            0b0100 => !z && (n == v),
            0b0101 => n == v,
            0b0110 => !c,
            0b0111 => !c || !z,
            0b1000 => c && !z,
            0b1001 => c,
            0b1010 => n,
            0b1011 => !n,
            0b1100 => v,
            0b1101 => !v,
            0b1110 => true,
            0b1111 => false,
            _ => panic!(),
        }
    }
}

// bit utils
fn is_word_aligned(addr: u16) -> bool {
    addr % 2 == 0
}

fn get_lsb(data: u16) -> u8 {
    data as u8
}

fn get_msb(data: u16) -> u8 {
    (data >> 8) as u8
}

fn into_word(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_lsb() {
        assert_eq!(get_lsb(0x1234), 0x34);
    }

    #[test]
    fn test_get_msb() {
        assert_eq!(get_msb(0x1234), 0x12);
    }

    #[test]
    fn test_into_word() {
        assert_eq!(into_word(0x12, 0x34), 0x1234);
    }
}
