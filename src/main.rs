use mos6502::cpu::{Memory, CPU, MEMORY_SIZE};

fn main() -> ! {
    let mut memory: Memory = [0u8; MEMORY_SIZE];
    let mut cpu = CPU::new();
    loop {
        let opcode = cpu.fetch_instruction(&memory);
        cpu.execute(&mut memory, opcode);
    }
}
