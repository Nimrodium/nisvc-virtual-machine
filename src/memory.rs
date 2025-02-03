use crate::constant::{self};

// memory.rs
// memory interaction
pub type Bytes = Vec<u8>;

pub struct Memory {
    pub program: Bytes,
    pub ram: Bytes,
    pub mmio_base: u64,
    pub rom_base: u64,
    // pub rom_exec_base: u64,
    pub ram_base: u64,
}
// [MMIO][PROGRAM][DATARAM]
impl Memory {
    pub fn new() -> Self {
        Memory {
            program: vec![],
            ram: vec![],
            mmio_base: 0, // always zero unless i put something under mmio
            rom_base: constant::MMIO_ADDRESS_SPACE as u64, // change this when i actually add an mmio system
            // always ZERO now. // rom_exec_base: 0,                              // start of program section
            ram_base: 0, // start of ram aka rom.len()-1
        }
    }
    /// return a slice of bytes starting address and extending for bytes
    pub fn read_bytes(&self, address: u64, bytes: usize) -> Result<Vec<u8>, String> {
        let mut byte_buffer: Vec<u8> = Vec::with_capacity(bytes);
        for n in 0..bytes {
            let byte_address = address + n as u64;
            let byte = self.mmu_read(byte_address)?;
            byte_buffer.push(byte);
        }
        Ok(byte_buffer)
    }
    /// write a slice of bytes starting at address and extending for length of bytes inputed
    pub fn write_bytes(&mut self, address: u64, bytes: &[u8]) -> Result<(), String> {
        for (n, byte) in bytes.iter().enumerate() {
            let byte_address = address + n as u64;
            self.mmu_write(byte_address, *byte)?;
        }
        Ok(())
    }
    pub fn address_from_bytes(address_bytes: &[u8]) -> Result<u64, String> {
        let address_bytes_arr: [u8; 8] = match address_bytes.try_into() {
            Ok(arr) => arr,
            Err(why) => return Err(format!("error building address from bytes :: {why}")),
        };
        let address = u64::from_le_bytes(address_bytes_arr);
        Ok(address)
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

    fn read(&self, physical_address: u64, is_program: bool) -> Result<u8, String> {
        let byte = match is_program {
            true => self.program.get(physical_address as usize).ok_or(format!(
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
