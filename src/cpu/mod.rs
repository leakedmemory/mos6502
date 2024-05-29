mod jsr;
mod lda;
mod ldx;
mod ldy;

use crate::cpu::jsr::*;
use crate::cpu::lda::*;
use crate::cpu::ldx::*;
use crate::cpu::ldy::*;
use crate::memory::Memory;

const CSF_ZERO: u8 = 0x02;
const CSF_NEGATIVE: u8 = 0x80;

const SYS_STACK_ADDR_END: u16 = 0x100;
#[allow(dead_code)]
const UNRESERVED_MEMORY_ADDR_START: u16 = 0x0200; // only used in tests for now
pub(crate) const POWER_ON_RESET_ADDR_L: u16 = 0xFFFC;
pub(crate) const POWER_ON_RESET_ADDR_H: u16 = 0xFFFD;

const CPU_DEFAULT_ACC: u8 = 0;
const CPU_DEFAULT_X: u8 = 0;
const CPU_DEFAULT_Y: u8 = 0;
const CPU_DEFAULT_SP: u8 = 0xFF;
const CPU_DEFAULT_STATUS: u8 = 0x20;

// ==================== OPCODES START ====================

// JSR
const OPCODE_JSR: u8 = 0x20;

// LDA
const OPCODE_LDA_IMM: u8 = 0xA9;
const OPCODE_LDA_ZPG: u8 = 0xA5;
const OPCODE_LDA_ZPX: u8 = 0xB5;
const OPCODE_LDA_ABS: u8 = 0xAD;
const OPCODE_LDA_ABX: u8 = 0xBD;
const OPCODE_LDA_ABY: u8 = 0xB9;
const OPCODE_LDA_IDX: u8 = 0xA1;
const OPCODE_LDA_IDY: u8 = 0xB1;

// LDX
const OPCODE_LDX_IMM: u8 = 0xA2;
const OPCODE_LDX_ZPG: u8 = 0xA6;
const OPCODE_LDX_ZPY: u8 = 0xB6;
const OPCODE_LDX_ABS: u8 = 0xAE;
const OPCODE_LDX_ABY: u8 = 0xBE;

// LDY
const OPCODE_LDY_IMM: u8 = 0xA0;
const OPCODE_LDY_ZPG: u8 = 0xA4;
const OPCODE_LDY_ZPX: u8 = 0xB4;
const OPCODE_LDY_ABS: u8 = 0xAC;
const OPCODE_LDY_ABX: u8 = 0xBC;

// ==================== OPCODES END =====================

pub struct CPU {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    status: u8,
    cycles: u64,
    memory: Memory,
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
        self.pc = ((self.memory.get(POWER_ON_RESET_ADDR_H) as u16) << 8)
            | (self.memory.get(POWER_ON_RESET_ADDR_L) as u16);
        self.status = CPU_DEFAULT_STATUS;
        self.cycles = 7;
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.execute_next_instruction()
        }
    }

    fn execute_next_instruction(&mut self) {
        let opcode = self.fetch_byte();
        match opcode {
            // JSR
            OPCODE_JSR => jsr(self),
            // LDA
            OPCODE_LDA_IMM => lda_immediate(self),
            OPCODE_LDA_ZPG => lda_zero_page(self),
            OPCODE_LDA_ZPX => lda_zero_page_x(self),
            OPCODE_LDA_ABS => lda_absolute(self),
            OPCODE_LDA_ABX => lda_absolute_x(self),
            OPCODE_LDA_ABY => lda_absolute_y(self),
            OPCODE_LDA_IDX => lda_indirect_x(self),
            OPCODE_LDA_IDY => lda_indirect_y(self),
            // LDX
            OPCODE_LDX_IMM => ldx_immediate(self),
            OPCODE_LDX_ZPG => ldx_zero_page(self),
            OPCODE_LDX_ZPY => ldx_zero_page_y(self),
            OPCODE_LDX_ABS => ldx_absolute(self),
            OPCODE_LDX_ABY => ldx_absolute_y(self),
            // LDY
            OPCODE_LDY_IMM => ldy_immediate(self),
            OPCODE_LDY_ZPG => ldy_zero_page(self),
            OPCODE_LDY_ZPX => ldy_zero_page_x(self),
            OPCODE_LDY_ABS => ldy_absolute(self),
            OPCODE_LDY_ABX => ldy_absolute_x(self),
            // UNREACHABLE
            _ => unreachable!("invalid opcode: {:#X}", opcode),
        }
    }

    /// gets a byte from program counter and increments it in 1 cycle
    fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.get(self.pc);
        self.increment_pc();
        self.cycles += 1;
        byte
    }

    #[inline]
    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    /// gets a byte from addr in 1 cycle
    fn read_byte(&mut self, addr: u16) -> u8 {
        let byte = self.memory.get(addr);
        self.cycles += 1;
        byte
    }

    /// gets an addr from the program counter and increments it by 2,
    /// in 2 cycles
    fn fetch_addr(&mut self) -> u16 {
        let addr_l = self.fetch_byte() as u16;
        let addr_h = self.fetch_byte() as u16;
        (addr_h << 8) | addr_l
    }

    /// gets an addr using the value in `low` as the low byte
    /// and in `high` as the high byte of the addr, in 2 cycles
    fn read_addr(&mut self, low: u16, high: u16) -> u16 {
        let addr_l = self.read_byte(low);
        let addr_h = self.read_byte(high);
        (addr_h as u16) << 8 | addr_l as u16
    }

    /// pushes an addr to the stack, wrapping around when overflowing or
    /// underflowing, in 2 cycles
    fn push_addr_to_stack(&mut self, addr: u16) {
        let mut sp = (self.sp as u16) | SYS_STACK_ADDR_END;
        let addr_l = addr as u8;
        let addr_h = (addr >> 8) as u8;

        self.memory.set(addr_h, sp);
        self.sp = self.sp.wrapping_sub(1);
        sp = (self.sp as u16) | SYS_STACK_ADDR_END;

        self.memory.set(addr_l, sp);
        self.sp = self.sp.wrapping_sub(1);
        self.cycles += 2;
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
