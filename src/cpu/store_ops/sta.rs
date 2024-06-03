use crate::cpu::CPU;

/// bytes: 2
/// cycles: 3
/// flags affected: none
pub(in crate::cpu) fn sta_zero_page(cpu: &mut CPU) {
    let addr = cpu.fetch_byte();
    cpu.write_byte(cpu.acc, addr.into());
}

/// bytes: 2
/// cycles: 4
/// flags affected: none
pub(in crate::cpu) fn sta_zero_page_x(cpu: &mut CPU) {
    let zpg_addr = cpu.fetch_byte();
    let eff_addr = zpg_addr.wrapping_add(cpu.x);
    cpu.cycles += 1;
    cpu.write_byte(cpu.acc, eff_addr.into());
}

/// bytes: 3
/// cycles: 4
/// flags affected: none
pub(in crate::cpu) fn sta_absolute(cpu: &mut CPU) {
    let addr = cpu.fetch_addr();
    cpu.write_byte(cpu.acc, addr);
}

/// bytes: 3
/// cycles: 5
/// flags affected: none
pub(in crate::cpu) fn sta_absolute_x(cpu: &mut CPU) {
    let abs_addr = cpu.fetch_addr();
    let eff_addr = abs_addr.wrapping_add(cpu.x.into());
    cpu.cycles += 1;
    cpu.write_byte(cpu.acc, eff_addr);
}

/// bytes: 3
/// cycles: 5
/// flags affected: none
pub(in crate::cpu) fn sta_absolute_y(cpu: &mut CPU) {
    let abs_addr = cpu.fetch_addr();
    let eff_addr = abs_addr.wrapping_add(cpu.y.into());
    cpu.cycles += 1;
    cpu.write_byte(cpu.acc, eff_addr);
}

/// bytes: 2
/// cycles: 6
/// flags affected: none
pub(in crate::cpu) fn sta_indirect_x(cpu: &mut CPU) {
    let zpg_addr = cpu.fetch_byte();
    let ind_addr = zpg_addr.wrapping_add(cpu.x.into());
    cpu.cycles += 1;
    let eff_addr = cpu.read_addr(ind_addr.into(), ind_addr.wrapping_add(1).into());
    cpu.write_byte(cpu.acc, eff_addr);
}

/// bytes: 2
/// cycles: 6
/// flags affected: none
pub(in crate::cpu) fn sta_indirect_y(cpu: &mut CPU) {
    let zpg_addr = cpu.fetch_byte();
    let ind_addr = cpu.read_addr(zpg_addr.into(), zpg_addr.wrapping_add(1).into());
    let eff_addr = ind_addr.wrapping_add(cpu.y.into());
    cpu.cycles += 1;
    cpu.write_byte(cpu.acc, eff_addr);
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::cpu::{
        CPU, CPU_DEFAULT_STATUS, OPCODE_STA_ABS, OPCODE_STA_ABX, OPCODE_STA_ABY, OPCODE_STA_IDX,
        OPCODE_STA_IDY, OPCODE_STA_ZPG, OPCODE_STA_ZPX, UNRESERVED_MEMORY_ADDR_START,
    };
    use crate::memory::Memory;

    #[test]
    fn sta_zero_page_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 3;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_STA_ZPG, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(OPCODE_STA_ZPG, MEM_OFFSET + 2);
        memory.set(0x57, MEM_OFFSET + 3);
        memory.set(OPCODE_STA_ZPG, MEM_OFFSET + 4);
        memory.set(0x69, MEM_OFFSET + 5);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.acc = 0x32;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(0x42), cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.acc = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(0x57), cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.acc = 0x80;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(0x69), cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }

    #[test]
    fn sta_zero_page_x_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_STA_ZPX, MEM_OFFSET);
        memory.set(0x42, MEM_OFFSET + 1);
        memory.set(OPCODE_STA_ZPX, MEM_OFFSET + 2);
        memory.set(0x57, MEM_OFFSET + 3);
        memory.set(OPCODE_STA_ZPX, MEM_OFFSET + 4);
        memory.set(0x69, MEM_OFFSET + 5);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.acc = 0x32;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(X.wrapping_add(0x42).into()), cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.acc = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(X.wrapping_add(0x57).into()), cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.acc = 0x80;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(X.wrapping_add(0x69).into()), cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }

    #[test]
    fn sta_absolute_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 4;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;

        let mut memory = Memory::new();
        memory.set(OPCODE_STA_ABS, MEM_OFFSET);
        memory.set(0x28, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(OPCODE_STA_ABS, MEM_OFFSET + 3);
        memory.set(0x97, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(OPCODE_STA_ABS, MEM_OFFSET + 6);
        memory.set(0x70, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.acc = 0x32;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(0x8028), cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.acc = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(0x2697), cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.acc = 0x80;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(0x5570), cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }

    #[test]
    fn sta_absolute_x_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_STA_ABX, MEM_OFFSET);
        memory.set(0x28, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(OPCODE_STA_ABX, MEM_OFFSET + 3);
        memory.set(0x97, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(OPCODE_STA_ABX, MEM_OFFSET + 6);
        memory.set(0x70, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.acc = 0x32;
        cpu.execute_next_instruction();
        assert_eq!(
            memory.borrow().get(0x8028_u16.wrapping_add(X.into())),
            cpu.acc
        );
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.acc = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(
            memory.borrow().get(0x2697_u16.wrapping_add(X.into())),
            cpu.acc
        );
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.acc = 0x80;
        cpu.execute_next_instruction();
        assert_eq!(
            memory.borrow().get(0x5570_u16.wrapping_add(X.into())),
            cpu.acc
        );
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }

    #[test]
    fn sta_absolute_y_test() {
        const BYTES: u16 = 3;
        const CYCLES: u64 = 5;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();
        memory.set(OPCODE_STA_ABY, MEM_OFFSET);
        memory.set(0x28, MEM_OFFSET + 1);
        memory.set(0x80, MEM_OFFSET + 2);
        memory.set(OPCODE_STA_ABY, MEM_OFFSET + 3);
        memory.set(0x97, MEM_OFFSET + 4);
        memory.set(0x26, MEM_OFFSET + 5);
        memory.set(OPCODE_STA_ABY, MEM_OFFSET + 6);
        memory.set(0x70, MEM_OFFSET + 7);
        memory.set(0x55, MEM_OFFSET + 8);

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.acc = 0x32;
        cpu.execute_next_instruction();
        assert_eq!(
            memory.borrow().get(0x8028_u16.wrapping_add(Y.into())),
            cpu.acc
        );
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.acc = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(
            memory.borrow().get(0x2697_u16.wrapping_add(Y.into())),
            cpu.acc
        );
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.acc = 0x80;
        cpu.execute_next_instruction();
        assert_eq!(
            memory.borrow().get(0x5570_u16.wrapping_add(Y.into())),
            cpu.acc
        );
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }

    #[test]
    fn sta_indirect_x_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const X: u8 = 0xAC;

        let mut memory = Memory::new();

        memory.set(OPCODE_STA_IDX, MEM_OFFSET);
        let zpg_addr = 0x08;
        memory.set(zpg_addr, MEM_OFFSET + 1);
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);
        memory.set(0x22, addr_l.into());
        memory.set(0x32, addr_h.into());

        memory.set(OPCODE_STA_IDX, MEM_OFFSET + 2);
        let zpg_addr = 0x57;
        memory.set(zpg_addr, MEM_OFFSET + 3);
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);
        memory.set(0x46, addr_l.into());
        memory.set(0x83, addr_h.into());

        memory.set(OPCODE_STA_IDX, MEM_OFFSET + 4);
        let zpg_addr = 0x69;
        memory.set(zpg_addr, MEM_OFFSET + 5);
        let addr_l = X.wrapping_add(zpg_addr);
        let addr_h = X.wrapping_add(zpg_addr + 1);
        memory.set(0x13, addr_l.into());
        memory.set(0x77, addr_h.into());

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.x = X;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.acc = 0x32;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(0x3222), cpu.acc);
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.acc = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(0x8346), cpu.acc);
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.acc = 0x80;
        cpu.execute_next_instruction();
        assert_eq!(memory.borrow().get(0x7713), cpu.acc);
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }

    #[test]
    fn sta_indirect_y_test() {
        const BYTES: u16 = 2;
        const CYCLES: u64 = 6;
        const MEM_OFFSET: u16 = UNRESERVED_MEMORY_ADDR_START;
        const Y: u8 = 0xAC;

        let mut memory = Memory::new();

        memory.set(OPCODE_STA_IDY, MEM_OFFSET);
        let zpg_addr = 0x08;
        memory.set(zpg_addr, MEM_OFFSET + 1);
        let addr_l = 0xDF;
        let addr_h = 0x32;
        memory.set(addr_l, zpg_addr.into());
        memory.set(addr_h, zpg_addr.wrapping_add(1).into());

        memory.set(OPCODE_STA_IDY, MEM_OFFSET + 2);
        let zpg_addr = 0x57;
        memory.set(zpg_addr, MEM_OFFSET + 3);
        let addr_l = 0xCC;
        let addr_h = 0x83;
        memory.set(addr_l, zpg_addr.into());
        memory.set(addr_h, zpg_addr.wrapping_add(1).into());

        memory.set(OPCODE_STA_IDY, MEM_OFFSET + 4);
        let zpg_addr = 0x69;
        memory.set(zpg_addr, MEM_OFFSET + 5);
        let addr_l = 0x86;
        let addr_h = 0x77;
        memory.set(addr_l, zpg_addr.into());
        memory.set(addr_h, zpg_addr.wrapping_add(1).into());

        let memory = RefCell::new(memory);
        let mut cpu = CPU::new(&memory);
        cpu.reset();
        cpu.y = Y;

        let init_pc = cpu.pc;
        let init_cycles = cpu.cycles;
        cpu.acc = 0x32;
        cpu.execute_next_instruction();
        assert_eq!(
            memory.borrow().get(0x32DF_u16.wrapping_add(Y.into())),
            cpu.acc
        );
        assert_eq!(cpu.pc - init_pc, BYTES);
        assert_eq!(cpu.cycles - init_cycles, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_first_exec = cpu.pc;
        let cycles_after_first_exec = cpu.cycles;
        cpu.acc = 0x00;
        cpu.execute_next_instruction();
        assert_eq!(
            memory.borrow().get(0x83CC_u16.wrapping_add(Y.into())),
            cpu.acc
        );
        assert_eq!(cpu.pc - pc_after_first_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_first_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);

        let pc_after_second_exec = cpu.pc;
        let cycles_after_second_exec = cpu.cycles;
        cpu.acc = 0x80;
        cpu.execute_next_instruction();
        assert_eq!(
            memory.borrow().get(0x7786_u16.wrapping_add(Y.into())),
            cpu.acc
        );
        assert_eq!(cpu.pc - pc_after_second_exec, BYTES);
        assert_eq!(cpu.cycles - cycles_after_second_exec, CYCLES);
        assert_eq!(cpu.status, CPU_DEFAULT_STATUS);
    }
}
