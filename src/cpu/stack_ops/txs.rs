use crate::cpu::CPU;

/// bytes: 1
/// cycles: 2
/// flags affected: none
pub(in crate::cpu) fn txs(cpu: &mut CPU) {
    cpu.sp = cpu.x;
    cpu.cycles += 1;
}

#[cfg(test)]
mod tests {
    use crate::cpu::{Opcode, CPU, CPU_DEFAULT_STATUS, UNRESERVED_MEMORY_ADDR_START};
    use crate::memory::Memory;

    #[test]
    fn txs_test() {
        const BYTES: u16 = 1;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::TXS as u8, MEM_OFFSET);
        memory.write(Opcode::TXS as u8, MEM_OFFSET + 1);
        memory.write(Opcode::TXS as u8, MEM_OFFSET + 2);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.x = 0x42;
        cpu.execute_next_instruction();
        assert_eq!(cpu.sp, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.x = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(cpu.sp, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.x = 0xFF;
        cpu.execute_next_instruction();
        assert_eq!(cpu.sp, 0xFF);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }
}
