use std::path::Path;

use crate::memory::{Memory, Operands};

pub struct Runtime {
    memory: Memory,
    pc: usize,
}

impl Runtime {
    /// load a program into Runtime
    pub fn new(binary: &Path) -> Result<Self, String> {
        todo!()
    }
    /// begins execution of binary
    pub fn exec(&mut self) -> Result<Self, String> {
        todo!()
    }

    pub fn fetch_operands(&mut self) -> Result<Operands, String> {
        todo!()
    }
}
