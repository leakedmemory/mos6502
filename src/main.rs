use mos6502::cpu::CPU;
use mos6502::memory::Memory;

fn main() -> ! {
    let memory = Memory::new();
    let mut cpu = CPU::new(memory);
    cpu.reset();
    cpu.run();
}
