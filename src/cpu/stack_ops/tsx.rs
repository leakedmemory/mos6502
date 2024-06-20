use crate::cpu::{CPU, CSF_NEGATIVE, CSF_ZERO};

fn tsx_set_status(cpu: &mut CPU) {
    cpu.status &= !(CSF_ZERO | CSF_NEGATIVE);
    if cpu.x == 0 {
        cpu.status |= CSF_ZERO;
    } else if CPU::byte_is_negative_int(cpu.x) {
        cpu.status |= CSF_NEGATIVE;
    }
}

/// bytes: 1
/// cycles: 2
/// flags affected: N,Z
pub(in crate::cpu) fn tsx(cpu: &mut CPU) {
    cpu.x = cpu.sp;
    tsx_set_status(cpu);
    cpu.cycles += 1;
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::cpu::{
        Opcode, CPU, CPU_DEFAULT_STATUS, CSF_NEGATIVE, CSF_ZERO, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::memory::Memory;

    #[test]
    fn tsx_test() {
        const BYTES: u16 = 1;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let memory = Memory::new();
        memory.borrow_mut().write(Opcode::TSX as u8, MEM_OFFSET);
        memory.borrow_mut().write(Opcode::TSX as u8, MEM_OFFSET + 1);
        memory.borrow_mut().write(Opcode::TSX as u8, MEM_OFFSET + 2);

        let mut cpu = CPU::new(Rc::clone(&memory));
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.sp = 0x42;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.sp = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.sp = 0xFF;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0xFF);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }
}
