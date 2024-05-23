use mos6502::cpu::CPU;
use mos6502::memory::Memory;

fn main() -> ! {
    let mut memory = Memory::new();
    let mut cpu = CPU::new();
    cpu.reset(&memory);
    loop {
        let opcode = cpu.fetch_byte(&memory);
        cpu.execute(&mut memory, opcode);
    }
}
