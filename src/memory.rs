use std::{fs::File, io::Read, path::Path};

use crate::constant::{self};

// memory.rs
// memory interaction
pub type Bytes = Vec<u8>;
// pub type MemoryAddress = Vec<u8>; // 72bits/9bytes actually
pub enum Pool {
    Rom,
    Ram,
}
pub struct MemoryAddress {
    pub pool: Pool,
    pub address: u64,
}
pub struct Memory {
    pub ram: Bytes, // general purpose memory
    pub rom: Bytes, // program
    pub start_of_exec: usize,
    pub end_of_exec: usize,
}
impl Memory {
    pub fn new() -> Self {
        Memory {
            ram: vec![],
            rom: vec![],
            start_of_exec: 0,
            end_of_exec: 0,
        }
    }

    pub fn byte_slice(&self, start_address: &MemoryAddress, size: usize) -> Result<&[u8], String> {
        let end_address = start_address.address as usize + size;
        match start_address.pool {
            Pool::Rom => self
                .rom
                .get((start_address.address as usize)..end_address)
                .ok_or(format!(
                    "MemoryAccessViolation on rom read request {:#x?}-{:#x?}",
                    start_address.address, end_address
                )),
            Pool::Ram => self
                .ram
                .get((start_address.address as usize)..end_address)
                .ok_or(format!(
                    "MemoryAccessViolation on rom read request {:#x?}-{:#x?}",
                    start_address.address, end_address
                )),
        }
    }
}
//(start_address.address as usize)..=size
