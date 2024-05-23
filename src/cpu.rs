use crate::memory::Memory;

pub const CSF_CARRY: u8 = 0x01;
pub const CSF_ZERO: u8 = 0x02;
pub const CSF_INTERRUPT_DISABLE: u8 = 0x04;
pub const CSF_DECIMAL_MODE: u8 = 0x08;
pub const CSF_BREAK_COMMAND: u8 = 0x10;
pub const CSF_OVERFLOW: u8 = 0x40;
pub const CSF_NEGATIVE: u8 = 0x80;

const ZERO_PAGE_ADDR_START: u16 = 0x0000;
const ZERO_PAGE_ADDR_END: u16 = 0x00FF;
const SYS_STACK_ADDR_START: u16 = 0x0100;
const SYS_STACK_ADDR_END: u16 = 0x01FF;
const UNRESERVED_MEMORY_START: u16 = 0x0200;
const UNRESERVED_MEMORY_END: u16 = 0xFFF9;
const NON_MASKABLE_INTERRUPT_HANDLER: u16 = 0xFFFA; // also takes 0xFFFB
pub(crate) const POWER_ON_RESET: u16 = 0xFFFC; // also takes 0xFFFD
const INTERRUPT_REQUEST_HANDLER: u16 = 0xFFFE; // also takes 0xFFFF

const CPU_DEFAULT_ACC: u8 = 0;
const CPU_DEFAULT_X: u8 = 0;
const CPU_DEFAULT_Y: u8 = 0;
const CPU_DEFAULT_SP: u8 = 0xFF;
const CPU_DEFAULT_STATUS: u8 = 0x20;

const OPCODE_LDA_IMMEDIATE: u8 = 0xA9;
const OPCODE_LDA_ZERO_PAGE: u8 = 0xA5;

/// ps register: NV1B DIZC
#[derive(Default)]
pub struct CPU {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    status: u8,
    cycles: u64,
}

impl CPU {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self, memory: &Memory) {
        self.acc = CPU_DEFAULT_ACC;
        self.x = CPU_DEFAULT_X;
        self.y = CPU_DEFAULT_Y;
        self.sp = CPU_DEFAULT_SP;
        self.pc =
            ((memory.get(POWER_ON_RESET + 1) as u16) << 8) | (memory.get(POWER_ON_RESET) as u16);
        self.status = CPU_DEFAULT_STATUS;
        self.cycles = 7;
    }

    pub fn acc(&self) -> u8 {
        self.acc
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn sp(&self) -> u8 {
        self.sp
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    #[inline]
    pub fn flag_is_set(&self, flag: u8) -> bool {
        self.status & flag != 0
    }

    pub fn execute(&mut self, memory: &mut Memory, opcode: u8) {
        match opcode {
            OPCODE_LDA_IMMEDIATE => self.lda_immediate(memory),
            OPCODE_LDA_ZERO_PAGE => self.lda_zero_page(memory),
            _ => panic!("invalid opcode: {:#X}", opcode),
        }
    }

    pub fn fetch_byte(&mut self, memory: &Memory) -> u8 {
        let byte = memory.get(self.pc);
        self.increment_pc();
        self.cycles += 1;
        byte
    }

    #[inline]
    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    /// bytes: 2
    /// cycles: 2
    /// flags affected: N and Z
    fn lda_immediate(&mut self, memory: &Memory) {
        let immediate = memory.get(self.pc);
        self.increment_pc();
        self.acc = immediate;
        self.lda_set_status(immediate);
        self.cycles += 1;
    }

    /// bytes: 2
    /// cycles: 3
    /// flags affected: N and Z
    fn lda_zero_page(&mut self, memory: &Memory) {
        let addr = self.fetch_byte(memory);
        let acc = memory.get(addr.into());
        self.acc = acc;
        self.lda_set_status(acc);
        self.cycles += 1;
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
        const MEMORY_OFFSET: u16 = UNRESERVED_MEMORY_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_IMMEDIATE, MEMORY_OFFSET);
        memory.set(0x42, MEMORY_OFFSET + 1);
        memory.set(OPCODE_LDA_IMMEDIATE, MEMORY_OFFSET + 2);
        memory.set(0x00, MEMORY_OFFSET + 3);
        memory.set(OPCODE_LDA_IMMEDIATE, MEMORY_OFFSET + 4);
        memory.set(0x80, MEMORY_OFFSET + 5);

        let mut cpu = CPU::new();
        cpu.reset(&memory);

        let mut saved_pc = cpu.pc();
        let mut saved_cycles = cpu.cycles();
        let mut opcode = cpu.fetch_byte(&memory);
        cpu.execute(&mut memory, opcode);
        assert_eq!(0x42, cpu.acc());
        assert_eq!(cpu.pc() - saved_pc, BYTES);
        assert_eq!(cpu.cycles() - saved_cycles, CYCLES);
        assert!(!cpu.flag_is_set(CSF_ZERO));
        assert!(!cpu.flag_is_set(CSF_NEGATIVE));

        saved_pc = cpu.pc();
        saved_cycles = cpu.cycles();
        opcode = cpu.fetch_byte(&memory);
        cpu.execute(&mut memory, opcode);
        assert_eq!(0x00, cpu.acc());
        assert_eq!(cpu.pc() - saved_pc, BYTES);
        assert_eq!(cpu.cycles() - saved_cycles, CYCLES);
        assert!(cpu.flag_is_set(CSF_ZERO));
        assert!(!cpu.flag_is_set(CSF_NEGATIVE));

        saved_pc = cpu.pc();
        saved_cycles = cpu.cycles();
        opcode = cpu.fetch_byte(&memory);
        cpu.execute(&mut memory, opcode);
        assert_eq!(0x80, cpu.acc());
        assert_eq!(cpu.pc() - saved_pc, BYTES);
        assert_eq!(cpu.cycles() - saved_cycles, CYCLES);
        assert!(!cpu.flag_is_set(CSF_ZERO));
        assert!(cpu.flag_is_set(CSF_NEGATIVE));
    }

    #[test]
    fn lda_zero_page_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEMORY_OFFSET: u16 = UNRESERVED_MEMORY_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ZERO_PAGE, MEMORY_OFFSET);
        memory.set(0x42, MEMORY_OFFSET + 1);
        memory.set(0x32, 0x42);
        memory.set(OPCODE_LDA_ZERO_PAGE, MEMORY_OFFSET + 2);
        memory.set(0x57, MEMORY_OFFSET + 3);
        memory.set(0x00, 0x57);
        memory.set(OPCODE_LDA_ZERO_PAGE, MEMORY_OFFSET + 4);
        memory.set(0x69, MEMORY_OFFSET + 5);
        memory.set(0x80, 0x69);

        let mut cpu = CPU::new();
        cpu.reset(&memory);

        let mut saved_pc = cpu.pc();
        let mut saved_cycles = cpu.cycles();
        let mut opcode = cpu.fetch_byte(&memory);
        cpu.execute(&mut memory, opcode);
        assert_eq!(0x32, cpu.acc());
        assert_eq!(cpu.pc() - saved_pc, BYTES);
        assert_eq!(cpu.cycles() - saved_cycles, CYCLES);
        assert!(!cpu.flag_is_set(CSF_ZERO));
        assert!(!cpu.flag_is_set(CSF_NEGATIVE));

        saved_pc = cpu.pc();
        saved_cycles = cpu.cycles();
        opcode = cpu.fetch_byte(&memory);
        cpu.execute(&mut memory, opcode);
        assert_eq!(0x00, cpu.acc());
        assert_eq!(cpu.pc() - saved_pc, BYTES);
        assert_eq!(cpu.cycles() - saved_cycles, CYCLES);
        assert!(cpu.flag_is_set(CSF_ZERO));
        assert!(!cpu.flag_is_set(CSF_NEGATIVE));

        saved_pc = cpu.pc();
        saved_cycles = cpu.cycles();
        opcode = cpu.fetch_byte(&memory);
        cpu.execute(&mut memory, opcode);
        assert_eq!(0x80, cpu.acc());
        assert_eq!(cpu.pc() - saved_pc, BYTES);
        assert_eq!(cpu.cycles() - saved_cycles, CYCLES);
        assert!(!cpu.flag_is_set(CSF_ZERO));
        assert!(cpu.flag_is_set(CSF_NEGATIVE));
    }
}
