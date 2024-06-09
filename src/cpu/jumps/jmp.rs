use crate::cpu::{CPU, POWER_ON_RESET_ADDR_L};

/// bytes: 3
/// cycles: 3
/// flags affected: none
pub(in crate::cpu) fn jmp_abs(cpu: &mut CPU) {
    cpu.pc = cpu.fetch_addr();
}

/// bytes: 3
/// cycles: 5
/// flags affected: none
pub(in crate::cpu) fn jmp_ind(cpu: &mut CPU) {
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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::cpu::{
        CPU, CPU_DEFAULT_STATUS, OPCODE_JMP_ABS, OPCODE_JMP_IND, OPCODE_LDA_IMM,
        POWER_ON_RESET_ADDR_L, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::memory::Memory;

    #[test]
    fn jmp_abs_test() {
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let memory = Memory::new();
        memory.borrow_mut().write(OPCODE_JMP_ABS, MEM_OFFSET);
        memory.borrow_mut().write(0x42, MEM_OFFSET + 1);
        memory.borrow_mut().write(0x30, MEM_OFFSET + 2);
        memory.borrow_mut().write(OPCODE_LDA_IMM, 0x3042);

        let mut cpu = CPU::new(Rc::clone(&memory));
        cpu.reset();

        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.pc, 0x3042);
        assert_eq!(cpu.memory.borrow().read(cpu.pc), OPCODE_LDA_IMM);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }

    #[test]
    fn jmp_ind_test() {
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let memory = Memory::new();
        memory.borrow_mut().write(OPCODE_JMP_IND, MEM_OFFSET);
        memory.borrow_mut().write(0x42, MEM_OFFSET + 1);
        memory.borrow_mut().write(0x30, MEM_OFFSET + 2);
        memory.borrow_mut().write(0xAC, 0x3042);
        memory.borrow_mut().write(0x28, 0x3043);
        memory.borrow_mut().write(OPCODE_JMP_IND, 0x28AC);
        memory.borrow_mut().write(0xFF, 0x28AC + 1);
        memory.borrow_mut().write(0x51, 0x28AC + 2);
        memory.borrow_mut().write(0x76, 0x51FF); // hardware bug
        memory.borrow_mut().write(0x11, 0x5100);
        memory.borrow_mut().write(OPCODE_JMP_IND, 0x1176);
        memory
            .borrow_mut()
            .write(POWER_ON_RESET_ADDR_L as u8, 0x1176 + 1);
        memory
            .borrow_mut()
            .write((POWER_ON_RESET_ADDR_L >> 8) as u8, 0x1176 + 2);

        let mut cpu = CPU::new(Rc::clone(&memory));
        cpu.reset();

        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.pc, 0x28AC);
        assert_eq!(cpu.memory.borrow().read(cpu.pc), OPCODE_JMP_IND);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.pc, 0x1176);
        assert_eq!(cpu.memory.borrow().read(cpu.pc), OPCODE_JMP_IND);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        cpu.execute_next_instruction();
        memory.borrow_mut().write(OPCODE_LDA_IMM, MEM_OFFSET);
        assert_eq!(cpu.pc, UNRESERVED_MEMORY_ADDR_START);
        assert_eq!(cpu.memory.borrow().read(cpu.pc), OPCODE_LDA_IMM);
        assert_eq!(cpu.cycles, 7);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }
}
