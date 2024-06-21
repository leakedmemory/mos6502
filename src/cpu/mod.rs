mod jumps;
mod load_ops;
mod stack_ops;
mod store_ops;

use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;

use num_enum::TryFromPrimitive;

use crate::memory::Memory;

use jumps::*;
use load_ops::*;
use stack_ops::*;
use store_ops::*;

const CSF_ZERO: u8 = 0x02;
const CSF_NEGATIVE: u8 = 0x80;

const SYS_STACK_ADDR_END: u16 = 0x100;
pub(crate) const UNRESERVED_MEMORY_ADDR_START: u16 = 0x0200;
pub(crate) const POWER_ON_RESET_ADDR_L: u16 = 0xFFFC;
pub(crate) const POWER_ON_RESET_ADDR_H: u16 = 0xFFFD;

const CPU_DEFAULT_ACC: u8 = 0;
const CPU_DEFAULT_X: u8 = 0;
const CPU_DEFAULT_Y: u8 = 0;
const CPU_DEFAULT_SP: u8 = 0xFF;
const CPU_DEFAULT_STATUS: u8 = 0x20;

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    // JMP
    JMPAbs = 0x4C,
    JMPInd = 0x6C,

    // JSR
    JSR = 0x20,

    // LDA
    LDAImm = 0xA9,
    LDAZpg = 0xA5,
    LDAZpx = 0xB5,
    LDAAbs = 0xAD,
    LDAAbx = 0xBD,
    LDAAby = 0xB9,
    LDAIdx = 0xA1,
    LDAIdy = 0xB1,

    // LDX
    LDXImm = 0xA2,
    LDXZpg = 0xA6,
    LDXZpy = 0xB6,
    LDXAbs = 0xAE,
    LDXAby = 0xBE,

    // LDY
    LDYImm = 0xA0,
    LDYZpg = 0xA4,
    LDYZpx = 0xB4,
    LDYAbs = 0xAC,
    LDYAbx = 0xBC,

    // PHA
    PHA = 0x48,

    // PHP
    PHP = 0x08,

    // PLA
    PLA = 0x68,

    // PLP
    PLP = 0x28,

    // RTS
    RTS = 0x60,

    // STA
    STAZpg = 0x85,
    STAZpx = 0x95,
    STAAbs = 0x8D,
    STAAbx = 0x9D,
    STAAby = 0x99,
    STAIdx = 0x81,
    STAIdy = 0x91,

    // STX
    STXZpg = 0x86,
    STXZpy = 0x96,
    STXAbs = 0x8E,

    // STY
    STYZpg = 0x84,
    STYZpx = 0x94,
    STYAbs = 0x8C,

    // TSX
    TSX = 0xBA,

    // TXS
    TXS = 0x9A,
}

pub struct CPU {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    status: u8,
    cycles: u64,
    memory: Rc<RefCell<Memory>>,
}

impl CPU {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Self {
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
        self.pc = ((self.memory.borrow().read(POWER_ON_RESET_ADDR_H) as u16) << 8)
            | (self.memory.borrow().read(POWER_ON_RESET_ADDR_L) as u16);
        self.status = CPU_DEFAULT_STATUS;
        self.cycles = 7;
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.execute_next_instruction()
        }
    }

    fn execute_next_instruction(&mut self) {
        let opcode_byte = self.fetch_byte();
        let opcode =
            Opcode::try_from(opcode_byte).expect(&format!("Invalid opcode: {:#04X}", opcode_byte));

        match opcode {
            // JMP
            Opcode::JMPAbs => jmp_abs(self),
            Opcode::JMPInd => jmp_ind(self),

            // JSR
            Opcode::JSR => jsr(self),

            // LDA
            Opcode::LDAImm => lda_immediate(self),
            Opcode::LDAZpg => lda_zero_page(self),
            Opcode::LDAZpx => lda_zero_page_x(self),
            Opcode::LDAAbs => lda_absolute(self),
            Opcode::LDAAbx => lda_absolute_x(self),
            Opcode::LDAAby => lda_absolute_y(self),
            Opcode::LDAIdx => lda_indirect_x(self),
            Opcode::LDAIdy => lda_indirect_y(self),

            // LDX
            Opcode::LDXImm => ldx_immediate(self),
            Opcode::LDXZpg => ldx_zero_page(self),
            Opcode::LDXZpy => ldx_zero_page_y(self),
            Opcode::LDXAbs => ldx_absolute(self),
            Opcode::LDXAby => ldx_absolute_y(self),

            // LDY
            Opcode::LDYImm => ldy_immediate(self),
            Opcode::LDYZpg => ldy_zero_page(self),
            Opcode::LDYZpx => ldy_zero_page_x(self),
            Opcode::LDYAbs => ldy_absolute(self),
            Opcode::LDYAbx => ldy_absolute_x(self),

            // PHA
            Opcode::PHA => pha(self),

            // PHS
            Opcode::PHP => php(self),

            // PLA
            Opcode::PLA => pla(self),

            // PLP
            Opcode::PLP => plp(self),

            // RTS
            Opcode::RTS => rts(self),

            // STA
            Opcode::STAZpg => sta_zero_page(self),
            Opcode::STAZpx => sta_zero_page_x(self),
            Opcode::STAAbs => sta_absolute(self),
            Opcode::STAAbx => sta_absolute_x(self),
            Opcode::STAAby => sta_absolute_y(self),
            Opcode::STAIdx => sta_indirect_x(self),
            Opcode::STAIdy => sta_indirect_y(self),

            // STX
            Opcode::STXZpg => stx_zero_page(self),
            Opcode::STXZpy => stx_zero_page_y(self),
            Opcode::STXAbs => stx_absolute(self),

            // STY
            Opcode::STYZpg => sty_zero_page(self),
            Opcode::STYZpx => sty_zero_page_x(self),
            Opcode::STYAbs => sty_absolute(self),

            // TSX
            Opcode::TSX => tsx(self),

            // TXS
            Opcode::TXS => txs(self),
        }
    }

    #[inline]
    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    /// reads a byte from program counter and increments it in 1 cycle
    fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.borrow().read(self.pc);
        self.increment_pc();
        self.cycles += 1;
        byte
    }

    /// reads an addr from the program counter and increments it by 2,
    /// in 2 cycles
    fn fetch_addr(&mut self) -> u16 {
        let addr_l = self.fetch_byte() as u16;
        let addr_h = self.fetch_byte() as u16;
        (addr_h << 8) | addr_l
    }

    /// reads a byte from `addr` in 1 cycle
    fn read_byte(&mut self, addr: u16) -> u8 {
        let byte = self.memory.borrow().read(addr);
        self.cycles += 1;
        byte
    }

    /// reads an addr using the value in `low` as the low byte
    /// and in `high` as the high byte of the addr, in 2 cycles
    fn read_addr(&mut self, low: u16, high: u16) -> u16 {
        let addr_l = self.read_byte(low);
        let addr_h = self.read_byte(high);
        (addr_h as u16) << 8 | addr_l as u16
    }

    /// writes a byte into the `addr` in 1 cycle
    fn write_byte(&mut self, byte: u8, addr: u16) {
        self.memory.borrow_mut().write(byte, addr);
        self.cycles += 1;
    }

    /// pushes a `byte` to the stack, wrapping around when ovewflowing or
    /// underflowing, in 1 cycle
    fn push_byte_to_stack(&mut self, byte: u8) {
        let stack_addr = self.sp as u16 | SYS_STACK_ADDR_END;
        self.memory.borrow_mut().write(byte, stack_addr);
        self.sp = self.sp.wrapping_sub(1);
        self.cycles += 1;
    }

    /// pushes an `addr` to the stack, wrapping around when overflowing or
    /// underflowing, in 2 cycles
    fn push_addr_to_stack(&mut self, addr: u16) {
        let addr_h = (addr >> 8) as u8;
        let addr_l = addr as u8;
        self.push_byte_to_stack(addr_h);
        self.push_byte_to_stack(addr_l);
    }

    /// pops a byte from the stack, wrapping around when overflowing or
    /// underflowing, in 2 cycles
    fn pop_byte_from_stack(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1); // takes 1 cycle
        let stack_addr = self.sp as u16 | SYS_STACK_ADDR_END;
        let byte = self.memory.borrow().read(stack_addr);
        self.cycles += 2;
        byte
    }

    /// pops an addr from the stack, wrapping around when overflowing or
    /// underflowing, in 4 cycles
    fn pop_addr_from_stack(&mut self) -> u16 {
        let addr_l = self.pop_byte_from_stack();
        let addr_h = self.pop_byte_from_stack();
        (addr_h as u16) << 8 | addr_l as u16
    }

    #[inline(always)]
    fn byte_is_negative_int(byte: u8) -> bool {
        byte & 0x80 != 0
    }

    // often used to know the need of another add operation with the high 8 bits
    // of the address, since the 6502's adder circuit only works with 8 bits
    #[inline(always)]
    fn page_crossed(addr_a: u16, addr_b: u16) -> bool {
        (addr_a & 0xFF00) != (addr_b & 0xFF00)
    }
}
