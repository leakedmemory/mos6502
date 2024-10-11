use crate::cpu::CPU;
use crate::instructions::{AddressingMode, Instruction, Opcode};

/// Used at the end of a subroutine to return to the calling routine. It pulls
/// the program counter (minus one) from the stack.
///
/// # Attributes
///
/// - Bytes: 1
/// - Cycles: 6
/// - Flags affected: none
///
/// # Addressing Modes:
///
/// Supported addressing mode(s):
///
/// - Implied
pub struct RTS {
    addr_mode: AddressingMode,
    opcode: u8,
    bytes: u8,
    cycles: u8,
}

impl RTS {
    /// Constructs a new `RTS` instruction.
    pub fn new() -> Self {
        Self {
            addr_mode: AddressingMode::Implied,
            opcode: Opcode::RTS.into(),
            bytes: 1,
            cycles: 6,
        }
    }
}

impl Instruction for RTS {
    fn execute(&self, cpu: &mut CPU) {
        let addr = cpu.pop_addr_from_stack();
        cpu.pc = addr + 1; // takes 1 cycle
        cpu.cycles += 1;
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
        CPU, CPU_DEFAULT_SP, CPU_DEFAULT_STATUS, SYS_STACK_ADDR_START, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::instructions::Opcode;
    use crate::memory::Memory;

    #[test]
    fn rts_test() {
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(((MEM_OFFSET + 2) >> 8) as u8, SYS_STACK_ADDR_START);
        memory.write((MEM_OFFSET + 2) as u8, SYS_STACK_ADDR_START - 1);
        memory.write(Opcode::RTS.into(), 0x3042);
        memory.write(Opcode::LDYImm.into(), MEM_OFFSET + 3);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.pc = 0x3042;
        cpu.sp = CPU_DEFAULT_SP - 2;

        let init_cycles = cpu.cycles;
        let init_status = cpu.status;
        cpu.execute_next_instruction();
        assert_eq!(cpu.pc, MEM_OFFSET + 3);
        assert_eq!(cpu.memory.read(cpu.pc), Opcode::LDYImm.into());
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.sp, CPU_DEFAULT_SP);
        assert_eq!(cpu.status, init_status);
    }
}
