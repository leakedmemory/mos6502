use crate::memory::Memory;

pub const CSF_CARRY: u8 = 0x01;
pub const CSF_ZERO: u8 = 0x02;
pub const CSF_INTERRUPT_DISABLE: u8 = 0x04;
pub const CSF_DECIMAL_MODE: u8 = 0x08;
pub const CSF_BREAK_COMMAND: u8 = 0x10;
pub const CSF_OVERFLOW: u8 = 0x40;
pub const CSF_NEGATIVE: u8 = 0x80;

#[allow(dead_code)]
const UNRESERVED_MEMORY_ADDR_START: u16 = 0x0200; // used only in tests for now
pub(crate) const POWER_ON_RESET_ADDR_L: u16 = 0xFFFC;
pub(crate) const POWER_ON_RESET_ADDR_H: u16 = 0xFFFD;

const CPU_DEFAULT_ACC: u8 = 0;
const CPU_DEFAULT_X: u8 = 0;
const CPU_DEFAULT_Y: u8 = 0;
const CPU_DEFAULT_SP: u8 = 0xFF;
const CPU_DEFAULT_STATUS: u8 = 0x20;

const OPCODE_LDA_IMM: u8 = 0xA9;
const OPCODE_LDA_ZPG: u8 = 0xA5;
const OPCODE_LDA_ZPX: u8 = 0xB5;

/// ps register: NV1B DIZC
pub struct CPU<'m> {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    status: u8,
    cycles: u64,
    memory: &'m mut Memory,
}

impl<'m> CPU<'m> {
    pub fn new(memory: &'m mut Memory) -> Self {
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

    #[inline]
    pub fn flag_is_set(&self, flag: u8) -> bool {
        self.status & flag != 0
    }

    pub fn execute(&mut self, opcode: u8) {
        match opcode {
            OPCODE_LDA_IMM => self.lda_immediate(),
            OPCODE_LDA_ZPG => self.lda_zero_page(),
            OPCODE_LDA_ZPX => self.lda_zero_page_x(),
            _ => panic!("invalid opcode: {:#X}", opcode),
        }
    }

    /// gets a byte from program counter and increments it consuming 1 cycle
    pub fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.get(self.pc);
        self.increment_pc();
        self.cycles += 1;
        byte
    }

    /// gets a byte from addr consuming 1 cycle
    fn read_byte(&mut self, addr: u16) -> u8 {
        let byte = self.memory.get(addr);
        self.cycles += 1;
        byte
    }

    #[inline]
    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    fn lda_set_status(&mut self, byte: u8) {
        if byte == 0 {
            self.status |= CSF_ZERO;
            self.status &= !CSF_NEGATIVE;
        } else if Self::byte_is_negative_int(byte) {
            self.status |= CSF_NEGATIVE;
            self.status &= !CSF_ZERO;
        }
    }

    /// bytes: 2
    /// cycles: 2
    /// flags affected: N and Z
    fn lda_immediate(&mut self) {
        let acc = self.fetch_byte();
        self.acc = acc;
        self.lda_set_status(acc);
    }

    /// bytes: 2
    /// cycles: 3
    /// flags affected: N and Z
    fn lda_zero_page(&mut self) {
        let addr = self.fetch_byte();
        let acc = self.read_byte(addr.into());
        self.acc = acc;
        self.lda_set_status(acc);
    }

    /// bytes: 2
    /// cycles: 4
    /// flags affected: N and Z
    fn lda_zero_page_x(&mut self) {
        let byte = self.fetch_byte();
        let addr = self.x.wrapping_add(byte);
        self.cycles += 1;
        let acc = self.read_byte(addr.into());
        self.acc = acc;
        self.lda_set_status(acc);
    }

    #[inline(always)]
    fn byte_is_negative_int(byte: u8) -> bool {
        byte & 0x80 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;

    #[test]
    fn lda_immediate_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEMORY_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_IMM, MEMORY_OFFSET);
        memory.set(0x42, MEMORY_OFFSET + 1);
        memory.set(OPCODE_LDA_IMM, MEMORY_OFFSET + 2);
        memory.set(0x00, MEMORY_OFFSET + 3);
        memory.set(OPCODE_LDA_IMM, MEMORY_OFFSET + 4);
        memory.set(0x80, MEMORY_OFFSET + 5);

        let mut cpu = CPU::new(&mut memory);
        cpu.reset();

        let mut saved_pc = cpu.pc;
        let mut saved_cycles = cpu.cycles;
        let mut opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x42, cpu.acc);
        assert_eq!(cpu.pc - saved_pc, BYTES);
        assert_eq!(cpu.cycles - saved_cycles, CYCLES);
        assert!(!cpu.flag_is_set(CSF_ZERO));
        assert!(!cpu.flag_is_set(CSF_NEGATIVE));

        saved_pc = cpu.pc;
        saved_cycles = cpu.cycles;
        opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - saved_pc, BYTES);
        assert_eq!(cpu.cycles - saved_cycles, CYCLES);
        assert!(cpu.flag_is_set(CSF_ZERO));
        assert!(!cpu.flag_is_set(CSF_NEGATIVE));

        saved_pc = cpu.pc;
        saved_cycles = cpu.cycles;
        opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - saved_pc, BYTES);
        assert_eq!(cpu.cycles - saved_cycles, CYCLES);
        assert!(!cpu.flag_is_set(CSF_ZERO));
        assert!(cpu.flag_is_set(CSF_NEGATIVE));
    }

    #[test]
    fn lda_zero_page_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEMORY_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ZPG, MEMORY_OFFSET);
        memory.set(0x42, MEMORY_OFFSET + 1);
        memory.set(0x32, 0x42);
        memory.set(OPCODE_LDA_ZPG, MEMORY_OFFSET + 2);
        memory.set(0x57, MEMORY_OFFSET + 3);
        memory.set(0x00, 0x57);
        memory.set(OPCODE_LDA_ZPG, MEMORY_OFFSET + 4);
        memory.set(0x69, MEMORY_OFFSET + 5);
        memory.set(0x80, 0x69);

        let mut cpu = CPU::new(&mut memory);
        cpu.reset();

        let mut saved_pc = cpu.pc;
        let mut saved_cycles = cpu.cycles;
        let mut opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x32, cpu.acc);
        assert_eq!(cpu.pc - saved_pc, BYTES);
        assert_eq!(cpu.cycles - saved_cycles, CYCLES);
        assert!(!cpu.flag_is_set(CSF_ZERO));
        assert!(!cpu.flag_is_set(CSF_NEGATIVE));

        saved_pc = cpu.pc;
        saved_cycles = cpu.cycles;
        opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - saved_pc, BYTES);
        assert_eq!(cpu.cycles - saved_cycles, CYCLES);
        assert!(cpu.flag_is_set(CSF_ZERO));
        assert!(!cpu.flag_is_set(CSF_NEGATIVE));

        saved_pc = cpu.pc;
        saved_cycles = cpu.cycles;
        opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - saved_pc, BYTES);
        assert_eq!(cpu.cycles - saved_cycles, CYCLES);
        assert!(!cpu.flag_is_set(CSF_ZERO));
        assert!(cpu.flag_is_set(CSF_NEGATIVE));
    }

    #[test]
    fn lda_zero_page_x_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEMORY_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ZPX, MEMORY_OFFSET);
        memory.set(0x42, MEMORY_OFFSET + 1);
        memory.set(0x32, X.wrapping_add(0x42).into());
        memory.set(OPCODE_LDA_ZPX, MEMORY_OFFSET + 2);
        memory.set(0x57, MEMORY_OFFSET + 3);
        memory.set(0x00, X.wrapping_add(0x57).into());
        memory.set(OPCODE_LDA_ZPX, MEMORY_OFFSET + 4);
        memory.set(0x69, MEMORY_OFFSET + 5);
        memory.set(0x80, X.wrapping_add(0x69).into());

        let mut cpu = CPU::new(&mut memory);
        cpu.reset();
        cpu.x = X;

        let mut saved_pc = cpu.pc;
        let mut saved_cycles = cpu.cycles;
        let mut opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x32, cpu.acc);
        assert_eq!(cpu.pc - saved_pc, BYTES);
        assert_eq!(cpu.cycles - saved_cycles, CYCLES);
        assert!(!cpu.flag_is_set(CSF_ZERO));
        assert!(!cpu.flag_is_set(CSF_NEGATIVE));

        saved_pc = cpu.pc;
        saved_cycles = cpu.cycles;
        opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x00, cpu.acc);
        assert_eq!(cpu.pc - saved_pc, BYTES);
        assert_eq!(cpu.cycles - saved_cycles, CYCLES);
        assert!(cpu.flag_is_set(CSF_ZERO));
        assert!(!cpu.flag_is_set(CSF_NEGATIVE));

        saved_pc = cpu.pc;
        saved_cycles = cpu.cycles;
        opcode = cpu.fetch_byte();
        cpu.execute(opcode);
        assert_eq!(0x80, cpu.acc);
        assert_eq!(cpu.pc - saved_pc, BYTES);
        assert_eq!(cpu.cycles - saved_cycles, CYCLES);
        assert!(!cpu.flag_is_set(CSF_ZERO));
        assert!(cpu.flag_is_set(CSF_NEGATIVE));
    }
}
