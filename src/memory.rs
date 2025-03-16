use crate::{
    _very_verbose_println,
    constant::{self, RegisterWidth},
    cpu::{VMError, VMErrorCode},
    mmio, verbose_println, very_verbose_println, very_very_verbose_println, DisplayMode,
};

// memory.rs
// memory interaction
pub type Bytes = Vec<u8>;
// pub struct NewMemory<'a> {
//     pub program: Bytes,
//     pub ram: Bytes,
//     pub mmio_base: RegisterWidth,
//     pub program_base: RegisterWidth,
//     // pub rom_exec_base: u64,
//     pub ram_base: RegisterWidth,
//     pub stack: Vec<RegisterWidth>,
//     mmio: mmio::MMIO<'a>,
// }
// impl NewMemory {
//     fn init_ram(bytes: usize) -> Vec<u8> {
//         let ram: Vec<u8> = vec![constant::INIT_RAM_VALUE; bytes];
//         verbose_println!(
//             "ram initalized as {bytes} bytes ({}KB)",
//             bytes as f32 / 1000 as f32
//         );
//         ram
//     }
// }

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
    pub fn new(display: DisplayMode) -> Result<Self, VMError> {
        verbose_println!("initializing memory...");
        Ok(Memory {
            program: vec![],
            ram: Memory::init_ram(constant::RAM_SIZE as usize),
            mmio_base: 0, // always zero unless i put something under mmio
            program_base: constant::MMIO_ADDRESS_SPACE as RegisterWidth, // change this when i actually add an mmio system
            ram_base: constant::MMIO_ADDRESS_SPACE as RegisterWidth,
            stack: vec![], // start of ram aka rom.len()-1
            mmio: mmio::MMIO::new(display)?,
        })
    }

    pub fn push(&mut self, value: usize) -> Result<(), VMError> {
        Err(VMError {
            code: VMErrorCode::GenericError,
            reason: "push not implemented".to_string(),
        })
    }
    pub fn pop(&mut self) -> Result<usize, VMError> {
        Err(VMError {
            code: VMErrorCode::GenericError,
            reason: "pop not implemented".to_string(),
        })
    }

    /// return a slice of bytes starting address and extending for bytes
    pub fn read_bytes(&mut self, address: RegisterWidth, bytes: usize) -> Result<Vec<u8>, VMError> {
        let mut byte_buffer: Vec<u8> = Vec::with_capacity(bytes);
        for n in 0..bytes {
            let byte_address = address + n as RegisterWidth;
            let byte = self.mmu_read(byte_address)?;
            byte_buffer.push(byte);
        }
        Ok(byte_buffer)
    }
    /// write a slice of bytes starting at address and extending for length of bytes inputed
    pub fn write_bytes(&mut self, address: RegisterWidth, bytes: &[u8]) -> Result<(), VMError> {
        for (n, byte) in bytes.iter().enumerate() {
            let byte_address = address + n as RegisterWidth;
            self.mmu_write(byte_address, *byte)?;
        }
        Ok(())
    }
    pub fn address_from_bytes(address_bytes: Vec<u8>) -> Result<RegisterWidth, VMError> {
        let address_bytes_arr: [u8; size_of::<RegisterWidth>()] = match address_bytes.try_into() {
            Ok(arr) => arr,
            Err(why) => {
                return Err(VMError {
                    code: VMErrorCode::GenericError,
                    reason: format!("error building address from bytes :: {why:?}"),
                })
            }
        };
        let address = RegisterWidth::from_le_bytes(address_bytes_arr);
        Ok(address)
    }

    /// returns byte at address
    pub fn mmu_read(&mut self, address: RegisterWidth) -> Result<u8, VMError> {
        // println!("{address} < {}", self.program_base);
        if address < self.program_base {
            // mmio address
            if constant::DEBUG_PRINT {
                // println!("mmu_read decode {address} -> mmio::{address}")
                // verbose_println!("reading mmio::${address}");
            }
            Ok(self.mmio.mmio_read_handler(address))
        } else if address < self.ram_base {
            // rom address
            let physical_address = address - self.program_base;

            // print!("mmu_read decode {address} -> rom::{physical_address}")
            // verbose_println!("reading rom::${address}");

            self.read(physical_address, true)
        } else {
            // ram address
            let physical_address = address - self.ram_base;
            self.read(physical_address, false)
        }
    }
    /// writes byte at address
    pub fn mmu_write(&mut self, address: RegisterWidth, byte: u8) -> Result<(), VMError> {
        // println!("{address} < {}", self.program_base);
        if address < self.program_base {
            // mmio address
            // print!("mmu_write decode {address} -> mmio::{address}");

            self.mmio.mmio_write_handler(address, byte)
        } else if address < self.ram_base {
            // rom address
            return Err(VMError {
                code: VMErrorCode::MemoryAccessViolation,
                reason: format!("attempted write operation on read-only address {address:#x?}"),
            });
        } else {
            // ram address
            let physical_address = address - self.ram_base;
            // print!("mmu_write decode {address} -> ram::{physical_address}");

            self.write(physical_address, byte)
        }
    }

    pub fn flash_ram(&mut self, ram_image: &[u8]) -> Result<(), VMError> {
        if ram_image.len() == 0 {
            very_verbose_println!("no ram image provided");
        } else {
            very_verbose_println!("flashing ram image of {} byte(s)...\n", ram_image.len());
            self.ram.fill(constant::INIT_RAM_VALUE);
            self.write_bytes(self.ram_base, ram_image)?;
        };
        Ok(())
    }

    fn read(&self, physical_address: RegisterWidth, is_program: bool) -> Result<u8, VMError> {
        let byte = if is_program {
            match self.program.get(physical_address as usize) {
                Some(b) => b,
                None => {
                    return Err(VMError {
                        code: VMErrorCode::MemoryAccessViolation,
                        reason: format!(
                        "attempted read operation on invalid rom address {physical_address:#x?}"
                    ),
                    })
                }
            }
        } else {
            match self.ram.get(physical_address as usize) {
                Some(b) => b,
                None => {
                    return Err(VMError {
                        code: VMErrorCode::MemoryAccessViolation,
                        reason: format!(
                        "attempted read operation on invalid ram address {physical_address:#x?}"
                    ),
                    })
                }
            }
        };
        let name = if is_program { "rom" } else { "ram" };
        very_very_verbose_println!("reading {name}::${physical_address} -> {byte}");
        Ok(*byte)
    }

    fn write(&mut self, physical_address: RegisterWidth, byte: u8) -> Result<(), VMError> {
        let byte_reference = match self.ram.get_mut(physical_address as usize) {
            Some(b) => b,
            None => {
                return Err(VMError {
                    code: VMErrorCode::MemoryAccessViolation,
                    reason: format!(
                        "attempted read operation on invalid ram address {physical_address:#x?}"
                    ),
                })
            }
        };
        if constant::DEBUG_PRINT {
            very_very_verbose_println!("${physical_address} <- {byte}");
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
