const CPU_CARRY: u8 = 0x1;
const CPU_ZERO: u8 = 0x2;
const CPU_INTERRUPT: u8 = 0x4;
const CPU_DECIMAL: u8 = 0x8;
const CPU_BREAK: u8 = 0x10;
const CPU_OVERFLOW: u8 = 0x40;
const CPU_NEGATIVE: u8 = 0x80;

const CPU_DEFAULT_ACC: u8 = 0;
const CPU_DEFAULT_X: u8 = 0;
const CPU_DEFAULT_Y: u8 = 0;
const CPU_DEFAULT_SP: u8 = 0xFF;
const CPU_DEFAULT_PC: u16 = 0x0000;
const CPU_DEFAULT_STATUS: u8 = 0xFF;

const OPCODE_LDA_IMMEDIATE: u8 = 0xA9;

pub const MEMORY_SIZE: usize = 65536; // 16-bit address bus == 2^16 == 64KB
pub type Memory = [u8; MEMORY_SIZE];

// ps register: NV1B DIZC
pub struct CPU {
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    status: u8,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            acc: CPU_DEFAULT_ACC,
            x: CPU_DEFAULT_X,
            y: CPU_DEFAULT_Y,
            sp: CPU_DEFAULT_SP,
            pc: CPU_DEFAULT_PC,
            status: CPU_DEFAULT_STATUS,
        }
    }

    fn reset(&mut self) {
        self.acc = CPU_DEFAULT_ACC;
        self.x = CPU_DEFAULT_X;
        self.y = CPU_DEFAULT_Y;
        self.sp = CPU_DEFAULT_SP;
        self.pc = CPU_DEFAULT_PC;
        self.status = CPU_DEFAULT_STATUS;
    }

    pub fn execute(&mut self, memory: &mut Memory, opcode: u8) {
        match opcode {
            OPCODE_LDA_IMMEDIATE => self.lda_immediate(memory),
            _ => println!("Opcode {} not implemented", opcode),
        }
    }

    pub fn fetch_instruction(&mut self, memory: &Memory) -> u8 {
        let opcode = memory[self.pc as usize];
        self.pc += 1;
        opcode
    }

    fn lda_immediate(&mut self, memory: &Memory) {
        let immediate = memory[self.pc as usize];
        self.acc = immediate;

        if immediate == 0 {
            self.status |= CPU_ZERO;
            self.status &= !CPU_NEGATIVE;
        } else if Self::is_negative_byte(immediate) {
            self.status |= CPU_NEGATIVE;
            self.status &= !CPU_ZERO;
        }
    }

    #[inline(always)]
    fn is_negative_byte(value: u8) -> bool {
        value & 0x80 != 0
    }
}
