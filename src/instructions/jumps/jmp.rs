use crate::cpu::{CPU, POWER_ON_RESET_ADDR_L};
use crate::instructions::{AddressingMode, Instruction, Opcode};

/// Sets the program counter to the address specified by the operand.
///
/// # Attributes
///
/// - Bytes: 3
/// - Cycles: 3-5
/// - Flags affected: none
///
/// # Addressing Modes
///
/// Supported addressing mode(s):
///
/// - Absolute
/// - Indirect
pub struct JMP {
    addr_mode: AddressingMode,
    opcode: u8,
    bytes: u8,
    cycles: u8,
}

impl JMP {
    /// Constructs a new `JMP` instruction.
    ///
    /// # Panics
    ///
    /// Panics if an invalid addressing mode is provided.
    pub fn new(addr_mode: AddressingMode) -> Self {
        let bytes = 3;
        let opcode;
        let cycles;
        match addr_mode {
            AddressingMode::Absolute => {
                opcode = Opcode::JMPAbs.into();
                cycles = 3;
            }
            AddressingMode::IndirectX => {
                opcode = Opcode::JMPInd.into();
                cycles = 5;
            }
            _ => panic!("Invalid addressing mode for this instruction"),
        }

        Self {
            addr_mode,
            opcode,
            bytes,
            cycles,
        }
    }

    /// Consumes:
    ///
    /// - Bytes: 3
    /// - Cycles: 3
    #[inline]
    fn absolute(&self, cpu: &mut CPU) {
        cpu.pc = cpu.fetch_addr()
    }

    /// Consumes:
    ///
    /// - Bytes: 3
    /// - Cycles: 5
    fn indirect_x(&self, cpu: &mut CPU) {
        // hardware bug if LSB is 0xFF
        // http://www.6502.org/users/obelisk/6502/reference.html#JMP
        let ind_addr = cpu.fetch_addr();
        if ind_addr & 0x00FF == 0x00FF {
            let ind_addr_h = ind_addr & 0xFF00;
            let addr = cpu.read_addr(ind_addr.into(), ind_addr_h.into());
            cpu.pc = addr;
        } else if ind_addr == POWER_ON_RESET_ADDR_L {
            cpu.reset();
        } else {
            let addr = cpu.read_addr(ind_addr.into(), (ind_addr + 1).into());
            cpu.pc = addr;
        }
    }
}

impl Instruction for JMP {
    fn execute(&self, cpu: &mut CPU) {
        match self.addr_mode {
            AddressingMode::Absolute => self.absolute(cpu),
            AddressingMode::IndirectX => self.indirect_x(cpu),
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
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::{
        CPU, CPU_DEFAULT_STATUS, POWER_ON_RESET_ADDR_L, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::instructions::Opcode;
    use crate::memory::Memory;

    #[test]
    fn jmp_abs_test() {
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::JMPAbs.into(), MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);
        memory.write(0x30, MEM_OFFSET + 2);
        memory.write(Opcode::LDAImm.into(), 0x3042);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.pc, 0x3042);
        assert_eq!(cpu.memory.read(cpu.pc), Opcode::LDAImm.into());
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }

    #[test]
    fn jmp_ind_test() {
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::JMPInd.into(), MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);
        memory.write(0x30, MEM_OFFSET + 2);
        memory.write(0xAC, 0x3042);
        memory.write(0x28, 0x3043);
        memory.write(Opcode::JMPInd.into(), 0x28AC);
        memory.write(0xFF, 0x28AC + 1);
        memory.write(0x51, 0x28AC + 2);
        memory.write(0x76, 0x51FF); // hardware bug
        memory.write(0x11, 0x5100);
        memory.write(Opcode::JMPInd.into(), 0x1176);
        memory.write(POWER_ON_RESET_ADDR_L as u8, 0x1176 + 1);
        memory.write((POWER_ON_RESET_ADDR_L >> 8) as u8, 0x1176 + 2);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.pc, 0x28AC);
        assert_eq!(cpu.memory.read(cpu.pc), Opcode::JMPInd.into());
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.pc, 0x1176);
        assert_eq!(cpu.memory.read(cpu.pc), Opcode::JMPInd.into());
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        cpu.execute_next_instruction();
        memory.write(Opcode::LDAImm.into(), MEM_OFFSET);
        cpu.memory = memory;
        assert_eq!(cpu.pc, UNRESERVED_MEMORY_ADDR_START);
        assert_eq!(cpu.memory.read(cpu.pc), Opcode::LDAImm.into());
        assert_eq!(cpu.cycles, 7);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }
}
