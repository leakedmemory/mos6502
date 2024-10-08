use crate::cpu::{CPU, CSF_NEGATIVE, CSF_ZERO};

fn ldy_set_status(cpu: &mut CPU) {
    cpu.status &= !(CSF_ZERO | CSF_NEGATIVE);
    if cpu.y == 0 {
        cpu.status |= CSF_ZERO;
    } else if CPU::byte_is_negative_int(cpu.y) {
        cpu.status |= CSF_NEGATIVE;
    }
}

/// bytes: 2
/// cycles: 2
/// flags affected: N,Z
pub(in crate::cpu) fn ldy_immediate(cpu: &mut CPU) {
    cpu.y = cpu.fetch_byte();
    ldy_set_status(cpu);
}

/// bytes: 2
/// cycles: 3
/// flags affected: N,Z
pub(in crate::cpu) fn ldy_zero_page(cpu: &mut CPU) {
    let addr = cpu.fetch_byte();
    cpu.y = cpu.read_byte(addr.into());
    ldy_set_status(cpu);
}

/// bytes: 2
/// cycles: 4
/// flags affected: N,Z
pub(in crate::cpu) fn ldy_zero_page_x(cpu: &mut CPU) {
    let byte = cpu.fetch_byte();
    let addr = cpu.x.wrapping_add(byte);
    cpu.cycles += 1;
    cpu.y = cpu.read_byte(addr.into());
    ldy_set_status(cpu);
}

/// bytes: 3
/// cycles: 4
/// flags affected: N,Z
pub(in crate::cpu) fn ldy_absolute(cpu: &mut CPU) {
    let addr = cpu.fetch_addr();
    cpu.y = cpu.read_byte(addr);
    ldy_set_status(cpu);
}

/// bytes: 3
/// cycles: 4 (+1 if page crossed)
/// flags affected: N,Z
pub(in crate::cpu) fn ldy_absolute_x(cpu: &mut CPU) {
    let abs_addr = cpu.fetch_addr();
    let eff_addr = abs_addr.wrapping_add(cpu.x.into());
    if CPU::page_crossed(abs_addr, eff_addr) {
        cpu.cycles += 1;
    }
    cpu.y = cpu.read_byte(eff_addr);
    ldy_set_status(cpu);
}

#[cfg(test)]
mod tests {
    use crate::cpu::{
        Opcode, CPU, CPU_DEFAULT_STATUS, CSF_NEGATIVE, CSF_ZERO, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::memory::Memory;

    #[test]
    fn ldy_immediate_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYImm as u8, MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);
        memory.write(Opcode::LDYImm as u8, MEM_OFFSET + 2);
        memory.write(0x00, MEM_OFFSET + 3);
        memory.write(Opcode::LDYImm as u8, MEM_OFFSET + 4);
        memory.write(0x80, MEM_OFFSET + 5);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn ldy_zero_page_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYZpg as u8, MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);
        memory.write(0x32, 0x42);
        memory.write(Opcode::LDYZpg as u8, MEM_OFFSET + 2);
        memory.write(0x57, MEM_OFFSET + 3);
        memory.write(0x00, 0x57);
        memory.write(Opcode::LDYZpg as u8, MEM_OFFSET + 4);
        memory.write(0x69, MEM_OFFSET + 5);
        memory.write(0x80, 0x69);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x32);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn ldy_zero_page_x_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYZpx as u8, MEM_OFFSET);
        memory.write(0x42, MEM_OFFSET + 1);
        memory.write(0x32, X.wrapping_add(0x42).into());
        memory.write(Opcode::LDYZpx as u8, MEM_OFFSET + 2);
        memory.write(0x57, MEM_OFFSET + 3);
        memory.write(0x00, X.wrapping_add(0x57).into());
        memory.write(Opcode::LDYZpx as u8, MEM_OFFSET + 4);
        memory.write(0x69, MEM_OFFSET + 5);
        memory.write(0x80, X.wrapping_add(0x69).into());

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x32);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn ldy_absolute_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbs as u8, MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x42, 0x8028);
        memory.write(Opcode::LDYAbs as u8, MEM_OFFSET + 3);
        memory.write(0x97, MEM_OFFSET + 4);
        memory.write(0x26, MEM_OFFSET + 5);
        memory.write(0x00, 0x2697);
        memory.write(Opcode::LDYAbs as u8, MEM_OFFSET + 6);
        memory.write(0x70, MEM_OFFSET + 7);
        memory.write(0x55, MEM_OFFSET + 8);
        memory.write(0x80, 0x5570);

        let mut cpu = CPU::new(memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn ldy_absolute_x_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbx as u8, MEM_OFFSET);
        memory.write(0x28, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x42, (X as u16).wrapping_add(0x8028));
        memory.write(Opcode::LDYAbx as u8, MEM_OFFSET + 3);
        memory.write(0x53, MEM_OFFSET + 4);
        memory.write(0x26, MEM_OFFSET + 5);
        memory.write(0x00, (X as u16).wrapping_add(0x2653));
        memory.write(Opcode::LDYAbx as u8, MEM_OFFSET + 6);
        memory.write(0x22, MEM_OFFSET + 7);
        memory.write(0x55, MEM_OFFSET + 8);
        memory.write(0x80, (X as u16).wrapping_add(0x5522));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn ldy_absolute_x_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.write(Opcode::LDYAbx as u8, MEM_OFFSET);
        memory.write(0x60, MEM_OFFSET + 1);
        memory.write(0x80, MEM_OFFSET + 2);
        memory.write(0x42, (X as u16).wrapping_add(0x8060));
        memory.write(Opcode::LDYAbx as u8, MEM_OFFSET + 3);
        memory.write(0x54, MEM_OFFSET + 4);
        memory.write(0x26, MEM_OFFSET + 5);
        memory.write(0x00, (X as u16).wrapping_add(0x2654));
        memory.write(Opcode::LDYAbx as u8, MEM_OFFSET + 6);
        memory.write(0x83, MEM_OFFSET + 7);
        memory.write(0x55, MEM_OFFSET + 8);
        memory.write(0x80, (X as u16).wrapping_add(0x5583));

        let mut cpu = CPU::new(memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.y, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }
}
