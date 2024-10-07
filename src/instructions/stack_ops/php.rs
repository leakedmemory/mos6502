use crate::cpu::CPU;

/// bytes: 1
/// cycles: 3
/// flags affected: none
pub(crate) fn php(cpu: &mut CPU) {
    cpu.push_byte_to_stack(cpu.status);
    // cycle 2 is just to decrement the SP and cycle 3 to actually push
    cpu.cycles += 1;
}

#[cfg(test)]
mod tests {
    use crate::cpu::{
        Opcode, CPU, CPU_DEFAULT_SP, CPU_DEFAULT_STATUS, CSF_NEGATIVE, CSF_ZERO,
        SYS_STACK_ADDR_END, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::memory::Memory;

    #[test]
    fn php_test() {
        const BYTES: u16 = 1;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::PHP as u8, MEM_OFFSET);
        memory.write(Opcode::PHP as u8, MEM_OFFSET + 1);
        memory.write(Opcode::PHP as u8, MEM_OFFSET + 2);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.sp, CPU_DEFAULT_SP.wrapping_sub(1));
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(
            cpu.status,
            cpu.memory
                .read(cpu.sp.wrapping_add(1) as u16 | SYS_STACK_ADDR_END)
        );

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.status = CPU_DEFAULT_STATUS | CSF_ZERO;
        cpu.execute_next_instruction();
        assert_eq!(cpu.sp, CPU_DEFAULT_SP.wrapping_sub(2));
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(
            cpu.status,
            cpu.memory
                .read(cpu.sp.wrapping_add(1) as u16 | SYS_STACK_ADDR_END)
        );

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.status = CPU_DEFAULT_STATUS | CSF_NEGATIVE;
        cpu.execute_next_instruction();
        assert_eq!(cpu.sp, CPU_DEFAULT_SP.wrapping_sub(3));
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(
            cpu.status,
            cpu.memory
                .read(cpu.sp.wrapping_add(1) as u16 | SYS_STACK_ADDR_END)
        );
    }
}
