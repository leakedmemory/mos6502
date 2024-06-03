use crate::cpu::{CPU, CSF_NEGATIVE, CSF_ZERO};

fn lda_set_status(cpu: &mut CPU) {
    cpu.status &= !(CSF_ZERO | CSF_NEGATIVE);
    if cpu.acc == 0 {
        cpu.status |= CSF_ZERO;
    } else if CPU::byte_is_negative_int(cpu.acc) {
        cpu.status |= CSF_NEGATIVE;
    }
}

/// bytes: 2
/// cycles: 2
/// flags affected: N,Z
pub(in crate::cpu) fn lda_immediate(cpu: &mut CPU) {
    cpu.acc = cpu.fetch_byte();
    lda_set_status(cpu);
}

/// bytes: 2
/// cycles: 3
/// flags affected: N,Z
pub(in crate::cpu) fn lda_zero_page(cpu: &mut CPU) {
    let addr = cpu.fetch_byte();
    cpu.acc = cpu.read_byte(addr.into());
    lda_set_status(cpu);
}

/// bytes: 2
/// cycles: 4
/// flags affected: N,Z
pub(in crate::cpu) fn lda_zero_page_x(cpu: &mut CPU) {
    let byte = cpu.fetch_byte();
    let addr = cpu.x.wrapping_add(byte);
    cpu.cycles += 1;
    cpu.acc = cpu.read_byte(addr.into());
    lda_set_status(cpu);
}

/// bytes: 3
/// cycles: 4
/// flags affected: N,Z
pub(in crate::cpu) fn lda_absolute(cpu: &mut CPU) {
    let addr = cpu.fetch_addr();
    cpu.acc = cpu.read_byte(addr);
    lda_set_status(cpu);
}

/// bytes: 3
/// cycles: 4 (+1 if page crossed)
/// flags affected: N,Z
pub(in crate::cpu) fn lda_absolute_x(cpu: &mut CPU) {
    let abs_addr = cpu.fetch_addr();
    let eff_addr = abs_addr.wrapping_add(cpu.x.into());
    if CPU::page_crossed(abs_addr, eff_addr) {
        cpu.cycles += 1;
    }
    cpu.acc = cpu.read_byte(eff_addr);
    lda_set_status(cpu);
}

/// bytes: 3
/// cycles: 4 (+1 if page crossed)
/// flags affected: N,Z
pub(in crate::cpu) fn lda_absolute_y(cpu: &mut CPU) {
    let abs_addr = cpu.fetch_addr();
    let eff_addr = abs_addr.wrapping_add(cpu.y.into());
    if CPU::page_crossed(abs_addr, eff_addr) {
        cpu.cycles += 1;
    }
    cpu.acc = cpu.read_byte(eff_addr);
    lda_set_status(cpu);
}

/// bytes: 2
/// cycles: 6
/// flags affected: N,Z
pub(in crate::cpu) fn lda_indirect_x(cpu: &mut CPU) {
    let zpg_addr = cpu.fetch_byte();
    let addr = zpg_addr.wrapping_add(cpu.x);
    cpu.cycles += 1;
    let eff_addr = cpu.read_addr(addr.into(), addr.wrapping_add(1).into());
    cpu.acc = cpu.read_byte(eff_addr);
    lda_set_status(cpu);
}

/// bytes: 2
/// cycles: 5 (+1 if page crossed)
/// flags affected: N,Z
pub(in crate::cpu) fn lda_indirect_y(cpu: &mut CPU) {
    let zpg_addr = cpu.fetch_byte();
    let addr = cpu.read_addr(zpg_addr.into(), zpg_addr.wrapping_add(1).into());
    let eff_addr = addr.wrapping_add(cpu.y.into());
    if CPU::page_crossed(addr, eff_addr) {
        cpu.cycles += 1;
    }
    cpu.acc = cpu.read_byte(eff_addr);
    lda_set_status(cpu);
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::cpu::{
        CPU, CPU_DEFAULT_STATUS, CSF_NEGATIVE, CSF_ZERO, OPCODE_LDA_ABS, OPCODE_LDA_ABX,
        OPCODE_LDA_ABY, OPCODE_LDA_IDX, OPCODE_LDA_IDY, OPCODE_LDA_IMM, OPCODE_LDA_ZPG,
        OPCODE_LDA_ZPX, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::memory::Memory;

    #[test]
    fn lda_immediate_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 2;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_IMM, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(OPCODE_LDA_IMM, MEM_OFFSET + 2);
        memory.set(0x00, MEM_OFFSET + 3);
        memory.set(OPCODE_LDA_IMM, MEM_OFFSET + 4);
        memory.set(0x80, MEM_OFFSET + 5);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_zero_page_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ZPG, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(0x32, 0x42);
        memory.set(OPCODE_LDA_ZPG, MEM_OFFSET + 2);
        memory.set(0x57, MEM_OFFSET + 3);
        memory.set(0x00, 0x57);
        memory.set(OPCODE_LDA_ZPG, MEM_OFFSET + 4);
        memory.set(0x69, MEM_OFFSET + 5);
        memory.set(0x80, 0x69);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x32);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_zero_page_x_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ZPX, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(0x32, X.wrapping_add(0x42).into());
        memory.set(OPCODE_LDA_ZPX, MEM_OFFSET + 2);
        memory.set(0x57, MEM_OFFSET + 3);
        memory.set(0x00, X.wrapping_add(0x57).into());
        memory.set(OPCODE_LDA_ZPX, MEM_OFFSET + 4);
        memory.set(0x69, MEM_OFFSET + 5);
        memory.set(0x80, X.wrapping_add(0x69).into());

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x32);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ABS, MEM_OFFSET);
        memory.set(0x28, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, 0x8028);
        memory.set(OPCODE_LDA_ABS, MEM_OFFSET + 3);
        memory.set(0x97, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, 0x2697);
        memory.set(OPCODE_LDA_ABS, MEM_OFFSET + 6);
        memory.set(0x70, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);
        memory.set(0x80, 0x5570);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_x_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET);
        memory.set(0x28, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, (X as u16).wrapping_add(0x8028));
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET + 3);
        memory.set(0x53, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, (X as u16).wrapping_add(0x2653));
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET + 6);
        memory.set(0x22, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);
        memory.set(0x80, (X as u16).wrapping_add(0x5522));

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_x_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET);
        memory.set(0x60, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, (X as u16).wrapping_add(0x8060));
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET + 3);
        memory.set(0x54, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, (X as u16).wrapping_add(0x2654));
        memory.set(OPCODE_LDA_ABX, MEM_OFFSET + 6);
        memory.set(0x83, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);
        memory.set(0x80, (X as u16).wrapping_add(0x5583));

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_y_without_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET);
        memory.set(0x28, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, (Y as u16).wrapping_add(0x8028));
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET + 3);
        memory.set(0x53, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, (Y as u16).wrapping_add(0x2653));
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET + 6);
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
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_absolute_y_with_page_crossing_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET);
        memory.set(0x60, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(0x42, (Y as u16).wrapping_add(0x8060));
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET + 3);
        memory.set(0x54, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(0x00, (Y as u16).wrapping_add(0x2654));
        memory.set(OPCODE_LDA_ABY, MEM_OFFSET + 6);
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
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_indirect_x_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();

        memory.set(OPCODE_LDA_IDX, MEM_OFFSET);
        let zpg_addr = 0x08;
        memory.set(zpg_addr, MEM_OFFSET + 1);
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);
        memory.set(0x22, addr_l.into());
        memory.set(0x32, addr_h.into());
        memory.set(0x42, 0x3222);

        memory.set(OPCODE_LDA_IDX, MEM_OFFSET + 2);
        let zpg_addr = 0x57;
        memory.set(zpg_addr, MEM_OFFSET + 3);
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);
        memory.set(0x46, addr_l.into());
        memory.set(0x83, addr_h.into());
        memory.set(0x00, 0x8346);

        memory.set(OPCODE_LDA_IDX, MEM_OFFSET + 4);
        let zpg_addr = 0x69;
        memory.set(zpg_addr, MEM_OFFSET + 5);
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);
        memory.set(0x13, addr_l.into());
        memory.set(0x77, addr_h.into());
        memory.set(0x80, 0x7713);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_indirect_y_without_page_crossing_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();

        memory.set(OPCODE_LDA_IDY, MEM_OFFSET);
        let zpg_addr = 0x08;
        memory.set(zpg_addr, MEM_OFFSET + 1);
        let addr_l = 0x22;
        let addr_h = 0x32;
        memory.set(addr_l, zpg_addr.into());
        memory.set(addr_h, zpg_addr.wrapping_add(1).into());
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());
        memory.set(0x42, addr);

        memory.set(OPCODE_LDA_IDY, MEM_OFFSET + 2);
        let zpg_addr = 0x57;
        memory.set(zpg_addr, MEM_OFFSET + 3);
        let addr_l = 0x46;
        let addr_h = 0x83;
        memory.set(addr_l, zpg_addr.into());
        memory.set(addr_h, zpg_addr.wrapping_add(1).into());
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());
        memory.set(0x00, addr);

        memory.set(OPCODE_LDA_IDY, MEM_OFFSET + 4);
        let zpg_addr = 0x69;
        memory.set(zpg_addr, MEM_OFFSET + 5);
        let addr_l = 0x13;
        let addr_h = 0x77;
        memory.set(addr_l, zpg_addr.into());
        memory.set(addr_h, zpg_addr.wrapping_add(1).into());
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());
        memory.set(0x80, addr);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }

    #[test]
    fn lda_indirect_y_with_page_crossing_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();

        memory.set(OPCODE_LDA_IDY, MEM_OFFSET);
        let zpg_addr = 0x08;
        memory.set(zpg_addr, MEM_OFFSET + 1);
        let addr_l = 0xDF;
        let addr_h = 0x32;
        memory.set(addr_l, zpg_addr.into());
        memory.set(addr_h, zpg_addr.wrapping_add(1).into());
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());
        memory.set(0x42, addr);

        memory.set(OPCODE_LDA_IDY, MEM_OFFSET + 2);
        let zpg_addr = 0x57;
        memory.set(zpg_addr, MEM_OFFSET + 3);
        let addr_l = 0xCC;
        let addr_h = 0x83;
        memory.set(addr_l, zpg_addr.into());
        memory.set(addr_h, zpg_addr.wrapping_add(1).into());
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());
        memory.set(0x00, addr);

        memory.set(OPCODE_LDA_IDY, MEM_OFFSET + 4);
        let zpg_addr = 0x69;
        memory.set(zpg_addr, MEM_OFFSET + 5);
        let addr_l = 0x86;
        let addr_h = 0x77;
        memory.set(addr_l, zpg_addr.into());
        memory.set(addr_h, zpg_addr.wrapping_add(1).into());
        let addr = ((addr_h as u16) << 8 | addr_l as u16).wrapping_add(Y.into());
        memory.set(0x80, addr);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x42);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_ZERO);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.execute_next_instruction();
        assert_eq!(cpu.acc, 0x80);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS | CSF_NEGATIVE);
    }
}
