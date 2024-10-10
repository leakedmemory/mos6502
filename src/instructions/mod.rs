use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::cpu::CPU;

pub mod jumps;
pub mod load_ops;
pub mod stack_ops;
pub mod store_ops;

use jumps::*;
use load_ops::*;

#[derive(PartialEq, Copy, Clone)]
pub enum AddressingMode {
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Accumulator,
    Immediate,
    Implied,
    IndirectX,
    IndirectY,
    Relative,
    ZeroPage,
    ZeroPageX,
}

#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
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

pub trait Instruction {
    /// Executes the instruction with the setup provided in `cpu`.
    fn execute(&self, cpu: &mut CPU);

    /// Returns the addressing mode of the instruction.
    fn addressing_mode(&self) -> AddressingMode;

    /// Returns the opcode of the instruction.
    fn opcode(&self) -> u8;

    /// Returns the amount of cycles consumed by the instruction.
    fn cycles(&self) -> u8;

    /// Returns the amount of bytes consumed by the instruction.
    fn bytes(&self) -> u8;

    /// Returns the flags affected by the instruction.
    ///
    /// If the bitfield is set, it means that the flag if affected.
    fn flags_affected(&self) -> u8;
}

pub struct InstructionDecoder;

impl InstructionDecoder {
    pub fn from_byte(byte: u8) -> Box<dyn Instruction> {
        let opcode = Opcode::try_from(byte).expect(&format!("Invalid opcode: {:#04X}", byte));
        match opcode {
            Opcode::JMPAbs => Box::new(JMP::new(AddressingMode::Absolute)),
            Opcode::JMPInd => Box::new(JMP::new(AddressingMode::IndirectX)),
            Opcode::JSR => Box::new(JSR::new()),
            Opcode::RTS => Box::new(RTS::new()),
            Opcode::LDAImm => Box::new(LDA::new(AddressingMode::Immediate)),
            Opcode::LDAZpg => Box::new(LDA::new(AddressingMode::ZeroPage)),
            Opcode::LDAZpx => Box::new(LDA::new(AddressingMode::ZeroPageX)),
            Opcode::LDAAbs => Box::new(LDA::new(AddressingMode::Absolute)),
            Opcode::LDAAbx => Box::new(LDA::new(AddressingMode::AbsoluteX)),
            Opcode::LDAAby => Box::new(LDA::new(AddressingMode::AbsoluteY)),
            Opcode::LDAIdx => Box::new(LDA::new(AddressingMode::IndirectX)),
            Opcode::LDAIdy => Box::new(LDA::new(AddressingMode::IndirectY)),
            _ => unreachable!(),
        }
    }
}
