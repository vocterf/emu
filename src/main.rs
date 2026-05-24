use core::panic;
use std::sync::atomic::{AtomicU32, Ordering};

mod instructions;

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, PartialEq, Eq)]
pub struct CPU {
    pub(crate) registers: Registers,
    pub(crate) pc: u16,
    pub(crate) sp: u16,
    pub(crate) memory: [u8; 0x10000]
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

    pub fn load_boot_rom(&mut self, path: &str) {
        let boot_rom_bytes = std::fs::read(path)
            .expect("cannot find boot rom");

        if boot_rom_bytes.len() != 256 {
            panic!("Boot rom has invalid length");
        }

        self.memory[..256].copy_from_slice(&boot_rom_bytes);
        println!("Success. Boot ROM loaded.");
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF44 => 0x90, 
            0x0104..=0x0133 => self.memory[(address - 0x0104 + 0x00A8) as usize],
            0x014D => 0xE7,
            _ => self.memory[address as usize],
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    pub fn fetch_byte(&mut self) -> u8 {
        let byte = self.read_byte(self.pc);
        self.pc += 1;
        byte
    }

    pub fn tick(&mut self) {
        let current_pc = self.pc;
        let opcode = self.read_byte(current_pc);
        let count = COUNTER.load(Ordering::Relaxed);

        if count < 500000 {
            COUNTER.store(count + 1, Ordering::Relaxed);
        } else {
            panic!("DIAGNOSTYKA: Osiągnięto 500 000 kroków! Aktualny PC: {:#06X}, Opcode: {:#04X}", current_pc, opcode);
        }

        let fetched_opcode = self.fetch_byte();
        let prefix_opcode = 0xCB;

        if fetched_opcode == prefix_opcode {
            let cb_opcode = self.fetch_byte();
            self.execute_cb(cb_opcode);
        } else {
            self.execute(fetched_opcode);
        }
    }

    pub fn execute(&mut self, opcode: u8) {
        match opcode {
            0x31 => self.ld_sp_n16(),
            0xAF => self.xor_a(),
            0x21 => self.ld_hl_n16(),
            0x32 => self.ld_hlptrdecr_a(),
            0x20 => self.jr_nz_e8(),
            0x00 => self.nop(),
            0x0E => self.ld_c_n8(),
            0x3E => self.ld_a_n8(),
            0xE2 => self.ldh_cptr_a(),
            0x0C => self.inc_c(),
            0x77 => self.ld_hlptr_a(),
            0xE0 => self.ldh_a8ptr_a(),
            0x11 => self.ld_de_n16(),
            0x1A => self.ld_a_deptr(),
            0xCD => self.call_a16(),
            0x4F => self.ld_c_a(),
            0x06 => self.ld_b_n8(),
            0xC5 => self.push_bc(),
            0x17 => self.rla(),
            0xC1 => self.pop_bc(),
            0x05 => self.dec_b(),
            0x22 => self.ld_hlptrinc_a(),
            0x23 => self.inc_hl(),
            0xC9 => self.ret(),
            0x13 => self.inc_de(),
            0x7B => self.ld_a_e(),
            0xFE => self.cp_a_n8(),
            0xEA => self.ld_a16ptr_a(),
            0x3D => self.dec_a(),
            0x28 => self.jr_z_e8(),
            0x67 => self.ld_h_a(),
            0x57 => self.ld_d_a(),
            0x04 => self.inc_b(),
            0x1E => self.ld_e_n8(),
            0xF0 => self.ldh_a_a8ptr(),
            0x0D => self.dec_c(),
            0x2E => self.ld_l_n8(),
            0x18 => self.jr_e8(),
            0x1D => self.dec_e(),
            0x24 => self.inc_h(),
            0x7C => self.ld_a_h(),
            0x90 => self.sub_a_b(),
            0x15 => self.dec_d(),
            0x16 => self.ld_d_n8(),
            0xBE => self.cp_a_hlptr(),
            _ => panic!(
                "STOP! Nieznany opcode: {:#04X} pod adresem: {:#06X}. Musisz go teraz zaimplementować!", 
                opcode, 
                self.pc - 1
            ),
        }
    } 

    pub fn execute_cb(&mut self, opcode: u8) {
        match opcode {
            0x7C => self.bit7_h(),
            0x11 => self.rl_c(),
            _ => panic!(
                "STOP! Nieznany CB opcode: {:#04X} pod adresem: {:#06X}. Musisz go teraz zaimplementować!", 
                opcode, 
                self.pc - 1
            ),
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
    let mut cpu = CPU::new();
    cpu.load_boot_rom("dmg_boot.bin");
    loop {
        cpu.tick();
    }
}