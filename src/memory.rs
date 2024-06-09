use std::cell::RefCell;
use std::rc::Rc;

use crate::cpu::{POWER_ON_RESET_ADDR_H, POWER_ON_RESET_ADDR_L};

// 16-bit address bus == 2^16 == 64KB
const MEMORY_SIZE: usize = 64 * 1024;

pub struct Memory {
    memory: [u8; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut memory = [0; MEMORY_SIZE];
        memory[POWER_ON_RESET_ADDR_L as usize] = 0x00;
        memory[POWER_ON_RESET_ADDR_H as usize] = 0x02;

        Rc::new(RefCell::new(Self { memory }))
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write(&mut self, byte: u8, addr: u16) {
        self.memory[addr as usize] = byte;
    }
}
