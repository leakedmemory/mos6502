use crate::instructions::InstructionDecoder;
use crate::memory::Memory;

pub(crate) const CSF_ZERO: u8 = 0x02;
pub(crate) const CSF_NEGATIVE: u8 = 0x80;

pub(crate) const SYS_STACK_ADDR_START: u16 = 0x01FF;
pub(crate) const SYS_STACK_ADDR_END: u16 = 0x0100;
pub(crate) const UNRESERVED_MEMORY_ADDR_START: u16 = 0x0200;
pub(crate) const POWER_ON_RESET_ADDR_L: u16 = 0xFFFC;
pub(crate) const POWER_ON_RESET_ADDR_H: u16 = 0xFFFD;

pub(crate) const CPU_DEFAULT_ACC: u8 = 0;
pub(crate) const CPU_DEFAULT_X: u8 = 0;
pub(crate) const CPU_DEFAULT_Y: u8 = 0;
pub(crate) const CPU_DEFAULT_SP: u8 = 0xFF;
pub(crate) const CPU_DEFAULT_STATUS: u8 = 0x20;

pub struct CPU {
    pub(crate) acc: u8,
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) sp: u8,
    pub(crate) pc: u16,
    pub(crate) status: u8,
    pub(crate) cycles: u64,
    pub(crate) memory: Memory,
}

impl CPU {
    pub fn new(memory: Memory) -> Self {
        Self {
            acc: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0,
            status: 0,
            cycles: 0,
            memory,
        }
    }

    pub fn reset(&mut self) {
        self.acc = CPU_DEFAULT_ACC;
        self.x = CPU_DEFAULT_X;
        self.y = CPU_DEFAULT_Y;
        self.sp = CPU_DEFAULT_SP;
        self.pc = ((self.memory.read(POWER_ON_RESET_ADDR_H) as u16) << 8)
            | (self.memory.read(POWER_ON_RESET_ADDR_L) as u16);
        self.status = CPU_DEFAULT_STATUS;
        self.cycles = 7;
    }

    pub fn run(&mut self) -> ! {
        loop {
            // separated function to facilitate tests
            self.execute_next_instruction()
        }
    }

    pub(crate) fn execute_next_instruction(&mut self) {
        let opcode = self.fetch_byte();
        let instruction = InstructionDecoder::from_byte(opcode);
        instruction.execute(self);
    }

    #[inline]
    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    /// reads a byte from program counter and increments it in 1 cycle
    pub(crate) fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.read(self.pc);
        self.increment_pc();
        self.cycles += 1;
        byte
    }

    /// reads an addr from the program counter and increments it by 2,
    /// in 2 cycles
    pub(crate) fn fetch_addr(&mut self) -> u16 {
        let addr_l = self.fetch_byte() as u16;
        let addr_h = self.fetch_byte() as u16;
        (addr_h << 8) | addr_l
    }

    /// reads a byte from `addr` in 1 cycle
    pub(crate) fn read_byte(&mut self, addr: u16) -> u8 {
        let byte = self.memory.read(addr);
        self.cycles += 1;
        byte
    }

    /// reads an addr using the value in `low` as the low byte
    /// and in `high` as the high byte of the addr, in 2 cycles
    pub(crate) fn read_addr(&mut self, low: u16, high: u16) -> u16 {
        let addr_l = self.read_byte(low);
        let addr_h = self.read_byte(high);
        (addr_h as u16) << 8 | addr_l as u16
    }

    /// writes a byte into the `addr` in 1 cycle
    pub(crate) fn write_byte(&mut self, byte: u8, addr: u16) {
        self.memory.write(byte, addr);
        self.cycles += 1;
    }

    /// pushes a `byte` to the stack, wrapping around when ovewflowing or
    /// underflowing, in 1 cycle
    pub(crate) fn push_byte_to_stack(&mut self, byte: u8) {
        let stack_addr = self.sp as u16 | SYS_STACK_ADDR_END;
        self.memory.write(byte, stack_addr);
        self.sp = self.sp.wrapping_sub(1);
        self.cycles += 1;
    }

    /// pushes an `addr` to the stack, wrapping around when overflowing or
    /// underflowing, in 2 cycles
    pub(crate) fn push_addr_to_stack(&mut self, addr: u16) {
        let addr_h = (addr >> 8) as u8;
        let addr_l = addr as u8;
        self.push_byte_to_stack(addr_h);
        self.push_byte_to_stack(addr_l);
    }

    /// pops a byte from the stack, wrapping around when overflowing or
    /// underflowing, in 2 cycles
    pub(crate) fn pop_byte_from_stack(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1); // takes 1 cycle
        let stack_addr = self.sp as u16 | SYS_STACK_ADDR_END;
        let byte = self.memory.read(stack_addr);
        self.cycles += 2;
        byte
    }

    /// pops an addr from the stack, wrapping around when overflowing or
    /// underflowing, in 4 cycles
    pub(crate) fn pop_addr_from_stack(&mut self) -> u16 {
        let addr_l = self.pop_byte_from_stack();
        let addr_h = self.pop_byte_from_stack();
        (addr_h as u16) << 8 | addr_l as u16
    }

    #[inline(always)]
    pub(crate) fn byte_is_negative_int(byte: u8) -> bool {
        byte & 0x80 != 0
    }

    // often used to know the need of another add operation with the high 8 bits
    // of the address, since the 6502's adder circuit only works with 8 bits
    #[inline(always)]
    pub(crate) fn page_crossed(addr_a: u16, addr_b: u16) -> bool {
        (addr_a & 0xFF00) != (addr_b & 0xFF00)
    }
}
