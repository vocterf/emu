use core::panic;

mod instructions;

#[derive(Debug, PartialEq, Eq)]
pub struct CPU {
    pub(crate) registers: Registers,
    pub(crate) pc: u16,
    pub(crate) sp: u16,
    pub(crate) memory: [u8; 0x10000],
    pub(crate) boot_rom: [u8; 256],
    pub(crate) boot_rom_active: bool,
    pub(crate) ly: u8,
    pub(crate) ly_divider: u32,
    pub(crate) div_counter: u32,
    pub(crate) ime: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: Registers::new(),
            pc: 0x0000,
            sp: 0x0000,
            memory: [0; 0x10000],
            boot_rom: [0; 256],
            boot_rom_active: true,
            ly: 0,
            ly_divider: 0,
            div_counter: 0,
            ime: false,
        }
    }

    pub fn load_boot_rom(&mut self, path: &str) {
        let boot_rom_bytes = std::fs::read(path).expect("cannot find boot rom");

        if boot_rom_bytes.len() != 256 {
            panic!("Boot rom has invalid length");
        }

        self.boot_rom.copy_from_slice(&boot_rom_bytes);
        println!("Success. Boot ROM loaded into secure boot area.");
    }

    pub fn load_game_rom(&mut self, path: &str) {
        let game_bytes = std::fs::read(path).expect("cannot find game rom");

        let size = game_bytes.len().min(0x8000);
        self.memory[..size].copy_from_slice(&game_bytes[..size]);
        println!("Success. Game ROM loaded into memory.");
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        if self.boot_rom_active && address < 0x0100 {
            return self.boot_rom[address as usize];
        }

        match address {
            0xFF41 => {
                // Dynamiczne obliczanie 4 trybów PPU na podstawie licznika cykli linii (ly_divider)
                let base_stat = self.memory[0xFF41] & 0xFC;
                if self.ly >= 144 {
                    base_stat | 0x01 // Linia >= 144 to zawsze Tryb 1: V-Blank
                } else {
                    if self.ly_divider < 80 {
                        base_stat | 0x02 // Pierwsze 80 cykli to Tryb 2: OAM Search
                    } else if self.ly_divider < 252 {
                        base_stat | 0x03 // Kolejne 172 cykle to Tryb 3: LCD Transfer
                    } else {
                        base_stat | 0x00 // Reszta cykli do 456 to Tryb 0: H-Blank
                    }
                }
            }
            0xFF44 => {
                if self.boot_rom_active {
                    0x90
                } else {
                    self.ly
                }
            }
            _ => self.memory[address as usize],
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => {
                self.memory[0xFF04] = 0;
            }
            0xFF02 => {
                if (value & 0x80) != 0 {
                    self.memory[0xFF02] = value & 0x7F;
                } else {
                    self.memory[0xFF02] = value;
                }
            }
            0xFF46 => {
                // IMPLEMENTACJA TRANSFERU OAM DMA:
                // Zapis pod 0xFF46 uruchamia sprzętowe kopiowanie całego bloku danych duszków
                self.memory[0xFF46] = value;
                let source_base = (value as u16) << 8;
                for i in 0..160 {
                    let src_addr = source_base + i;
                    let byte_to_copy = self.read_byte(src_addr);
                    self.memory[0xFE00 + i as usize] = byte_to_copy;
                }
            }
            0xFF50 => {
                if value == 1 {
                    self.boot_rom_active = false;
                    println!("--- BOOT ROM UNMAPPED. GAME CONTROL STARTED ---");
                }
            }
            _ => self.memory[address as usize] = value,
        }
    }

    pub fn fetch_byte(&mut self) -> u8 {
        let byte = self.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    pub fn tick(&mut self) {
        self.div_counter += 1;
        if self.div_counter >= 256 {
            self.div_counter = 0;
            self.memory[0xFF04] = self.memory[0xFF04].wrapping_add(1);
        }

        if !self.boot_rom_active {
            self.ly_divider += 1;
            if self.ly_divider >= 456 {
                self.ly_divider = 0;
                self.ly = (self.ly + 1) % 154;

                if self.ly == 144 {
                    self.memory[0xFF0F] |= 0x01;
                }
            }
        }

        if self.ime {
            let ie = self.memory[0xFFFF];
            let if_reg = self.memory[0xFF0F];
            let pending = ie & if_reg;

            if (pending & 0x01) != 0 {
                self.ime = false;
                self.memory[0xFF0F] &= !0x01;

                let return_address = self.pc;
                self.sp = self.sp.wrapping_sub(1);
                self.write_byte(self.sp, ((return_address & 0xFF00) >> 8) as u8);
                self.sp = self.sp.wrapping_sub(1);
                self.write_byte(self.sp, (return_address & 0x00FF) as u8);

                self.pc = 0x0040;
            }
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
            0x7D => self.ld_a_l(),
            0x86 => self.add_a_hlptr(),
            0xC3 => self.jp_a16(),
            0xF3 => self.di(),
            0x78 => self.ld_a_b(),
            0x36 => self.ld_hlptr_n8(),
            0x2A => self.ld_a_hlptrinc(),
            0x01 => self.ld_bc_n16(),
            0x0A => self.ld_a_bcptr(),
            0x0B => self.dec_bc(),
            0xB1 => self.or_a_c(),
            0xFB => self.ei(),
            0x2F => self.cpl(),
            0xE6 => self.and_a_n8(),
            0x47 => self.ld_b_a(),
            0xB0 => self.or_a_b(),
            0xA9 => self.xor_a_c(),
            0xA1 => self.and_a_c(),
            0x79 => self.ld_a_c(),
            0xEF => self.rst_28(),
            0x87 => self.add_a_a(),
            0xE1 => self.pop_hl(),
            0x5F => self.ld_e_a(),
            0x19 => self.add_hl_de(),
            0x5E => self.ld_e_hlptr(),
            0x56 => self.ld_d_hlptr(),
            0xD5 => self.push_de(),
            0xE9 => self.jp_hl(),
            0x12 => self.ld_deptr_a(),
            0xE5 => self.push_hl(),
            0xD1 => self.pop_de(),
            0xF5 => self.push_af(),
            0xFA => self.ld_a_a16ptr(),
            0xA7 => self.and_a_a(),
            0x1C => self.inc_e(),
            0xCA => self.jp_z_a16(),
            0xC8 => self.ret_z(),
            0x7E => self.ld_a_hlptr(),
            0xF1 => self.pop_af(),
            0xC0 => self.ret_nz(),
            0xD8 => self.ret_c(),
            0x0F => self.rrca(),
            0xB6 => self.or_a_hlptr(),
            0x40 => self.ld_b_b(),
            0x14 => self.inc_d(),
            0x02 => self.ld_bcptr_a(),
            0xE8 => self.add_sp_e8(),
            0x2C => self.inc_l(),
            0x82 => self.add_a_d(),
            0x84 => self.add_a_h(),
            0x34 => self.inc_hlptr(),
            0x3C => self.inc_a(),
            0xD9 => self.reti(),
            _ => panic!(
                "STOP! Nieznany opcode: {:#04X} pod adresem: {:#06X}. Pora go zaimplementować!",
                opcode,
                self.pc - 1
            ),
        }
    }

    pub fn execute_cb(&mut self, opcode: u8) {
        match opcode {
            0x7C => self.bit7_h(),
            0x7D => self.bit_7_l(),
            0x11 => self.rl_c(),
            0x37 => self.swap_a(),
            0x87 => self.res_0_a(),
            0x41 => self.bit0_c(),
            _ => panic!(
                "STOP! Nieznany CB opcode: {:#04X} pod adresem: {:#06X}.",
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
        Registers {
            a: 0x00,
            f: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
        }
    }
}

fn main() {
    let mut cpu = CPU::new();
    cpu.load_boot_rom("dmg_boot.bin");
    cpu.load_game_rom("tetris.gb");
    loop {
        cpu.tick();
    }
}