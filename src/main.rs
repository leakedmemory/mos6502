use mos6502::cpu::CPU;
use mos6502::memory::Memory;

fn main() -> ! {
    let mut memory = Memory::new();
    let mut cpu = CPU::new(&mut memory);
    cpu.reset();
    loop {
        let opcode = cpu.fetch_byte();
        cpu.execute(opcode);
    }
}
