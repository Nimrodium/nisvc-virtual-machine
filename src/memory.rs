use crate::{
    constant::{self, RegisterWidth},
    mmio,
};

// memory.rs
// memory interaction
pub type Bytes = Vec<u8>;

pub struct Memory {
    pub program: Bytes,
    pub ram: Bytes,
    pub mmio_base: RegisterWidth,
    pub program_base: RegisterWidth,
    // pub rom_exec_base: u64,
    pub ram_base: RegisterWidth,
    pub stack: Vec<RegisterWidth>,
    mmio: mmio::MMIO,
}
// [MMIO][PROGRAM][DATARAM]
impl Memory {
    /// initializes ram with bytes size as 0xFF
    fn init_ram(bytes: usize) -> Vec<u8> {
        let ram: Vec<u8> = vec![constant::INIT_RAM_VALUE; bytes];
        println!(
            "ram initalized as {bytes} bytes ({}KB)",
            bytes as f32 / 1000 as f32
        );
        ram
        // vec![]
    }
    pub fn new() -> Result<Self, String> {
        Ok(Memory {
            program: vec![],
            ram: Memory::init_ram(constant::RAM_SIZE as usize),
            mmio_base: 0, // always zero unless i put something under mmio
            program_base: constant::MMIO_ADDRESS_SPACE as RegisterWidth, // change this when i actually add an mmio system
            ram_base: constant::MMIO_ADDRESS_SPACE as RegisterWidth,
            stack: vec![], // start of ram aka rom.len()-1
            mmio: mmio::MMIO::new()?,
        })
    }

    pub fn push(&mut self, value: RegisterWidth) -> Result<(), String> {
        Err("push not implemented".to_string())
    }
    pub fn pop(&mut self) -> Result<RegisterWidth, String> {
        Err("push not implemented".to_string())
    }

    /// return a slice of bytes starting address and extending for bytes
    pub fn read_bytes(&mut self, address: RegisterWidth, bytes: usize) -> Result<Vec<u8>, String> {
        let mut byte_buffer: Vec<u8> = Vec::with_capacity(bytes);
        for n in 0..bytes {
            let byte_address = address + n as RegisterWidth;
            let byte = self.mmu_read(byte_address)?;
            byte_buffer.push(byte);
        }
        Ok(byte_buffer)
    }
    /// write a slice of bytes starting at address and extending for length of bytes inputed
    pub fn write_bytes(&mut self, address: RegisterWidth, bytes: &[u8]) -> Result<(), String> {
        for (n, byte) in bytes.iter().enumerate() {
            let byte_address = address + n as RegisterWidth;
            self.mmu_write(byte_address, *byte)?;
        }
        Ok(())
    }
    pub fn address_from_bytes(address_bytes: Vec<u8>) -> Result<RegisterWidth, String> {
        if constant::DEBUG_PRINT {
            println!("address bytes :: {address_bytes:?}");
        }
        let address_bytes_arr: [u8; size_of::<RegisterWidth>()] = match address_bytes.try_into() {
            Ok(arr) => arr,
            Err(why) => return Err(format!("error building address from bytes :: {why:?}")),
        };
        let address = RegisterWidth::from_le_bytes(address_bytes_arr);
        Ok(address)
    }

    /// returns byte at address
    pub fn mmu_read(&mut self, address: RegisterWidth) -> Result<u8, String> {
        // println!("{address} < {}", self.program_base);
        if address < self.program_base {
            // mmio address
            if constant::DEBUG_PRINT {
                println!("mmu_read decode {address} -> mmio::{address}")
            }
            Ok(self.mmio.mmio_read_handler(address))
        } else if address < self.ram_base {
            // rom address
            let physical_address = address - self.program_base;
            if constant::DEBUG_PRINT {
                print!("mmu_read decode {address} -> rom::{physical_address}")
            }
            self.read(physical_address, true)
        } else {
            // ram address
            let physical_address = address - self.ram_base;
            if constant::DEBUG_PRINT {
                print!("mmu_read decode {address} -> ram::{physical_address}")
            }
            self.read(physical_address, false)
        }
    }
    /// writes byte at address
    pub fn mmu_write(&mut self, address: RegisterWidth, byte: u8) -> Result<(), String> {
        // println!("{address} < {}", self.program_base);
        if address < self.program_base {
            // mmio address
            print!("mmu_write decode {address} -> mmio::{address}");

            self.mmio.mmio_write_handler(address, byte)
        } else if address < self.ram_base {
            // rom address
            return Err(format!("MemoryAccessViolation :: attempted write operation on read-only address {address:#x?}"));
        } else {
            // ram address
            let physical_address = address - self.ram_base;
            print!("mmu_write decode {address} -> ram::{physical_address}");

            self.write(physical_address, byte)
        }
    }

    pub fn flash_ram(&mut self, ram_image: &[u8]) -> Result<(), String> {
        if constant::DEBUG_PRINT {
            println!(
                "flashing ram image\nhead: {}\nimage size {}b",
                self.ram_base,
                ram_image.len()
            );
        }
        self.ram.fill(constant::INIT_RAM_VALUE);
        if constant::DEBUG_PRINT {
            println!("first ram address {}", self.ram[0]);
        }
        self.write_bytes(self.ram_base, ram_image)
    }

    fn read(&self, physical_address: RegisterWidth, is_program: bool) -> Result<u8, String> {
        let byte = match is_program {
            true => self.program.get(physical_address as usize).ok_or(format!(
                "MemoryAccessViolation :: attempted read operation on invalid rom address {physical_address:#x?}"
            )),
            false => self.ram.get(physical_address as usize).ok_or(format!(
                "MemoryAccessViolation :: attempted read operation on invalid ram address {physical_address:#x?}"
            )),
        }?;
        if constant::DEBUG_PRINT {
            println!("-> [ {byte} ]");
        }
        Ok(*byte)
    }

    fn write(&mut self, physical_address: RegisterWidth, byte: u8) -> Result<(), String> {
        let byte_reference = self.ram.get_mut(physical_address as usize).ok_or(format!("MemoryAccessViolation :: attempted write operation on invalid ram address {physical_address:#x?}"))?;
        if constant::DEBUG_PRINT {
            println!(" -> old::[ {} ] new::[ {byte} ]", *byte_reference);
        }
        *byte_reference = byte;

        Ok(())
    }

    // fn mmio_read_handler(&self, address: RegisterWidth) -> Result<u8, String> {
    //     // self.mmio.mmio_route(address)
    //     // Err(format!("mmio not implemented"))
    // }
    // fn mmio_write_handler(&mut self, address: RegisterWidth, byte: u8) -> Result<(), String> {
    //     Err(format!("mmio not implemented"))
    // }
}
