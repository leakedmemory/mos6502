use crate::cpu::CPU;

/// bytes: 2
/// cycles: 3
/// flags affected: none
pub(in crate::cpu) fn sty_zero_page(cpu: &mut CPU) {
    let addr = cpu.fetch_byte();
    cpu.write_byte(cpu.y, addr.into());
}

/// bytes: 2
/// cycles: 4
/// flags affected: none
pub(in crate::cpu) fn sty_zero_page_x(cpu: &mut CPU) {
    let zpg_addr = cpu.fetch_byte();
    let eff_addr = zpg_addr.wrapping_add(cpu.x);
    cpu.cycles += 1;
    cpu.write_byte(cpu.y, eff_addr.into());
}

/// bytes: 3
/// cycles: 4
/// flags affected: none
pub(in crate::cpu) fn sty_absolute(cpu: &mut CPU) {
    let addr = cpu.fetch_addr();
    cpu.write_byte(cpu.y, addr);
}

#[cfg(test)]
mod tests {
    use crate::cpu::{Opcode, CPU, CPU_DEFAULT_STATUS, UNRESERVED_MEMORY_ADDR_START};
    use crate::memory::Memory;

    #[test]
    fn stx_zero_page_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::STYZpg as u8, MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);
        memory.write(Opcode::STYZpg as u8, MEM_OFFSET + 2);
        memory.write(0x57, MEM_OFFSET + 3);
        memory.write(Opcode::STYZpg as u8, MEM_OFFSET + 4);
        memory.write(0x69, MEM_OFFSET + 5);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.y = 0x32;
        cpu.execute_next_instruction();
        assert_eq!(cpu.memory.read(0x42), cpu.y);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.y = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(memory.read(0x57), cpu.y);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.y = 0x80;
        cpu.execute_next_instruction();
        assert_eq!(cpu.memory.read(0x69), cpu.y);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }

    #[test]
    fn stx_zero_page_y_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::STYZpx as u8, MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);
        memory.write(Opcode::STYZpx as u8, MEM_OFFSET + 2);
        memory.write(0x57, MEM_OFFSET + 3);
        memory.write(Opcode::STYZpx as u8, MEM_OFFSET + 4);
        memory.write(0x69, MEM_OFFSET + 5);

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.y = 0x32;
        cpu.execute_next_instruction();
        assert_eq!(cpu.memory.read(X.wrapping_add(0x42).into()), cpu.y);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.y = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(memory.read(X.wrapping_add(0x57).into()), cpu.y);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.y = 0x80;
        cpu.execute_next_instruction();
        assert_eq!(cpu.memory.read(X.wrapping_add(0x69).into()), cpu.y);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }

    #[test]
    fn stx_absolute_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::STYAbs as u8, MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(Opcode::STYAbs as u8, MEM_OFFSET + 3);
        memory.write(0x97, MEM_OFFSET + 4);
        memory.write(0x26, MEM_OFFSET + 5);
        memory.write(Opcode::STYAbs as u8, MEM_OFFSET + 6);
        memory.write(0x70, MEM_OFFSET + 7);
        memory.write(0x55, MEM_OFFSET + 8);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.y = 0x32;
        cpu.execute_next_instruction();
        assert_eq!(cpu.memory.read(0x8028), cpu.y);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.y = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(cpu.memory.read(0x2697), cpu.y);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.y = 0x80;
        cpu.execute_next_instruction();
        assert_eq!(cpu.memory.read(0x5570), cpu.y);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }
}
