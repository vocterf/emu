use crate::CPU;

impl CPU {
    pub fn ld_sp_n16(&mut self) {
        let low = self.fetch_byte();
        let high = self.fetch_byte();
        let n16 = (low as u16) | ((high as u16) << 8);
        self.sp = n16;
    }

    pub fn xor_a(&mut self) {
        self.registers.a = self.registers.a ^ self.registers.a;
        self.registers.f = 0b1000_0000;
    }

    pub fn ld_hl_n16(&mut self) {
        let low = self.fetch_byte();
        let high = self.fetch_byte();
        self.registers.l = low;
        self.registers.h = high;
    }

    pub fn ld_hlptrdecr_a(&mut self) {
        let mut hl = ((self.registers.h as u16) << 8) | (self.registers.l as u16);
        self.write_byte(hl, self.registers.a);
        hl = hl.wrapping_sub(1);
        self.registers.h = ((hl & 0xFF00) >> 8) as u8;
        self.registers.l = (hl & 0x00FF) as u8;
    }

    pub fn jr_nz_e8(&mut self) {
        let e8 = self.fetch_byte();
        let z_flag = (self.registers.f & 0b1000_0000) != 0;
        if !z_flag {
            let offset = e8 as i8 as i32;
            self.pc = (self.pc as i32 + offset) as u16;
        }
    }

    pub fn nop(&mut self) {}
    
    pub fn bit7_h(&mut self) {
        self.registers.f &= 0b0001_0000;
        self.registers.f |= 0b0010_0000;
        let is_set = (self.registers.h & 0b1000_0000) != 0;
        if !is_set {
            self.registers.f |= 0b1000_0000;
        }
    }

    pub fn ld_c_n8(&mut self) {
        let n8 = self.fetch_byte();
        self.registers.c = n8;
    }

    pub fn ld_a_n8(&mut self) {
        let n8 = self.fetch_byte();
        self.registers.a = n8;
    }

    pub fn ldh_cptr_a(&mut self) {
         let address = 0xFF00 + (self.registers.c as u16);
         self.write_byte(address, self.registers.a);
    }
    
    pub fn inc_c(&mut self) {
        let original = self.registers.c;
        let result = original.wrapping_add(1);
        self.registers.c = result;
        self.registers.f &= 0b0001_0000;
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
        if (original & 0x0F) == 0x0F {
            self.registers.f |= 0b0010_0000;
        }
    }

    pub fn ld_hlptr_a(&mut self) {
        let hl = ((self.registers.l as u16) | ((self.registers.h as u16) << 8)) as u16;
        self.write_byte(hl, self.registers.a);
    }

    pub fn ldh_a8ptr_a(&mut self) {
        let a8 = self.fetch_byte();
        let address = 0xFF00 + (a8 as u16);
        self.write_byte(address, self.registers.a);
    }

    pub fn ld_de_n16(&mut self) {
        let low = self.fetch_byte();
        let high = self.fetch_byte();
        self.registers.e = low;
        self.registers.d = high;
    }

    pub fn ld_a_deptr(&mut self) {
        let de = ((self.registers.e as u16) | ((self.registers.d as u16) << 8)) as u16;
        self.registers.a = self.read_byte(de);
    }

    pub fn call_a16(&mut self) {
        let low = self.fetch_byte();
        let high = self.fetch_byte();
        let dest_address = (low as u16) | ((high as u16) << 8);
        let return_address = self.pc;
        let hi_byte = ((return_address & 0xFF00) >> 8) as u8;
        let lo_byte = (return_address & 0x00FF) as u8;
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, hi_byte);
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, lo_byte);
        self.pc = dest_address;
    }

    pub fn ld_c_a(&mut self) {
        self.registers.c = self.registers.a;
    }

    pub fn ld_b_n8(&mut self) {
        let n8 = self.fetch_byte();
        self.registers.b = n8;
    }

    pub fn push_bc(&mut self) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, self.registers.b);
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, self.registers.c);
    }

    pub fn rl_c(&mut self) {
        let original = self.registers.c;
        let old_carry = if (self.registers.f & 0b0001_0000) != 0 { 1 } else { 0 };
        let new_carry = (original & 0b1000_0000) != 0;
        let result = (original << 1) | old_carry;
        self.registers.c = result;
        let mut new_f = 0b0000_0000;
        if result == 0 {
            new_f |= 0b1000_0000;
        }
        if new_carry {
            new_f |= 0b0001_0000;
        }
        self.registers.f = new_f;
    }

    pub fn rla(&mut self) {
        let original = self.registers.a;
        let old_carry = (self.registers.f & 0b0001_0000) >> 4;
        let new_carry = (original & 0b1000_0000) >> 7;
        self.registers.a = (self.registers.a << 1) | old_carry;
        let mut new_f = 0b0000_0000;
        if new_carry != 0 {
            new_f |= 0b0001_0000;
        }
        self.registers.f = new_f;
    }

    pub fn pop_bc(&mut self) {
        let low = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.registers.c = low;
        self.registers.b = high;
    }

    pub fn dec_b(&mut self) {
        let original = self.registers.b;
        let result = original.wrapping_sub(1);
        self.registers.b = result;
        self.registers.f &= 0b0001_0000;
        self.registers.f |= 0b0100_0000;
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
        if (original & 0x0F) == 0x00 {
            self.registers.f |= 0b0010_0000;
        }
    }

    pub fn ld_hlptrinc_a(&mut self) {
        let mut hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        self.write_byte(hl, self.registers.a);
        hl = hl.wrapping_add(1);
        self.registers.l = (hl & 0x00FF) as u8;
        self.registers.h = ((hl & 0xFF00) >> 8) as u8; 
    }

    pub fn inc_hl(&mut self) {
        let mut hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        hl = hl.wrapping_add(1);
        self.registers.l = (hl & 0x00FF) as u8;
        self.registers.h = ((hl & 0xFF00) >> 8) as u8;
    }

    pub fn ret(&mut self) {
        let low = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.pc = (low as u16) | ((high as u16) << 8);
    }

    pub fn inc_de(&mut self) {
        let mut de = (self.registers.e as u16) | ((self.registers.d as u16) << 8);
        de = de.wrapping_add(1);
        self.registers.e = (de & 0x00FF) as u8;
        self.registers.d = ((de & 0xFF00) >> 8) as u8;
    }

    pub fn ld_a_e(&mut self) {
        self.registers.a = self.registers.e;
    }

    pub fn cp_a_n8(&mut self) {
        let n8 = self.fetch_byte();
        let a = self.registers.a;
        let mut new_f = 0b0100_0000;
        if a == n8 {
            new_f |= 0b1000_0000;
        }
        if (a & 0x0F) < (n8 & 0x0F) {
            new_f |= 0b0010_0000;
        }
        if a < n8 {
            new_f |= 0b0001_0000;
        }
        self.registers.f = new_f;
    }

    pub fn ld_a16ptr_a(&mut self) {
        let low = self.fetch_byte();
        let high = self.fetch_byte();
        let a16 = (low as u16) | ((high as u16) << 8);
        self.write_byte(a16, self.registers.a);
    }

    pub fn dec_a(&mut self) {
        let original = self.registers.a;
        let result = original.wrapping_sub(1);
        self.registers.a = result;
        self.registers.f &= 0b0001_0000;
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
        self.registers.f |= 0b0100_0000;
        if (original & 0x0F) == 0x00 {
            self.registers.f |= 0b0010_0000;
        } 
    }

    pub fn jr_z_e8(&mut self) {
        let e8 = self.fetch_byte();
        let address = (self.pc as i32 + e8 as i8 as i32) as u16;
        let z_flag = (self.registers.f & 0b1000_0000) != 0;
        if z_flag {
            self.pc = address;
        }
    }

    pub fn ld_h_a(&mut self) {
        self.registers.h = self.registers.a;
    }

    pub fn ld_d_a(&mut self) {
        self.registers.d = self.registers.a;
    }

    pub fn inc_b(&mut self) {
        let original = self.registers.b;
        let result = original.wrapping_add(1);
        self.registers.b = result;
        self.registers.f &= 0b0001_0000;
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
        if (original & 0x0F) == 0x0F {
            self.registers.f |= 0b0010_0000;
        }
    }

    pub fn ld_e_n8(&mut self) {
        let n8 = self.fetch_byte();
        self.registers.e = n8;
    }

    pub fn ldh_a_a8ptr(&mut self) {
        let a8 = self.fetch_byte();
        let address = 0xFF00 + (a8 as u16);
        self.registers.a = self.read_byte(address);
    }

    pub fn dec_c(&mut self) {
        let original = self.registers.c;
        let result = original.wrapping_sub(1);
        self.registers.c = result;
        self.registers.f &= 0b0001_0000;
        if (original & 0x0F) == 0x00 {
            self.registers.f |= 0b0010_0000;
        }
        self.registers.f |= 0b0100_0000;
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
    }

    pub fn ld_l_n8(&mut self) {
        let n8 = self.fetch_byte();
        self.registers.l = n8;
    }

    pub fn jr_e8(&mut self) {
        let e8 = self.fetch_byte();
        let address = (self.pc as i32 + e8 as i8 as i32) as u16;
        self.pc = address;
    }

    pub fn dec_e(&mut self) {
        let original = self.registers.e;
        let result = original.wrapping_sub(1);
        self.registers.e = result;
        self.registers.f &= 0b0001_0000;
        self.registers.f |= 0b0100_0000;
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
        if (original & 0x0F) == 0x00 {
            self.registers.f |= 0b0010_0000;
        }
    }

    pub fn inc_h(&mut self) {
        let original = self.registers.h;
        let result = original.wrapping_add(1);
        self.registers.h = result;
        self.registers.f &= 0b0001_0000;
        if (original & 0x0F) == 0x0F {
            self.registers.f |= 0b0010_0000;
        }
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
    }

    pub fn ld_a_h(&mut self) {
        self.registers.a = self.registers.h;
    }

    pub fn sub_a_b(&mut self) {
        let a = self.registers.a;
        let b = self.registers.b;
        let result = a.wrapping_sub(b);
        self.registers.a = result;
        let mut new_f = 0b0100_0000;
        if a < b {
            new_f |= 0b0001_0000;
        }
        if (a & 0x0F) < (b & 0x0F) {
            new_f |= 0b0010_0000;
        }
        if self.registers.a == 0 {
            new_f |= 0b1000_0000;
        }
        self.registers.f = new_f;
    }

    pub fn dec_d(&mut self) {
        let original = self.registers.d;
        let result = original.wrapping_sub(1);
        self.registers.d = result;
        self.registers.f &= 0b0001_0000;
        self.registers.f |= 0b0100_0000;
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
        if (original & 0x0F) == 0x00 {
            self.registers.f |= 0b0010_0000;
        }
    }

    pub fn ld_d_n8(&mut self) {
        let n8 = self.fetch_byte();
        self.registers.d = n8;
    }

    pub fn cp_a_hlptr(&mut self) {
        let a = self.registers.a;
        let hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        let val = self.read_byte(hl);
        let mut new_f = 0b0100_0000;
        if a < val {
            new_f |= 0b0001_0000;
        }
        if (a & 0x0F) < (val & 0x0F) {
            new_f |= 0b0010_0000;
        }
        if a == val {
            new_f |= 0b1000_0000;
        }
        self.registers.f = new_f;
    }

    pub fn bit_7_l(&mut self) {
        let bit = (self.registers.l & 0b1000_0000) >> 7;
        self.registers.f &= 0b0001_0000;
        self.registers.f |= 0b0010_0000;
        if bit == 0 {
            self.registers.f |= 0b1000_0000;
        }
    }

    pub fn ld_a_l(&mut self) {
        self.registers.a = self.registers.l;
    }

    pub fn ld_a_b(&mut self) {
        self.registers.a = self.registers.b;
    }

    pub fn add_a_hlptr(&mut self) {
        let original = self.registers.a;
        let hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        let val = self.read_byte(hl);
        let mut new_f = 0b0000_0000;
        if ((original as u16) + (val as u16)) > 0xFF {
            new_f |= 0b0001_0000;
        }
        if (original & 0x0F) + (val & 0x0F) > 0x0F {
            new_f |= 0b0010_0000;
        }
        let result = original.wrapping_add(val);
        if result == 0 {
            new_f |= 0b1000_0000;
        }
        self.registers.f = new_f;
        self.registers.a = result;
    }

    pub fn di(&mut self) {
        self.ime = false;
    }

    pub fn jp_a16(&mut self) {
        let low = self.fetch_byte();
        let high = self.fetch_byte();
        let address = (low as u16) | ((high as u16) << 8);
        self.pc = address;
    }

    pub fn ld_hlptr_n8(&mut self) {
        let hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        let n8 = self.fetch_byte();
        self.write_byte(hl, n8);
    }

    pub fn ld_a_hlptrinc(&mut self) {
        let mut hl = (self.registers.l as u16) | ((self.registers.h as u16)  << 8);
        self.registers.a = self.read_byte(hl);
        hl = hl.wrapping_add(1);
        self.registers.l = (hl & 0x00FF) as u8;
        self.registers.h = ((hl & 0xFF00) >> 8) as u8;
    }

    pub fn ld_bc_n16(&mut self) {
        let low = self.fetch_byte();
        let high = self.fetch_byte();
        self.registers.b = high;
        self.registers.c = low;
    }

    pub fn dec_bc(&mut self) {
        let mut bc = (self.registers.c as u16) | ((self.registers.b as u16) << 8);
        bc = bc.wrapping_sub(1);
        self.registers.c = (bc & 0x00FF) as u8;
        self.registers.b = ((bc & 0xFF00) >> 8) as u8;
    }

    pub fn or_a_c(&mut self) {
        let mut new_f = 0b0000_0000;
        self.registers.a |= self.registers.c;
        if self.registers.a == 0 {
            new_f |= 0b1000_0000;
        }
        self.registers.f = new_f;
    }

    pub fn ei(&mut self) {
        self.ime = true;
    }
    
    pub fn cpl(&mut self) {
        self.registers.a = !self.registers.a;
        self.registers.f &= 0b1001_0000;
        self.registers.f |= 0b0110_0000;
    }

    pub fn and_a_n8(&mut self) {
        let n8 = self.fetch_byte();
        self.registers.a &= n8;
        let mut new_f = 0b0010_0000;
        if self.registers.a == 0 {
            new_f |= 0b1000_0000;
        }
        self.registers.f = new_f;
    }

    pub fn swap_a(&mut self) {
        let value = self.registers.a;
        let result = (value >> 4) | (value << 4);
        self.registers.a = result;
        self.registers.f &= 0b0000_0000;
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
    }

    pub fn ld_b_a(&mut self) {
        self.registers.b = self.registers.a;
    }

    pub fn or_a_b(&mut self) {
        self.registers.a |= self.registers.b;
        self.registers.f &= 0b0000_0000;
        if self.registers.a == 0 {
            self.registers.f |= 0b1000_0000;
        }
    }

    pub fn xor_a_c(&mut self) {
        self.registers.a ^= self.registers.c;
        self.registers.f &= 0b0000_0000;
        if self.registers.a == 0 {
            self.registers.f |= 0b1000_0000;
        }
    }

    pub fn and_a_c(&mut self) {
        self.registers.a &= self.registers.c;
        let mut new_f = 0b0010_0000;
        if self.registers.a == 0 {
            new_f |= 0b1000_0000;
        }
        self.registers.f = new_f;
    }

    pub fn ld_a_c(&mut self) {
        self.registers.a = self.registers.c;
    }

    pub fn rst_28(&mut self) {
        let return_address = self.pc;
        let lo_byte = (return_address & 0x00FF) as u8;
        let hi_byte = ((return_address & 0xFF00) >> 8) as u8;
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, hi_byte);
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, lo_byte);
        self.pc = 0x0028;
    }
 
    pub fn add_a_a(&mut self) {
        let original = self.registers.a;
        let result = original.wrapping_add(original);
        let mut new_f = 0b0000_0000;
        if ((original as u16) + (original as u16)) > 0xFF {
            new_f |= 0b0001_0000;
        }
        if ((original & 0x0F) + (original & 0x0F)) > 0x0F {
            new_f |= 0b0010_0000;
        }
        if result == 0 {
            new_f |= 0b1000_0000;
        }
        self.registers.f = new_f;
        self.registers.a = result;
    }

    pub fn pop_hl(&mut self) {
        let low = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.registers.l = low;
        self.registers.h = high;
    }

    pub fn ld_e_a(&mut self) {
        self.registers.e = self.registers.a;
    }

    pub fn add_hl_de(&mut self) {
        let hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        let de = (self.registers.e as u16) | ((self.registers.d as u16) << 8);
        let result = hl.wrapping_add(de);
        self.registers.f &= 0b1000_0000;
        if (hl as u32) + (de as u32) > 0xFFFF {
            self.registers.f |= 0b0001_0000;
        }
        if (hl & 0x0FFF) + (de & 0x0FFF) > 0x0FFF {
            self.registers.f |= 0b0010_0000;
        }
        self.registers.l = (result & 0x00FF) as u8;
        self.registers.h = ((result & 0xFF00) >> 8) as u8;
    }

    pub fn ld_e_hlptr(&mut self) {
        let hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        let val = self.read_byte(hl);
        self.registers.e = val;
    }

    pub fn ld_d_hlptr(&mut self) {
        let hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        let val = self.read_byte(hl);
        self.registers.d = val;
    }

    pub fn push_de(&mut self) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, self.registers.d);
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, self.registers.e);
    }

    pub fn jp_hl(&mut self) {
        let hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        self.pc = hl; 
    }

    pub fn res_0_a(&mut self) {
        self.registers.a &= 0b1111_1110;
    }

    pub fn ld_deptr_a(&mut self) {
        let de = (self.registers.e as u16) | ((self.registers.d as u16) << 8);
        self.write_byte(de, self.registers.a);
    }

    pub fn push_hl(&mut self) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, self.registers.h);
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, self.registers.l);
    }

    pub fn pop_de(&mut self) {
        let low = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.registers.d = high;
        self.registers.e = low;
    }

    pub fn push_af(&mut self) {
        let low = self.registers.f;
        let high = self.registers.a;
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, high);
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, low);
    }

    pub fn ld_a_a16ptr(&mut self) {
        let low = self.fetch_byte();
        let high = self.fetch_byte();
        let a16 = (low as u16) | ((high as u16) << 8);
        self.registers.a = self.read_byte(a16);
    }

    pub fn and_a_a(&mut self) {
        self.registers.a &= self.registers.a;
        let mut new_f = 0b0010_0000;
        if self.registers.a == 0 {
            new_f |= 0b1000_0000;
        }
        self.registers.f = new_f;
    }

    pub fn inc_e(&mut self) {
        let original = self.registers.e;
        self.registers.e = self.registers.e.wrapping_add(1);
        self.registers.f &= 0b0001_0000;
        if (original & 0x0F) == 0x0F {
            self.registers.f |= 0b0010_0000;
        }
        if self.registers.e == 0 {
            self.registers.f |= 0b1000_0000;
        }
    }

    pub fn jp_z_a16(&mut self) {
        let low = self.fetch_byte();
        let high = self.fetch_byte();
        let a16 = (low as u16) | ((high as u16) << 8);
        if (self.registers.f & 0b1000_0000) != 0 {
            self.pc = a16;
        }
    }

    pub fn ret_z(&mut self) {
        let is_set = (self.registers.f & 0b1000_0000) != 0;
        if is_set {
            let low = self.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            let high = self.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            self.pc = (low as u16) | ((high as u16) << 8);
        }
    }

    pub fn ld_a_hlptr(&mut self) {
        let hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        let val = self.read_byte(hl);
        self.registers.a = val;
    }

    pub fn pop_af(&mut self) {
        let low = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.registers.f = low & 0xF0;
        self.registers.a = high;
    }

    pub fn ret_nz(&mut self) {
        let is_set = (self.registers.f & 0b1000_0000) != 0;
        if !is_set {
            let low = self.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            let high = self.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            self.pc = (low as u16) | ((high as u16) << 8);
        }
    }

    pub fn ret_c(&mut self) {
        let is_set = (self.registers.f & 0b0001_0000) != 0;
        if is_set {
            let low = self.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            let high = self.read_byte(self.sp);
            self.sp = self.sp.wrapping_add(1);
            self.pc = (low as u16) | ((high as u16) << 8);
        }
    }

    pub fn rrca(&mut self) {
        let original = self.registers.a;
        let bit_0 = original & 0b0000_0001;
        self.registers.a = (original >> 1) | (bit_0 << 7);
        let mut new_f = 0b0000_0000;
        if bit_0 != 0 {
            new_f |= 0b0001_0000;
        }
        self.registers.f = new_f;
    }

    pub fn or_a_hlptr(&mut self) {
        let hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        let val = self.read_byte(hl);
        self.registers.a |= val;
        let mut new_f = 0b0000_0000;
        if self.registers.a == 0 {
            new_f |= 0b1000_0000;
        }
        self.registers.f = new_f;
    }

    pub fn ld_b_b(&mut self) {
        self.registers.b = self.registers.b;
    }

    pub fn inc_d(&mut self) {
        let original = self.registers.d;
        let result = self.registers.d.wrapping_add(1);
        self.registers.f &= 0b0001_0000;
        if (original & 0x0F) == 0x0F {
            self.registers.f |= 0b0010_0000;
        }
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
        self.registers.d = result;
    }

    pub fn ld_bcptr_a(&mut self) {
        let bc = (self.registers.c as u16) | ((self.registers.b as u16) << 8);
        self.write_byte(bc, self.registers.a);
    }

    pub fn add_sp_e8(&mut self) {
        let e8 = self.fetch_byte();
        let original = self.sp;
        self.sp = original.wrapping_add(e8 as i8 as u16);
        let mut new_f = 0b0000_0000;
        if (original & 0x00FF) + (e8 as u16) > 0x00FF {
            new_f |= 0b0001_0000;
        }
        if (original & 0x000F) + ((e8 & 0x0F) as u16) > 0x000F {
            new_f |= 0b0010_0000;
        }
        self.registers.f = new_f;
    }

    pub fn inc_l(&mut self) {
        let original = self.registers.l;
        let result = self.registers.l.wrapping_add(1);
        self.registers.f &= 0b0001_0000;
        if (original & 0x0F) == 0x0F {
            self.registers.f |= 0b0010_0000;
        }
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
        self.registers.l = result;
    }

    pub fn bit0_c(&mut self) {
        self.registers.f &= 0b0001_0000;
        self.registers.f |= 0b0010_0000;
        let is_set = (self.registers.c & 0b0000_0001) != 0;
        if !is_set {
            self.registers.f |= 0b1000_0000;
        }
    }

    pub fn add_a_d(&mut self) {
        let mut new_f = 0b0000_0000;
        let result = self.registers.a.wrapping_add(self.registers.d);
        if ((self.registers.a as u16) + (self.registers.d as u16)) > 0xFF {
            new_f |= 0b0001_0000;
        }
        if ((self.registers.a & 0x0F) + (self.registers.d & 0x0F)) > 0x0F {
            new_f |= 0b0010_0000;
        }
        if result == 0 {
            new_f |= 0b1000_0000;
        }
        self.registers.a = result;
        self.registers.f = new_f;
    }

    pub fn inc_hlptr(&mut self) {
        let hl = (self.registers.l as u16) | ((self.registers.h as u16) << 8);
        let val = self.read_byte(hl);
        let result = val.wrapping_add(1);
        self.registers.f &= 0b0001_0000;
        if (val & 0x0F) == 0x0F {
            self.registers.f |= 0b0010_0000;
        }
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
        self.write_byte(hl, result);
    }

    pub fn inc_a(&mut self) {
        let original = self.registers.a;
        let result = self.registers.a.wrapping_add(1);
        self.registers.f &= 0b0001_0000;
        if (original & 0x0F) == 0x0F {
            self.registers.f |= 0b0010_0000;
        }
        if result == 0 {
            self.registers.f |= 0b1000_0000;
        }
        self.registers.a = result;
    }

    pub fn reti(&mut self) {
        let low = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.pc = (low as u16) | ((high as u16) << 8);
        self.ime = true;
    }

    pub fn ld_a_bcptr(&mut self) {
        let bc = (self.registers.c as u16) | ((self.registers.b as u16) << 8);
        let val = self.read_byte(bc);
        self.registers.a = val;
    }

    pub fn add_a_h(&mut self) {
        let mut new_f = 0b0000_0000;
        let result = self.registers.a.wrapping_add(self.registers.h);
        if ((self.registers.a as u16) + (self.registers.h as u16)) > 0xFF {
            new_f |= 0b0001_0000;
        }
        if ((self.registers.a & 0x0F) + (self.registers.h & 0x0F)) > 0x0F {
            new_f |= 0b0010_0000;
        }
        if result == 0 {
            new_f |= 0b1000_0000;
        }
        self.registers.a = result;
        self.registers.f = new_f;
    }
}