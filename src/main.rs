

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPU {
    v: [u8; 16],
    i: u16,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    memory: [u8; 4096]
}

impl CPU {
    fn new() -> Self {
        CPU {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            memory: [0; 4096]
        }
    }
}

fn main() {
    println!("Hello, world!");
}
