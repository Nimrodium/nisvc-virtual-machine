use std::{fs::File, io::Read, path::Path};

use crate::constant::{self};

// memory.rs
// memory interaction
pub type Bytes = Vec<u8>;
// pub type MemoryAddress = Vec<u8>; // 72bits/9bytes actually
// pub enum Pool {
//     Rom,
//     Ram,
// }
// pub struct MemoryAddress {
//     pub pool: Pool,
//     pub address: u64,
// }
// impl MemoryAddress {
//     pub fn new(bytes: Vec<u8>) -> Result<Self, String> {
//         let length = bytes.len();
//         if length != 9 {
//             return Err(format!(
//                 "invalid memory address, should be 9 bytes, {length} bytes passed"
//             ));
//         }
//         let pool: Pool = match bytes[0] {
//             0 => Pool::Rom,
//             1 => Pool::Ram,
//             _ => {
//                 return Err(format!(
//                     "invalid memory address! invalid pool byte {:#x?}",
//                     bytes[0]
//                 ))
//             }
//         };
//         let address: u64 = u64::from_le_bytes(match bytes.try_into() {
//             Ok(array) => array,
//             Err(why) => {
//                 let error = format!("failed to build address from :: {:#x?}", why);
//                 return Err(error);
//             }
//         });
//         Ok(MemoryAddress { pool, address })
//     }
// }
// pub struct Memory {
//     pub ram: Bytes, // general purpose memory
//     pub rom: Bytes, // program
//     pub start_of_exec: usize,
//     pub end_of_exec: usize,
// }
// impl Memory {
//     pub fn new() -> Self {
//         Memory {
//             ram: vec![],
//             rom: vec![],
//             start_of_exec: 0,
//             end_of_exec: 0,
//         }
//     }

//     pub fn byte_slice(&self, start_address: &MemoryAddress, size: usize) -> Result<&[u8], String> {
//         let end_address = start_address.address as usize + size;
//         match start_address.pool {
//             Pool::Rom => self
//                 .rom
//                 .get((start_address.address as usize)..end_address)
//                 .ok_or(format!(
//                     "MemoryAccessViolation on rom read request {:#x?}-{:#x?}",
//                     start_address.address, end_address
//                 )),
//             Pool::Ram => self
//                 .ram
//                 .get((start_address.address as usize)..end_address)
//                 .ok_or(format!(
//                     "MemoryAccessViolation on rom read request {:#x?}-{:#x?}",
//                     start_address.address, end_address
//                 )),
//         }
//     }
// }

struct Memory {
    rom: Bytes,
    ram: Bytes,
    mmio_base: u64,
    rom_base: u64,
    rom_exec_base: u64,
    ram_base: u64,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            rom: vec![],
            ram: vec![],
            mmio_base: 0,     // always zero unless i put something under mmio
            rom_base: 0,      // change this when i actually add an mmio system
            rom_exec_base: 0, // start of program section
            ram_base: 0,      // start of ram aka rom.len()-1
        }
    }

    /// returns byte at address
    pub fn mmu_read(&self, address: u64) -> Result<u8, String> {
        if address < self.rom_base {
            // mmio address
            self.mmio_read_handler(address)
        } else if address < self.ram_base {
            // rom address
            let physical_address = address - self.rom_base;
            self.read(physical_address, true)
        } else {
            // ram address
            let physical_address = address - self.ram_base;
            self.read(physical_address, false)
        }
    }
    /// writes byte to
    pub fn mmu_write(&mut self, address: u64, byte: u8) -> Result<(), String> {
        if address < self.rom_base {
            // mmio address
            self.mmio_write_handler(address, byte)
        } else if address < self.ram_base {
            // rom address
            return Err(format!("MemoryAccessViolation :: attempted write operation on read-only address {address:#x?}"));
        } else {
            // ram address
            let physical_address = address - self.ram_base;
            self.write(physical_address, byte)
        }
    }

    fn read(&self, physical_address: u64, is_rom: bool) -> Result<u8, String> {
        let byte = match is_rom {
            true => self.rom.get(physical_address as usize).ok_or(format!(
                "MemoryAccessViolation :: attempted read operation on invalid rom address {physical_address:#x?}"
            )),
            false => self.ram.get(physical_address as usize).ok_or(format!(
                "MemoryAccessViolation :: attempted read operation on invalid ram address {physical_address:#x?}"
            )),
        }?;
        Ok(*byte)
    }

    fn write(&mut self, physical_address: u64, byte: u8) -> Result<(), String> {
        let byte_reference = self.ram.get_mut(physical_address as usize).ok_or(format!("MemoryAccessViolation :: attempted write operation on invalid ram address {physical_address:#x?}"))?;
        *byte_reference = byte;
        Ok(())
    }

    fn mmio_read_handler(&self, address: u64) -> Result<u8, String> {
        todo!()
    }
    fn mmio_write_handler(&mut self, address: u64, byte: u8) -> Result<(), String> {
        todo!()
    }
}
