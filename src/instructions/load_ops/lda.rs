use crate::cpu::{CPU, CSF_NEGATIVE, CSF_ZERO};
use crate::instructions::{AddressingMode, Instruction, Opcode};

/// Loads a byte of memory into the accumulator setting the zero and negative
/// flags as appropriate.
///
/// # Attributes
///
/// - Bytes: 2-3
/// - Cycles: 2-6
/// - Flags affected: N, Z
///
/// # Addressing Modes
///
/// Supported addressing mode(s):
///
/// - Immediate
/// - Zero Page
/// - Zero Page,X
/// - Absolute
/// - Absolute,X
/// - Absolute,Y
/// - (Indirect,X)
/// - (Indirect),Y
///
/// # Cycles
///
/// If a page crossing occurs, the following addressing mode(s) will consume one
/// more cycle than what is returned in `self.cycles()`:
///
/// - Absolute,X
/// - Absolute,Y
/// - (Indirect),Y
pub struct LDA {
    addr_mode: AddressingMode,
    opcode: u8,
    bytes: u8,
    cycles: u8,
}

impl LDA {
    /// Constructs a new `LDA` instruction.
    ///
    /// # Panics
    ///
    /// Panics if an invalid addressing mode is provided.
    pub fn new(addr_mode: AddressingMode) -> Self {
        match addr_mode {
            AddressingMode::Immediate => Self {
                addr_mode,
                opcode: Opcode::LDAImm.into(),
                bytes: 2,
                cycles: 2,
            },
            AddressingMode::ZeroPage => Self {
                addr_mode,
                opcode: Opcode::LDAZpg.into(),
                bytes: 2,
                cycles: 3,
            },
            AddressingMode::ZeroPageX => Self {
                addr_mode,
                opcode: Opcode::LDAZpx.into(),
                bytes: 2,
                cycles: 4,
            },
            AddressingMode::Absolute => Self {
                addr_mode,
                opcode: Opcode::LDAAbs.into(),
                bytes: 3,
                cycles: 4,
            },
            AddressingMode::AbsoluteX => Self {
                addr_mode,
                opcode: Opcode::LDAAbx.into(),
                bytes: 3,
                cycles: 4,
            },
            AddressingMode::AbsoluteY => Self {
                addr_mode,
                opcode: Opcode::LDAAby.into(),
                bytes: 3,
                cycles: 4,
            },
            AddressingMode::IndirectX => Self {
                addr_mode,
                opcode: Opcode::LDAIdx.into(),
                bytes: 2,
                cycles: 6,
            },
            AddressingMode::IndirectY => Self {
                addr_mode,
                opcode: Opcode::LDAIdy.into(),
                bytes: 2,
                cycles: 5,
            },
            _ => panic!(
                "Invalid addressing mode for this instruction: {:?}",
                addr_mode
            ),
        }
    }

    fn set_status_flags(&self, cpu: &mut CPU) {
        cpu.status &= !(CSF_ZERO | CSF_NEGATIVE);
        if cpu.acc == 0 {
            cpu.status |= CSF_ZERO;
        } else if CPU::byte_is_negative_int(cpu.acc) {
            cpu.status |= CSF_NEGATIVE;
        }
    }

    /// Consumes:
    ///
    /// - Bytes: 2
    /// - Cycles: 2
    fn immediate(&self, cpu: &mut CPU) {
        cpu.acc = cpu.fetch_byte();
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 2
    /// - Cycles: 3
    fn zero_page(&self, cpu: &mut CPU) {
        let addr = cpu.fetch_byte();
        cpu.acc = cpu.read_byte(addr.into());
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 2
    /// - Cycles: 4
    fn zero_page_x(&self, cpu: &mut CPU) {
        let byte = cpu.fetch_byte();
        let addr = cpu.x.wrapping_add(byte);
        cpu.cycles += 1;
        cpu.acc = cpu.read_byte(addr.into());
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 3
    /// - Cycles: 4
    fn absolute(&self, cpu: &mut CPU) {
        let addr = cpu.fetch_addr();
        cpu.acc = cpu.read_byte(addr);
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 3
    /// - Cycles: 4 (+1 if page crossed)
    fn absolute_x(&self, cpu: &mut CPU) {
        let abs_addr = cpu.fetch_addr();
        let eff_addr = abs_addr.wrapping_add(cpu.x.into());
        if CPU::page_crossed(abs_addr, eff_addr) {
            cpu.cycles += 1;
        }
        cpu.acc = cpu.read_byte(eff_addr);
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 3
    /// - Cycles: 4 (+1 if page crossed)
    fn absolute_y(&self, cpu: &mut CPU) {
        let abs_addr = cpu.fetch_addr();
        let eff_addr = abs_addr.wrapping_add(cpu.y.into());
        if CPU::page_crossed(abs_addr, eff_addr) {
            cpu.cycles += 1;
        }
        cpu.acc = cpu.read_byte(eff_addr);
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 2
    /// - Cycles: 6
    fn indirect_x(&self, cpu: &mut CPU) {
        let zpg_addr = cpu.fetch_byte();
        let addr = zpg_addr.wrapping_add(cpu.x);
        cpu.cycles += 1;
        let eff_addr = cpu.read_addr(addr.into(), addr.wrapping_add(1).into());
        cpu.acc = cpu.read_byte(eff_addr);
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 2
    /// - Cycles: 5 (+1 if page crossed)
    fn indirect_y(&self, cpu: &mut CPU) {
        let zpg_addr = cpu.fetch_byte();
        let addr = cpu.read_addr(zpg_addr.into(), zpg_addr.wrapping_add(1).into());
        let eff_addr = addr.wrapping_add(cpu.y.into());
        if CPU::page_crossed(addr, eff_addr) {
            cpu.cycles += 1;
        }
        cpu.acc = cpu.read_byte(eff_addr);
        self.set_status_flags(cpu);
    }
}

impl Instruction for LDA {
    fn execute(&self, cpu: &mut CPU) {
        match self.addr_mode {
            AddressingMode::Immediate => self.immediate(cpu),
            AddressingMode::ZeroPage => self.zero_page(cpu),
            AddressingMode::ZeroPageX => self.zero_page_x(cpu),
            AddressingMode::Absolute => self.absolute(cpu),
            AddressingMode::AbsoluteX => self.absolute_x(cpu),
            AddressingMode::AbsoluteY => self.absolute_y(cpu),
            AddressingMode::IndirectX => self.indirect_x(cpu),
            AddressingMode::IndirectY => self.indirect_y(cpu),
            _ => unreachable!(),
        }
    }

    fn addressing_mode(&self) -> AddressingMode {
        self.addr_mode
    }

    fn opcode(&self) -> u8 {
        self.opcode
    }

    fn cycles(&self) -> u8 {
        self.cycles
    }

    fn bytes(&self) -> u8 {
        self.bytes
    }

    fn flags_affected(&self) -> u8 {
        CSF_ZERO | CSF_NEGATIVE
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::{CPU, CSF_NEGATIVE, CSF_ZERO, UNRESERVED_MEMORY_ADDR_START};
    use crate::instructions::Opcode;
    use crate::memory::Memory;

    #[test]
    fn lda_immediate_positive_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAImm.into(), MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn lda_immediate_negative_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAImm.into(), MEM_OFFSET);
        memory.write(0x82, MEM_OFFSET + 1);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn lda_immediate_zero_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAImm.into(), MEM_OFFSET);
        memory.write(0x00, MEM_OFFSET + 1);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn lda_zero_page_positive_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAZpg.into(), MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);
        memory.write(0x32, 0x42);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x32);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn lda_zero_page_negative_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAZpg.into(), MEM_OFFSET);
        memory.write(0x69, MEM_OFFSET + 1);
        memory.write(0x82, 0x69);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn lda_zero_page_zero_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAZpg.into(), MEM_OFFSET);
        memory.write(0x57, MEM_OFFSET + 1);
        memory.write(0x00, 0x57);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn lda_zero_page_x_positive_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAZpx.into(), MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);
        memory.write(0x32, X.wrapping_add(0x42).into());

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x32);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn lda_zero_page_x_negative_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAZpx.into(), MEM_OFFSET);
        memory.write(0x69, MEM_OFFSET + 1);
        memory.write(0x82, X.wrapping_add(0x69).into());

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn lda_zero_page_x_zero_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAZpx.into(), MEM_OFFSET);
        memory.write(0x57, MEM_OFFSET + 1);
        memory.write(0x00, X.wrapping_add(0x57).into());

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn lda_absolute_positive_value_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAbs.into(), MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x42, 0x8028);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn lda_absolute_negative_value_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAbs.into(), MEM_OFFSET);
        memory.write(0x70, MEM_OFFSET + 1);
        memory.write(0x55, MEM_OFFSET + 2);
        memory.write(0x82, 0x5570);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_zero_value_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAbs.into(), MEM_OFFSET);
        memory.write(0x97, MEM_OFFSET + 1);
        memory.write(0x26, MEM_OFFSET + 2);
        memory.write(0x00, 0x2697);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn lda_absolute_x_positive_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAbx.into(), MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x42, (X as u16).wrapping_add(0x8028));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn lda_absolute_x_negative_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAbx.into(), MEM_OFFSET);
        memory.write(0x22, MEM_OFFSET + 1);
        memory.write(0x55, MEM_OFFSET + 2);
        memory.write(0x82, (X as u16).wrapping_add(0x5522));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_x_zero_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAbx.into(), MEM_OFFSET);
        memory.write(0x53, MEM_OFFSET + 1);
        memory.write(0x26, MEM_OFFSET + 2);
        memory.write(0x00, (X as u16).wrapping_add(0x2653));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn lda_absolute_x_positive_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAbx.into(), MEM_OFFSET);
        memory.write(0x60, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x42, (X as u16).wrapping_add(0x8060));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn lda_absolute_x_negative_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAbx.into(), MEM_OFFSET);
        memory.write(0x83, MEM_OFFSET + 1);
        memory.write(0x55, MEM_OFFSET + 2);
        memory.write(0x82, (X as u16).wrapping_add(0x5583));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_x_zero_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAbx.into(), MEM_OFFSET);
        memory.write(0x54, MEM_OFFSET + 1);
        memory.write(0x26, MEM_OFFSET + 2);
        memory.write(0x00, (X as u16).wrapping_add(0x2654));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn lda_absolute_y_positive_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAby.into(), MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x42, (Y as u16).wrapping_add(0x8028));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn lda_absolute_y_negative_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAby.into(), MEM_OFFSET);
        memory.write(0x22, MEM_OFFSET + 1);
        memory.write(0x55, MEM_OFFSET + 2);
        memory.write(0x82, (Y as u16).wrapping_add(0x5522));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_y_zero_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAby.into(), MEM_OFFSET);
        memory.write(0x53, MEM_OFFSET + 1);
        memory.write(0x26, MEM_OFFSET + 2);
        memory.write(0x00, (Y as u16).wrapping_add(0x2653));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn lda_absolute_y_positive_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAby.into(), MEM_OFFSET);
        memory.write(0x60, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x42, (Y as u16).wrapping_add(0x8060));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn lda_absolute_y_negative_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAby.into(), MEM_OFFSET);
        memory.write(0x83, MEM_OFFSET + 1);
        memory.write(0x55, MEM_OFFSET + 2);
        memory.write(0x82, (Y as u16).wrapping_add(0x5583));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_y_zero_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDAAby.into(), MEM_OFFSET);
        memory.write(0x54, MEM_OFFSET + 1);
        memory.write(0x26, MEM_OFFSET + 2);
        memory.write(0x00, (Y as u16).wrapping_add(0x2654));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn lda_indirect_x_positive_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        let zpg_addr = 0x08;
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);

        memory.write(Opcode::LDAIdx.into(), MEM_OFFSET);
        memory.write(zpg_addr, MEM_OFFSET + 1);
        memory.write(0x22, addr_l.into());
        memory.write(0x32, addr_h.into());
        memory.write(0x42, 0x3222);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn lda_indirect_x_negative_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        let zpg_addr = 0x69;
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);

        memory.write(Opcode::LDAIdx.into(), MEM_OFFSET);
        memory.write(zpg_addr, MEM_OFFSET + 1);
        memory.write(0x13, addr_l.into());
        memory.write(0x77, addr_h.into());
        memory.write(0x82, 0x7713);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn lda_indirect_x_zero_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        let zpg_addr = 0x57;
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);

        memory.write(Opcode::LDAIdx.into(), MEM_OFFSET);
        memory.write(zpg_addr, MEM_OFFSET + 1);
        memory.write(0x46, addr_l.into());
        memory.write(0x83, addr_h.into());
        memory.write(0x00, 0x8346);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn lda_indirect_y_positive_value_without_page_crossing_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        let zpg_addr = 0x08;
        let addr_l = 0x22;
        let addr_h = 0x32;
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());

        memory.write(Opcode::LDAIdy.into(), MEM_OFFSET);
        memory.write(zpg_addr, MEM_OFFSET + 1);
        memory.write(addr_l, zpg_addr.into());
        memory.write(addr_h, zpg_addr.wrapping_add(1).into());
        memory.write(0x42, addr);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn lda_indirect_y_negative_value_without_page_crossing_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        let zpg_addr = 0x69;
        let addr_l = 0x13;
        let addr_h = 0x77;
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());

        memory.write(Opcode::LDAIdy.into(), MEM_OFFSET);
        memory.write(zpg_addr, MEM_OFFSET + 1);
        memory.write(addr_l, zpg_addr.into());
        memory.write(addr_h, zpg_addr.wrapping_add(1).into());
        memory.write(0x82, addr);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn lda_indirect_y_zero_value_without_page_crossing_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        let zpg_addr = 0x57;
        let addr_l = 0x46;
        let addr_h = 0x83;
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());

        memory.write(Opcode::LDAIdy.into(), MEM_OFFSET);
        memory.write(zpg_addr, MEM_OFFSET + 1);
        memory.write(addr_l, zpg_addr.into());
        memory.write(addr_h, zpg_addr.wrapping_add(1).into());
        memory.write(0x00, addr);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn lda_indirect_y_positive_value_with_page_crossing_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        let zpg_addr = 0x08;
        let addr_l = 0xDF;
        let addr_h = 0x32;
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());

        memory.write(Opcode::LDAIdy.into(), MEM_OFFSET);
        memory.write(zpg_addr, MEM_OFFSET + 1);
        memory.write(addr_l, zpg_addr.into());
        memory.write(addr_h, zpg_addr.wrapping_add(1).into());
        memory.write(0x42, addr);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn lda_indirect_y_negative_value_with_page_crossing_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        let zpg_addr = 0x69;
        let addr_l = 0x86;
        let addr_h = 0x77;
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());

        memory.write(Opcode::LDAIdy.into(), MEM_OFFSET);
        memory.write(zpg_addr, MEM_OFFSET + 1);
        memory.write(addr_l, zpg_addr.into());
        memory.write(addr_h, zpg_addr.wrapping_add(1).into());
        memory.write(0x82, addr);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn lda_indirect_y_zero_value_with_page_crossing_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        let zpg_addr = 0x57;
        let addr_l = 0xCC;
        let addr_h = 0x83;
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());

        memory.write(Opcode::LDAIdy.into(), MEM_OFFSET);
        memory.write(zpg_addr, MEM_OFFSET + 1);
        memory.write(addr_l, zpg_addr.into());
        memory.write(addr_h, zpg_addr.wrapping_add(1).into());
        memory.write(0x00, addr);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }
}
