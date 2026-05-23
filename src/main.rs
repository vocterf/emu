

#[derive(Debug, PartialEq, Eq)]
pub struct CPU {
    registers: Registers,
    pc: u16,
    sp: u16,
    memory: [u8; 0x10000]
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: Registers::new(),
            pc: 0x0000,
            sp: 0x0000,
            memory: [0; 0x10000]
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
}

impl Registers {
    pub fn new() -> Self {
        Registers { a: 0x00, f: 0x00, b: 0x00, c: 0x00, d: 0x00, e: 0x00, h: 0x00, l: 0x00 }
    }
}


fn main() {
    let cpu = CPU::new();
}