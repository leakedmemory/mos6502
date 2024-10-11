use crate::cpu::{CPU, CSF_NEGATIVE, CSF_ZERO};
use crate::instructions::{AddressingMode, Instruction, Opcode};

/// Loads a byte of memory into the Y register setting the zero and negative
/// flags as appropriate.
///
/// # Attributes
///
/// - Bytes: 2-3
/// - Cycles: 2-5
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
///
/// # Cycles
///
/// If a page crossing occurs, the following addressing mode(s) will consume one
/// more cycle than what is returned in `self.cycles()`:
///
/// - Absolute,X
pub struct LDY {
    addr_mode: AddressingMode,
    opcode: u8,
    bytes: u8,
    cycles: u8,
}

impl LDY {
    /// Constructs a new `LDY` instruction.
    ///
    /// # Panics
    ///
    /// Panics if an invalid addressing mode is provided.
    pub fn new(addr_mode: AddressingMode) -> Self {
        match addr_mode {
            AddressingMode::Immediate => Self {
                addr_mode,
                opcode: Opcode::LDYImm.into(),
                bytes: 2,
                cycles: 2,
            },
            AddressingMode::ZeroPage => Self {
                addr_mode,
                opcode: Opcode::LDYZpg.into(),
                bytes: 2,
                cycles: 3,
            },
            AddressingMode::ZeroPageX => Self {
                addr_mode,
                opcode: Opcode::LDYZpx.into(),
                bytes: 2,
                cycles: 4,
            },
            AddressingMode::Absolute => Self {
                addr_mode,
                opcode: Opcode::LDYAbs.into(),
                bytes: 3,
                cycles: 4,
            },
            AddressingMode::AbsoluteX => Self {
                addr_mode,
                opcode: Opcode::LDYAbx.into(),
                bytes: 3,
                cycles: 4,
            },
            _ => panic!(
                "Invalid addressing mode for this instruction {:?}",
                addr_mode
            ),
        }
    }

    fn set_status_flags(&self, cpu: &mut CPU) {
        cpu.status &= !(CSF_ZERO | CSF_NEGATIVE);
        if cpu.y == 0 {
            cpu.status |= CSF_ZERO;
        } else if CPU::byte_is_negative_int(cpu.y) {
            cpu.status |= CSF_NEGATIVE;
        }
    }

    /// Consumes:
    ///
    /// - Bytes: 2
    /// - Cycles: 2
    fn immediate(&self, cpu: &mut CPU) {
        cpu.y = cpu.fetch_byte();
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 2
    /// - Cycles: 3
    fn zero_page(&self, cpu: &mut CPU) {
        let addr = cpu.fetch_byte();
        cpu.y = cpu.read_byte(addr.into());
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
        cpu.y = cpu.read_byte(addr.into());
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 3
    /// - Cycles: 4
    fn absolute(&self, cpu: &mut CPU) {
        let addr = cpu.fetch_addr();
        cpu.y = cpu.read_byte(addr);
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
        cpu.y = cpu.read_byte(eff_addr);
        self.set_status_flags(cpu);
    }
}

impl Instruction for LDY {
    fn execute(&self, cpu: &mut CPU) {
        match self.addr_mode {
            AddressingMode::Immediate => self.immediate(cpu),
            AddressingMode::ZeroPage => self.zero_page(cpu),
            AddressingMode::ZeroPageX => self.zero_page_x(cpu),
            AddressingMode::Absolute => self.absolute(cpu),
            AddressingMode::AbsoluteX => self.absolute_x(cpu),
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
    use crate::cpu::{
        CPU, CPU_DEFAULT_STATUS, CSF_NEGATIVE, CSF_ZERO, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::instructions::Opcode;
    use crate::memory::Memory;

    #[test]
    fn ldy_immediate_positive_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYImm.into(), MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldy_immediate_negative_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYImm.into(), MEM_OFFSET);
        memory.write(0x82, MEM_OFFSET + 1);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn ldy_immediate_zero_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYImm.into(), MEM_OFFSET);
        memory.write(0x00, MEM_OFFSET + 1);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn ldy_zero_page_positive_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYZpg.into(), MEM_OFFSET);
        memory.write(0x32, MEM_OFFSET + 1);
        memory.write(0x42, 0x32);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldy_zero_page_negative_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYZpg.into(), MEM_OFFSET);
        memory.write(0x32, MEM_OFFSET + 1);
        memory.write(0x82, 0x32);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn ldy_zero_page_zero_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYZpg.into(), MEM_OFFSET);
        memory.write(0x32, MEM_OFFSET + 1);
        memory.write(0x00, 0x32);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn ldy_zero_page_x_positive_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYZpx.into(), MEM_OFFSET);
        memory.write(0x32, MEM_OFFSET + 1);
        memory.write(0x42, X.wrapping_add(0x32).into());

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldy_zero_page_x_negative_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYZpx.into(), MEM_OFFSET);
        memory.write(0x32, MEM_OFFSET + 1);
        memory.write(0x82, X.wrapping_add(0x32).into());

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn ldy_zero_page_x_zero_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYZpx.into(), MEM_OFFSET);
        memory.write(0x32, MEM_OFFSET + 1);
        memory.write(0x00, X.wrapping_add(0x32).into());

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn ldy_absolute_positive_value_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbs.into(), MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x42, 0x8028);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldy_absolute_negative_value_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbs.into(), MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x82, 0x8028);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }
    #[test]
    fn ldy_absolute_zero_value_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbs.into(), MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x00, 0x8028);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn ldy_absolute_x_positive_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbx.into(), MEM_OFFSET);
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
        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldy_absolute_x_negative_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbx.into(), MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x82, (X as u16).wrapping_add(0x8028));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn ldy_absolute_x_zero_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbx.into(), MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x00, (X as u16).wrapping_add(0x8028));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn ldy_absolute_x_positive_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbx.into(), MEM_OFFSET);
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
        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldy_absolute_x_negative_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbx.into(), MEM_OFFSET);
        memory.write(0x60, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x82, (X as u16).wrapping_add(0x8060));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }
    #[test]
    fn ldy_absolute_x_zero_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbx.into(), MEM_OFFSET);
        memory.write(0x60, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x00, (X as u16).wrapping_add(0x8060));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }
}
