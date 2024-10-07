use crate::cpu::{CPU, CSF_NEGATIVE, CSF_ZERO};

// TODO: clean this type of "set status" functions across the files when
// better understood how all of the flags properly work
fn pla_set_status(cpu: &mut CPU) {
    cpu.status &= !(CSF_ZERO | CSF_NEGATIVE);
    if cpu.acc == 0 {
        cpu.status |= CSF_ZERO;
    } else if CPU::byte_is_negative_int(cpu.acc) {
        cpu.status |= CSF_NEGATIVE;
    }
}

/// bytes: 1
/// cycles: 4
/// flags affected: N,Z
pub(crate) fn pla(cpu: &mut CPU) {
    cpu.acc = cpu.pop_byte_from_stack();
    // cycle 3 is a dummy read for internal timing
    cpu.cycles += 1;
    pla_set_status(cpu);
}

#[cfg(test)]
mod tests {
    use crate::cpu::{
        Opcode, CPU, CPU_DEFAULT_SP, CPU_DEFAULT_STATUS, CSF_NEGATIVE, CSF_ZERO,
        SYS_STACK_ADDR_END, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::memory::Memory;

    #[test]
    fn pla_test() {
        const BYTES: u16 = 1;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::PLA as u8, MEM_OFFSET);
        memory.write(0x80, CPU_DEFAULT_SP as u16 | SYS_STACK_ADDR_END);
        memory.write(Opcode::PLA as u8, MEM_OFFSET + 1);
        memory.write(
            0x00,
            CPU_DEFAULT_SP.wrapping_sub(1) as u16 | SYS_STACK_ADDR_END,
        );
        memory.write(Opcode::PLA as u8, MEM_OFFSET + 2);
        memory.write(
            0x42,
            CPU_DEFAULT_SP.wrapping_sub(2) as u16 | SYS_STACK_ADDR_END,
        );

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.sp = CPU_DEFAULT_SP.wrapping_sub(3);

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.sp, CPU_DEFAULT_SP.wrapping_sub(2));
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.sp, CPU_DEFAULT_SP.wrapping_sub(1));
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.sp, CPU_DEFAULT_SP);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }
}
