use crate::cpu::CPU;

/// bytes: 1
/// cycles: 6
/// flags affected: none
pub(in crate::cpu) fn rts(cpu: &mut CPU) {
    let addr = cpu.pop_addr_from_stack();
    cpu.pc = addr + 1; // takes 1 cycle
    cpu.cycles += 1;
}

#[cfg(test)]
mod tests {
    use crate::cpu::{
        Opcode, CPU, CPU_DEFAULT_SP, CPU_DEFAULT_STATUS, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::memory::Memory;

    #[test]
    fn rts_test() {
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::JSR as u8, MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);
        memory.write(0x30, MEM_OFFSET + 2);
        memory.write(Opcode::LDAImm as u8, 0x3042); // subroutine
        memory.write(0x42, 0x3042 + 1);
        memory.write(Opcode::RTS as u8, 0x3042 + 2);
        memory.write(Opcode::LDYImm as u8, MEM_OFFSET + 3);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.execute_next_instruction(); // JSR
        cpu.execute_next_instruction(); // LDA

        let rts_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.pc, MEM_OFFSET + 3);
        assert_eq!(cpu.memory.read(cpu.pc), Opcode::LDYImm as u8);
        assert_eq!(cpu.cycles - rts_cycles, CYCLES);
        assert_eq!(cpu.sp, CPU_DEFAULT_SP);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }
}
