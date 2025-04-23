use crate::{constant::UNINITIALIZED_MEMORY, ExecutionError};
pub struct Memory {
    physical: Vec<u8>,
    max: usize,
}

impl Memory {
    pub fn new(max: usize) -> Self {
        Self {
            physical: Vec::with_capacity(max),
            max,
        }
    }
    pub fn load(&mut self, image: Vec<u8>) {
        let undefined_bytes = self.max - image.len();
        self.physical.extend(image);
        self.physical
            .extend(std::iter::repeat(UNINITIALIZED_MEMORY).take(undefined_bytes));
    }
    pub fn read(&self, address: u64) -> Result<u8, ExecutionError> {
        self.physical
            .get(address as usize)
            .ok_or(ExecutionError::new(format!(
                "Memory Access Violation : address {}|{:#x} out of bounds",
                address, address
            )))
            .map(|v| *v)
    }
    pub fn write(&mut self, address: u64, value: u8) -> Result<(), ExecutionError> {
        if let Some(mem_cell) = self.physical.get_mut(address as usize) {
            *mem_cell = value;
        } else {
            return Err(ExecutionError::new(format!(
                "Memory Access Violation : address {}|{:#x} out of bounds",
                address, address
            )));
        }
        Ok(())
    }
    fn read_bytes(&self, address: u64, n: u64) -> Result<Vec<u8>, ExecutionError> {
        let mut buf = Vec::with_capacity(n as usize);
        for i in address..address + n {
            buf.push(self.read(i)?);
        }
        Ok(buf)
    }
    fn write_bytes(&mut self, address: u64, bytes: Vec<u8>) -> Result<(), ExecutionError> {
        for i in 0..bytes.len() as u64 {
            self.write(address + i, bytes[i as usize])?;
        }
        Ok(())
    }
    pub fn read_immediate(&self, address: u64) -> Result<(u64, u64), ExecutionError> {
        let size_byte = self.read(address)?;
        Ok((
            bytes_to_u64(&self.read_bytes(address + 1, size_byte as u64)?),
            (size_byte + 1) as u64,
        ))
    }
    pub fn read_address(&self, address: u64) -> Result<u64, ExecutionError> {
        Ok(bytes_to_u64(
            &self.read_bytes(address + 1, size_of::<u64>() as u64)?,
        ))
    }
}

fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let target_len = size_of::<u64>();
    let mut byte_buf: Vec<u8> = Vec::with_capacity(target_len);
    byte_buf.extend_from_slice(bytes);
    byte_buf.resize(target_len, 0);
    let byte_array: [u8; size_of::<u64>()] = match byte_buf.try_into() {
        Ok(v) => v,
        Err(_) => panic!("failed to build usize"),
    };
    u64::from_le_bytes(byte_array)
}
