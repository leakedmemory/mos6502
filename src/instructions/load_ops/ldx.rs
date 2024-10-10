use crate::cpu::{CPU, CSF_NEGATIVE, CSF_ZERO};
use crate::instructions::{AddressingMode, Instruction, Opcode};

/// Loads a byte of memory into the X register setting the zero and negative
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
/// - Zero Page,Y
/// - Absolute
/// - Absolute,Y
///
/// # Cycles
///
/// If a page crossing occurs, the following addressing mode(s) will consume one
/// more cycle than what is returned in `self.cycles()`:
///
/// - Absolute,Y
pub struct LDX {
    addr_mode: AddressingMode,
    opcode: u8,
    bytes: u8,
    cycles: u8,
}

impl LDX {
    /// Constructs a new `LDX` instruction.
    ///
    /// # Panics
    ///
    /// Panics if an invalid addressing mode is provided.
    pub fn new(addr_mode: AddressingMode) -> Self {
        match addr_mode {
            AddressingMode::Immediate => Self {
                addr_mode,
                opcode: Opcode::LDXImm.into(),
                bytes: 2,
                cycles: 2,
            },
            AddressingMode::ZeroPage => Self {
                addr_mode,
                opcode: Opcode::LDXZpg.into(),
                bytes: 2,
                cycles: 3,
            },
            AddressingMode::ZeroPageY => Self {
                addr_mode,
                opcode: Opcode::LDXZpy.into(),
                bytes: 2,
                cycles: 4,
            },
            AddressingMode::Absolute => Self {
                addr_mode,
                opcode: Opcode::LDXAbs.into(),
                bytes: 3,
                cycles: 4,
            },
            AddressingMode::AbsoluteY => Self {
                addr_mode,
                opcode: Opcode::LDXAby.into(),
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
        if cpu.x == 0 {
            cpu.status |= CSF_ZERO;
        } else if CPU::byte_is_negative_int(cpu.x) {
            cpu.status |= CSF_NEGATIVE;
        }
    }

    /// Consumes:
    ///
    /// - Bytes: 2
    /// - Cycles: 2
    fn immediate(&self, cpu: &mut CPU) {
        cpu.x = cpu.fetch_byte();
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 2
    /// - Cycles: 3
    fn zero_page(&self, cpu: &mut CPU) {
        let addr = cpu.fetch_byte();
        cpu.x = cpu.read_byte(addr.into());
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 2
    /// - Cycles: 4
    fn zero_page_y(&self, cpu: &mut CPU) {
        let byte = cpu.fetch_byte();
        let addr = cpu.y.wrapping_add(byte);
        cpu.cycles += 1;
        cpu.x = cpu.read_byte(addr.into());
        self.set_status_flags(cpu);
    }

    /// Consumes:
    ///
    /// - Bytes: 3
    /// - Cycles: 4
    fn absolute(&self, cpu: &mut CPU) {
        let addr = cpu.fetch_addr();
        cpu.x = cpu.read_byte(addr);
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
        cpu.x = cpu.read_byte(eff_addr);
        self.set_status_flags(cpu);
    }
}

impl Instruction for LDX {
    fn execute(&self, cpu: &mut CPU) {
        match self.addr_mode {
            AddressingMode::Immediate => self.immediate(cpu),
            AddressingMode::ZeroPage => self.zero_page(cpu),
            AddressingMode::ZeroPageY => self.zero_page_y(cpu),
            AddressingMode::Absolute => self.absolute(cpu),
            AddressingMode::AbsoluteY => self.absolute_y(cpu),
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
    fn ldx_immediate_positive_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXImm.into(), MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldx_immediate_negative_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXImm.into(), MEM_OFFSET);
        memory.write(0x82, MEM_OFFSET + 1);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn ldx_immediate_zero_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXImm.into(), MEM_OFFSET);
        memory.write(0x00, MEM_OFFSET + 1);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn ldx_zero_page_positive_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXZpg.into(), MEM_OFFSET);
        memory.write(0x32, MEM_OFFSET + 1);
        memory.write(0x42, 0x32);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldx_zero_page_negative_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXZpg.into(), MEM_OFFSET);
        memory.write(0x69, MEM_OFFSET + 1);
        memory.write(0x82, 0x69);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn ldx_zero_page_zero_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXZpg.into(), MEM_OFFSET);
        memory.write(0x57, MEM_OFFSET + 1);
        memory.write(0x00, 0x57);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn ldx_zero_page_y_positive_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXZpy.into(), MEM_OFFSET);
        memory.write(0x32, MEM_OFFSET + 1);
        memory.write(0x42, Y.wrapping_add(0x32).into());

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldx_zero_page_y_negative_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXZpy.into(), MEM_OFFSET);
        memory.write(0x69, MEM_OFFSET + 1);
        memory.write(0x82, Y.wrapping_add(0x69).into());

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn ldx_zero_page_y_zero_value_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXZpy.into(), MEM_OFFSET);
        memory.write(0x57, MEM_OFFSET + 1);
        memory.write(0x00, Y.wrapping_add(0x57).into());

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn ldx_absolute_positive_value_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXAbs.into(), MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x42, 0x8028);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldx_absolute_negative_value_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXAbs.into(), MEM_OFFSET);
        memory.write(0x70, MEM_OFFSET + 1);
        memory.write(0x55, MEM_OFFSET + 2);
        memory.write(0x82, 0x5570);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn ldx_absolute_zero_value_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXAbs.into(), MEM_OFFSET);
        memory.write(0x97, MEM_OFFSET + 1);
        memory.write(0x26, MEM_OFFSET + 2);
        memory.write(0x00, 0x2697);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn ldx_absolute_y_positive_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXAby.into(), MEM_OFFSET);
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
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldx_absolute_y_negative_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXAby.into(), MEM_OFFSET);
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
        assert_eq!(cpu.x, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }

    #[test]
    fn ldx_absolute_y_zero_value_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXAby.into(), MEM_OFFSET);
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
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }

    #[test]
    fn ldx_absolute_y_positive_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXAby.into(), MEM_OFFSET);
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
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status);
    }

    #[test]
    fn ldx_absolute_y_negative_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXAby.into(), MEM_OFFSET);
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
        assert_eq!(cpu.x, 0x82);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_NEGATIVE);
    }
    #[test]
    fn ldx_absolute_y_zero_value_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDXAby.into(), MEM_OFFSET);
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
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, init_status | CSF_ZERO);
    }
}
