use crate::cpu::CPU;

/// bytes: 1
/// cycles: 4
/// flags affected: all
pub(in crate::cpu) fn plp(cpu: &mut CPU) {
    cpu.status = cpu.pop_byte_from_stack();
    // cycle 3 is a dummy read for internal timing
    cpu.cycles += 1;
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::cpu::{
        Opcode, CPU, CPU_DEFAULT_SP, CPU_DEFAULT_STATUS, CSF_NEGATIVE, CSF_ZERO,
        SYS_STACK_ADDR_END, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::memory::Memory;

    #[test]
    fn plp_test() {
        const BYTES: u16 = 1;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let memory = Memory::new();
        memory.borrow_mut().write(Opcode::PLP as u8, MEM_OFFSET);
        memory.borrow_mut().write(
            CPU_DEFAULT_STATUS,
            CPU_DEFAULT_SP as u16 | SYS_STACK_ADDR_END,
        );
        memory.borrow_mut().write(Opcode::PLP as u8, MEM_OFFSET + 1);
        memory.borrow_mut().write(
            CPU_DEFAULT_STATUS | CSF_ZERO,
            CPU_DEFAULT_SP.wrapping_sub(1) as u16 | SYS_STACK_ADDR_END,
        );
        memory.borrow_mut().write(Opcode::PLP as u8, MEM_OFFSET + 2);
        memory.borrow_mut().write(
            CPU_DEFAULT_STATUS | CSF_NEGATIVE,
            CPU_DEFAULT_SP.wrapping_sub(2) as u16 | SYS_STACK_ADDR_END,
        );

        let mut cpu = CPU::new(Rc::clone(&memory));
        cpu.reset();
        cpu.sp = CPU_DEFAULT_SP.wrapping_sub(3);

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
        assert_eq!(cpu.sp, CPU_DEFAULT_SP.wrapping_sub(2));
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);
        assert_eq!(cpu.sp, CPU_DEFAULT_SP.wrapping_sub(1));
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
        assert_eq!(cpu.sp, CPU_DEFAULT_SP);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
    }
}
