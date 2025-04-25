use std::{fs::File, io::Read};

use crate::{constant::SIGNATURE, ExecutionError};

pub struct NISVCEF {
    entry_point: u64,
    image: Vec<u8>,
}

impl NISVCEF {
    pub fn load_file(file: &str) -> Result<Self, ExecutionError> {
        let open_file = File::open(file)
            .map_err(|e| ExecutionError::new(format!("could not open {file}: {e}")))?;
        let image: Vec<u8> = Vec::new();
        open_file
            .read_to_end(&mut image)
            .map_err(|e| ExecutionError::new(format!("could not read {file}: {e}")))?;
    let read_signature = &open_file[]ss
    if SIGNATURE == 
        
    }
}
