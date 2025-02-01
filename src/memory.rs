use std::{fs::File, io::Read, path::Path};

use crate::constant::{self, DATAROM_LENGTH_LOCATION};

// memory.rs
// memory interaction
pub type Bytes = Vec<u8>;
pub type MemoryAddress = Vec<u8>; // 72bits/9bytes actually
pub struct Memory {
    ram: Bytes, // general purpose memory
    rom: Bytes, // program
    start_of_exec: usize,
    end_of_exec: usize,
}
impl Memory {
    pub fn new(binary: &Path) -> Result<Self, String> {
        // verify signature
        // locate start and end of exec
        // mark start of execution section
        //

        let mut binary_file: File = match File::open(binary) {
            Ok(file) => file,
            Err(why) => return Err(why.to_string()),
        };
        let mut program_signature_buffer = vec![0; constant::SIGNATURE.len()];
        match binary_file.read_exact(&mut program_signature_buffer) {
            Ok(_) => (),
            Err(why) => {
                let error = format!("could not read signature :: {}", why);
                return Err(error);
            }
        };

        let program_signature = match String::from_utf8(program_signature_buffer) {
            Ok(string) => string,
            Err(why) => {
                let error = format!("could not convert signature to string :: {}", why);
                return Err(error);
            }
        };

        if constant::SIGNATURE != program_signature {
            let why = format!(
                "exec format error; signature not valid, {} != {}",
                constant::SIGNATURE,
                program_signature
            );
            return Err(why);
        } else {
            println!("valid exec format");
        }
        let mut rom: Vec<u8> = vec![];
        match binary_file.read_to_end(&mut rom) {
            Ok(_) => (),
            Err(why) => {
                let error = format!("failed to read file into rom :: {}", why);
                return Err(error);
            }
        };

        let mut head = 0;
        // read header data -- VVV --
        //
        // read data length
        // first u64 after the signature is size of data section in bytes
        let data_rom_length = u64::from_le_bytes(match &rom[head..head + 8].try_into() {
            Ok(array) => *array,
            Err(why) => {
                let error = format!("failed to read datarom length :: {}", why);
                return Err(error);
            }
        });
        head += 8; // pass the datarom length
                   // read exec length
                   // next 8 bytes after datarom length
        let start_of_exec = head + data_rom_length as usize;

        let exec_rom_length = u64::from_le_bytes(match &rom[head..head + 8].try_into() {
            Ok(array) => *array,
            Err(why) => {
                let error = format!("failed to read execrom length :: {}", why);
                return Err(error);
            }
        });
        head += 8;
        let end_of_exec = head + exec_rom_length as usize;

        Ok(Memory {
            ram: vec![],
            rom,
            start_of_exec,
            end_of_exec,
        })
    }
}
