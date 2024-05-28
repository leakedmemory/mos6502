use crate::memory::Memory;

const CSF_ZERO: u8 = 0x02;
const CSF_NEGATIVE: u8 = 0x80;

const SYS_STACK_ADDR_END: u16 = 0x100;
pub(crate) const POWER_ON_RESET_ADDR_L: u16 = 0xFFFC;
pub(crate) const POWER_ON_RESET_ADDR_H: u16 = 0xFFFD;

const CPU_DEFAULT_ACC: u8 = 0;
const CPU_DEFAULT_X: u8 = 0;
const CPU_DEFAULT_Y: u8 = 0;
const CPU_DEFAULT_SP: u8 = 0xFF;
const CPU_DEFAULT_STATUS: u8 = 0x20;

// ==================== OPCODES START ====================

// JSR
const OPCODE_JSR: u8 = 0x20;

// LDA
const OPCODE_LDA_IMM: u8 = 0xA9;
const OPCODE_LDA_ZPG: u8 = 0xA5;
const OPCODE_LDA_ZPX: u8 = 0xB5;
const OPCODE_LDA_ABS: u8 = 0xAD;
const OPCODE_LDA_ABX: u8 = 0xBD;
const OPCODE_LDA_ABY: u8 = 0xB9;
const OPCODE_LDA_IDX: u8 = 0xA1;

// ==================== OPCODES END =====================

pub struct CPU {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    status: u8,
    cycles: u64,
    memory: Memory,
}

impl CPU {
    pub fn new(memory: Memory) -> Self {
        Self {
            acc: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0,
            status: 0,
            cycles: 0,
            memory,
        }
    }

    pub fn reset(&mut self) {
        self.acc = CPU_DEFAULT_ACC;
        self.x = CPU_DEFAULT_X;
        self.y = CPU_DEFAULT_Y;
        self.sp = CPU_DEFAULT_SP;
        self.pc = ((self.memory.get(POWER_ON_RESET_ADDR_H) as u16) << 8)
            | (self.memory.get(POWER_ON_RESET_ADDR_L) as u16);
        self.status = CPU_DEFAULT_STATUS;
        self.cycles = 7;
    }

    pub fn execute(&mut self, opcode: u8) {
        match opcode {
            OPCODE_JSR => self.jsr(),
            OPCODE_LDA_IMM => self.lda_immediate(),
            OPCODE_LDA_ZPG => self.lda_zero_page(),
            OPCODE_LDA_ZPX => self.lda_zero_page_x(),
            OPCODE_LDA_ABS => self.lda_absolute(),
            OPCODE_LDA_ABX => self.lda_absolute_x(),
            OPCODE_LDA_ABY => self.lda_absolute_y(),
            OPCODE_LDA_IDX => self.lda_indirect_x(),
            _ => panic!("invalid opcode: {:#X}", opcode),
        }
    }

    /// gets a byte from program counter and increments it in 1 cycle
    pub fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.get(self.pc);
        self.increment_pc();
        self.cycles += 1;
        byte
    }

    #[inline]
    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    /// gets a byte from addr in 1 cycle
    fn read_byte(&mut self, addr: u16) -> u8 {
        let byte = self.memory.get(addr);
        self.cycles += 1;
        byte
    }

    /// gets an addr from the program counter and increments it by 2,
    /// in 2 cycles
    fn fetch_addr(&mut self) -> u16 {
        let addr_l = self.fetch_byte() as u16;
        let addr_h = self.fetch_byte() as u16;
        (addr_h << 8) | addr_l
    }

    /// pushes an addr to the stack, wrapping around when overflowing or
    /// underflowing, in 2 cycles
    fn push_addr_to_stack(&mut self, addr: u16) {
        let mut sp = (self.sp as u16) | SYS_STACK_ADDR_END;
        let addr_l = addr as u8;
        let addr_h = (addr >> 8) as u8;

        self.memory.set(addr_h, sp);
        self.sp = self.sp.wrapping_sub(1);
        sp = (self.sp as u16) | SYS_STACK_ADDR_END;

        self.memory.set(addr_l, sp);
        self.sp = self.sp.wrapping_sub(1);
        self.cycles += 2;
    }

    #[inline(always)]
    fn byte_is_negative_int(byte: u8) -> bool {
        byte & 0x80 != 0
    }

    // often used to know the need of another add operation with the high 8 bits
    // of the address, since the 6502's adder circuit only works with 8 bits
    #[inline(always)]
    fn page_crossed(addr_a: u16, addr_b: u16) -> bool {
        (addr_a & 0xFF00) != (addr_b & 0xFF00)
    }

    // ==================== JSR ====================

    /// bytes: 3
    /// cycles: 6
    /// flags affected: none
    fn jsr(&mut self) {
        let addr = self.fetch_addr();
        self.push_addr_to_stack(self.pc - 1);
        self.pc = addr; // takes 1 cycle
        self.cycles += 1;
    }

    // ==================== LDA ====================

    fn lda_set_status(&mut self) {
        self.status &= !(CSF_ZERO | CSF_NEGATIVE);
        if self.acc == 0 {
            self.status |= CSF_ZERO;
        } else if Self::byte_is_negative_int(self.acc) {
            self.status |= CSF_NEGATIVE;
        }
    }

    /// bytes: 2
    /// cycles: 2
    /// flags affected: N and Z
    fn lda_immediate(&mut self) {
        self.acc = self.fetch_byte();
        self.lda_set_status();
    }

    /// bytes: 2
    /// cycles: 3
    /// flags affected: N and Z
    fn lda_zero_page(&mut self) {
        let addr = self.fetch_byte();
        self.acc = self.read_byte(addr.into());
        self.lda_set_status();
    }

    /// bytes: 2
    /// cycles: 4
    /// flags affected: N and Z
    fn lda_zero_page_x(&mut self) {
        let byte = self.fetch_byte();
        let addr = self.x.wrapping_add(byte);
        self.cycles += 1;
        self.acc = self.read_byte(addr.into());
        self.lda_set_status();
    }

    /// bytes: 3
    /// cycles: 4
    /// flags affected: N and Z
    fn lda_absolute(&mut self) {
        let addr = self.fetch_addr();
        self.acc = self.read_byte(addr);
        self.lda_set_status();
    }

    /// bytes: 3
    /// cycles: 4 (+1 if page crossed)
    /// flags affected: N and Z
    fn lda_absolute_x(&mut self) {
        let abs_addr = self.fetch_addr();
        let eff_addr = abs_addr.wrapping_add(self.x.into());
        if Self::page_crossed(abs_addr, eff_addr) {
            self.cycles += 1;
        }
        self.acc = self.read_byte(eff_addr);
        self.lda_set_status();
    }

    /// bytes: 3
    /// cycles: 4 (+1 if page crossed)
    /// flags affected: N and Z
    fn lda_absolute_y(&mut self) {
        let abs_addr = self.fetch_addr();
        let eff_addr = abs_addr.wrapping_add(self.y.into());
        if Self::page_crossed(abs_addr, eff_addr) {
            self.cycles += 1;
        }
        self.acc = self.read_byte(eff_addr);
        self.lda_set_status();
    }

    /// bytes: 2
    /// cycles: 6
    /// flags affected: N and Z
    fn lda_indirect_x(&mut self) {
        let zpg_addr = self.fetch_byte();
        let eff_zpg_addr = zpg_addr.wrapping_add(self.x);
        self.cycles += 1;

        let eff_addr_l = self.read_byte(eff_zpg_addr.into());
        let eff_addr_h = self.read_byte(eff_zpg_addr.wrapping_add(1).into());
        let eff_addr = (eff_addr_h as u16) << 8 | eff_addr_l as u16;
        self.acc = self.read_byte(eff_addr);
        self.lda_set_status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;

    const UNRESERVED_MEMORY_ADDR_START: u16 = 0x0200;

    #[test]
    fn jsr_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_JSR, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(0x30, MEM_OFFSET + 2);
        memory.set(OPCODE_LDA_IMM, 0x3042);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(cpu.pc, 0x3042);
        assert_eq!(cpu.memory.get(cpu.pc), OPCODE_LDA_IMM);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.sp, CPU_DEFAULT_SP - 2);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let stack_pc_l = cpu.memory.get(((cpu.sp + 1) as u16) | SYS_STACK_ADDR_END) as u16;
        let stack_pc_h = cpu.memory.get(((cpu.sp + 2) as u16) | SYS_STACK_ADDR_END) as u16;
        let stack_pc = (stack_pc_h << 8) | stack_pc_l;
        assert_eq!(stack_pc + 1 - init_pc, BYTES);
    }

    #[test]
    fn lda_immediate_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_IMM, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(OPCODE_LDA_IMM, MEM_OFFSET + 2);
        memory.set(0x00, MEM_OFFSET + 3);
        memory.set(OPCODE_LDA_IMM, MEM_OFFSET + 4);
        memory.set(0x80, MEM_OFFSET + 5);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x42, cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_zero_page_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ZPG, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(0x32, 0x42);
        memory.set(OPCODE_LDA_ZPG, MEM_OFFSET + 2);
        memory.set(0x57, MEM_OFFSET + 3);
        memory.set(0x00, 0x57);
        memory.set(OPCODE_LDA_ZPG, MEM_OFFSET + 4);
        memory.set(0x69, MEM_OFFSET + 5);
        memory.set(0x80, 0x69);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x32, cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_zero_page_x_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ZPX, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(0x32, X.wrapping_add(0x42).into());
        memory.set(OPCODE_LDA_ZPX, MEM_OFFSET + 2);
        memory.set(0x57, MEM_OFFSET + 3);
        memory.set(0x00, X.wrapping_add(0x57).into());
        memory.set(OPCODE_LDA_ZPX, MEM_OFFSET + 4);
        memory.set(0x69, MEM_OFFSET + 5);
        memory.set(0x80, X.wrapping_add(0x69).into());

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x32, cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ABS, MEM_OFFSET);
        memory.set(0x28, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, 0x8028);
        memory.set(OPCODE_LDA_ABS, MEM_OFFSET + 3);
        memory.set(0x97, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, 0x2697);
        memory.set(OPCODE_LDA_ABS, MEM_OFFSET + 6);
        memory.set(0x70, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);
        memory.set(0x80, 0x5570);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x42, cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_x_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET);
        memory.set(0x28, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, (X as u16).wrapping_add(0x8028));
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET + 3);
        memory.set(0x53, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, (X as u16).wrapping_add(0x2653));
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET + 6);
        memory.set(0x22, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);
        memory.set(0x80, (X as u16).wrapping_add(0x5522));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x42, cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_x_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET);
        memory.set(0x60, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, (X as u16).wrapping_add(0x8060));
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET + 3);
        memory.set(0x54, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, (X as u16).wrapping_add(0x2654));
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET + 6);
        memory.set(0x83, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);
        memory.set(0x80, (X as u16).wrapping_add(0x5583));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x42, cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_y_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET);
        memory.set(0x28, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, (Y as u16).wrapping_add(0x8028));
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET + 3);
        memory.set(0x53, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, (Y as u16).wrapping_add(0x2653));
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET + 6);
        memory.set(0x22, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);
        memory.set(0x80, (Y as u16).wrapping_add(0x5522));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x42, cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_y_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET);
        memory.set(0x60, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, (Y as u16).wrapping_add(0x8060));
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET + 3);
        memory.set(0x54, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, (Y as u16).wrapping_add(0x2654));
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET + 6);
        memory.set(0x83, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);
        memory.set(0x80, (Y as u16).wrapping_add(0x5583));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x42, cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_indirect_x_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        // first instruction
        memory.set(OPCODE_LDA_IDX, MEM_OFFSET);
        let zpg_addr = 0x08;
        memory.set(zpg_addr, MEM_OFFSET + 1);
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);
        memory.set(0x32, addr_h.into());
        memory.set(0x22, addr_l.into());
        memory.set(0x42, 0x3222);
        // second instruction
        memory.set(OPCODE_LDA_IDX, MEM_OFFSET + 2);
        let zpg_addr = 0x57;
        memory.set(zpg_addr, MEM_OFFSET + 3);
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);
        memory.set(0x83, addr_h.into());
        memory.set(0x46, addr_l.into());
        memory.set(0x00, 0x8346);
        // third instruction
        memory.set(OPCODE_LDA_IDX, MEM_OFFSET + 4);
        let zpg_addr = 0x69;
        memory.set(zpg_addr, MEM_OFFSET + 5);
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);
        memory.set(0x77, addr_h.into());
        memory.set(0x13, addr_l.into());
        memory.set(0x80, 0x7713);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x42, cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }
}
