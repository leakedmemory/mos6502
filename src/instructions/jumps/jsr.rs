use crate::cpu::CPU;

/// bytes: 3
/// cycles: 6
/// flags affected: none
pub(crate) fn jsr(cpu: &mut CPU) {
    let addr = cpu.fetch_addr();
    cpu.push_addr_to_stack(cpu.pc - 1);
    cpu.pc = addr; // takes 1 cycle
    cpu.cycles += 1;
}

#[cfg(test)]
mod tests {
    use crate::cpu::{
        Opcode, CPU, CPU_DEFAULT_SP, CPU_DEFAULT_STATUS, SYS_STACK_ADDR_END,
        UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::memory::Memory;

    #[test]
    fn jsr_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::JSR as u8, MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);
        memory.write(0x30, MEM_OFFSET + 2);
        memory.write(Opcode::LDAImm as u8, 0x3042);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.pc, 0x3042);
        assert_eq!(cpu.memory.read(cpu.pc), Opcode::LDAImm as u8);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.sp, CPU_DEFAULT_SP.wrapping_sub(2));
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let stack_pc_l = cpu
            .memory
            .read(cpu.sp.wrapping_add(1) as u16 | SYS_STACK_ADDR_END);
        let stack_pc_h = cpu
            .memory
            .read(cpu.sp.wrapping_add(2) as u16 | SYS_STACK_ADDR_END);
        let stack_pc = (stack_pc_h as u16) << 8 | stack_pc_l as u16;
        assert_eq!(stack_pc + 1 - init_pc, BYTES);
    }
}
