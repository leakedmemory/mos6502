use crate::cpu::{POWER_ON_RESET_ADDR_H, POWER_ON_RESET_ADDR_L, UNRESERVED_MEMORY_ADDR_START};

// 16-bit address bus == 2^16 == 64KB
const MEMORY_SIZE: usize = 64 * 1024;

// clone trait needed for testing purposes
// in some tests the memory is changed manually after passed into the cpu
#[derive(Copy, Clone)]
pub struct Memory {
    memory: [u8; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        let mut memory = [0; MEMORY_SIZE];
        memory[POWER_ON_RESET_ADDR_L as usize] = UNRESERVED_MEMORY_ADDR_START as u8;
        memory[POWER_ON_RESET_ADDR_H as usize] = (UNRESERVED_MEMORY_ADDR_START >> 8) as u8;

        Self { memory }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write(&mut self, byte: u8, addr: u16) {
        self.memory[addr as usize] = byte;
    }
}
