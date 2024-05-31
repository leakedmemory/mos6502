use super::{CPU, CSF_NEGATIVE, CSF_ZERO};

fn ldx_set_status(cpu: &mut CPU) {
    cpu.status &= !(CSF_ZERO | CSF_NEGATIVE);
    if cpu.x == 0 {
        cpu.status |= CSF_ZERO;
    } else if CPU::byte_is_negative_int(cpu.x) {
        cpu.status |= CSF_NEGATIVE;
    }
}

/// bytes: 2
/// cycles: 2
/// flags affected: N and Z
pub(super) fn ldx_immediate(cpu: &mut CPU) {
    cpu.x = cpu.fetch_byte();
    ldx_set_status(cpu);
}

/// bytes: 2
/// cycles: 3
/// flags affected: N and Z
pub(super) fn ldx_zero_page(cpu: &mut CPU) {
    let addr = cpu.fetch_byte();
    cpu.x = cpu.read_byte(addr.into());
    ldx_set_status(cpu);
}

/// bytes: 2
/// cycles: 4
/// flags affected: N and Z
pub(super) fn ldx_zero_page_y(cpu: &mut CPU) {
    let byte = cpu.fetch_byte();
    let addr = cpu.y.wrapping_add(byte);
    cpu.cycles += 1;
    cpu.x = cpu.read_byte(addr.into());
    ldx_set_status(cpu);
}

/// bytes: 3
/// cycles: 4
/// flags affected: N and Z
pub(super) fn ldx_absolute(cpu: &mut CPU) {
    let addr = cpu.fetch_addr();
    cpu.x = cpu.read_byte(addr);
    ldx_set_status(cpu);
}

/// bytes: 3
/// cycles: 4 (+1 if page crossed)
/// flags affected: N and Z
pub(super) fn ldx_absolute_y(cpu: &mut CPU) {
    let abs_addr = cpu.fetch_addr();
    let eff_addr = abs_addr.wrapping_add(cpu.y.into());
    if CPU::page_crossed(abs_addr, eff_addr) {
        cpu.cycles += 1;
    }
    cpu.x = cpu.read_byte(eff_addr);
    ldx_set_status(cpu);
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::super::{
        CPU, CPU_DEFAULT_STATUS, CSF_NEGATIVE, CSF_ZERO, OPCODE_LDX_ABS, OPCODE_LDX_ABY,
        OPCODE_LDX_IMM, OPCODE_LDX_ZPG, OPCODE_LDX_ZPY, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::memory::Memory;

    #[test]
    fn ldx_immediate_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDX_IMM, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(OPCODE_LDX_IMM, MEM_OFFSET + 2);
        memory.set(0x00, MEM_OFFSET + 3);
        memory.set(OPCODE_LDX_IMM, MEM_OFFSET + 4);
        memory.set(0x80, MEM_OFFSET + 5);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn ldx_zero_page_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDX_ZPG, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(0x32, 0x42);
        memory.set(OPCODE_LDX_ZPG, MEM_OFFSET + 2);
        memory.set(0x57, MEM_OFFSET + 3);
        memory.set(0x00, 0x57);
        memory.set(OPCODE_LDX_ZPG, MEM_OFFSET + 4);
        memory.set(0x69, MEM_OFFSET + 5);
        memory.set(0x80, 0x69);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x32);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn ldx_zero_page_y_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDX_ZPY, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(0x32, Y.wrapping_add(0x42).into());
        memory.set(OPCODE_LDX_ZPY, MEM_OFFSET + 2);
        memory.set(0x57, MEM_OFFSET + 3);
        memory.set(0x00, Y.wrapping_add(0x57).into());
        memory.set(OPCODE_LDX_ZPY, MEM_OFFSET + 4);
        memory.set(0x69, MEM_OFFSET + 5);
        memory.set(0x80, Y.wrapping_add(0x69).into());

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x32);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn ldx_absolute_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDX_ABS, MEM_OFFSET);
        memory.set(0x28, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, 0x8028);
        memory.set(OPCODE_LDX_ABS, MEM_OFFSET + 3);
        memory.set(0x97, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, 0x2697);
        memory.set(OPCODE_LDX_ABS, MEM_OFFSET + 6);
        memory.set(0x70, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);
        memory.set(0x80, 0x5570);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn ldx_absolute_y_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDX_ABY, MEM_OFFSET);
        memory.set(0x28, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, (Y as u16).wrapping_add(0x8028));
        memory.set(OPCODE_LDX_ABY, MEM_OFFSET + 3);
        memory.set(0x53, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, (Y as u16).wrapping_add(0x2653));
        memory.set(OPCODE_LDX_ABY, MEM_OFFSET + 6);
        memory.set(0x22, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);
        memory.set(0x80, (Y as u16).wrapping_add(0x5522));

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn ldx_absolute_y_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDX_ABY, MEM_OFFSET);
        memory.set(0x60, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, (Y as u16).wrapping_add(0x8060));
        memory.set(OPCODE_LDX_ABY, MEM_OFFSET + 3);
        memory.set(0x54, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, (Y as u16).wrapping_add(0x2654));
        memory.set(OPCODE_LDX_ABY, MEM_OFFSET + 6);
        memory.set(0x83, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);
        memory.set(0x80, (Y as u16).wrapping_add(0x5583));

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.x, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }
}
