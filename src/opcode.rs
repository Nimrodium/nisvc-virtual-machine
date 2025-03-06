use std::{collections::HashMap, fmt::Debug};

use crate::{
    constant::OpcodeSize,
    cpu::{VMError, VMErrorCode},
};

pub struct _Opcode {
    pub code: u8,
    pub name: String,
}
impl _Opcode {
    fn new(code: u8, name: &str) -> (u8, Self) {
        let s = Self {
            code,
            name: name.to_string(),
        };
        (code, s)
    }
}
pub struct OpcodeTable {
    table: HashMap<u8, _Opcode>,
}
impl OpcodeTable {
    pub fn new() -> Self {
        let table: HashMap<u8, _Opcode> = HashMap::from([
            _Opcode::new(0x0, "nop"),
            _Opcode::new(0x1, "mov"),
            _Opcode::new(0x2, "movim"),
            _Opcode::new(0x3, "load"),
            _Opcode::new(0x4, "store"),
            _Opcode::new(0x5, "add"),
            _Opcode::new(0x6, "sub"),
            _Opcode::new(0x7, "mult"),
            _Opcode::new(0x8, "div"),
            _Opcode::new(0x9, "or"),
            _Opcode::new(0xa, "xor"),
            _Opcode::new(0xb, "and"),
            _Opcode::new(0xc, "not"),
            _Opcode::new(0xd, "shl"),
            _Opcode::new(0xe, "shr"),
            _Opcode::new(0xf, "rotl"),
            _Opcode::new(0x10, "rotr"),
            _Opcode::new(0x11, "neg"),
            _Opcode::new(0x12, "jmp"),
            _Opcode::new(0x13, "jifz"),
            _Opcode::new(0x14, "jifnz"),
            _Opcode::new(0x15, "pr"),
            _Opcode::new(0x16, "inc"),
            _Opcode::new(0x17, "dec"),
            _Opcode::new(0x18, "push"),
            _Opcode::new(0x19, "pop"),
            _Opcode::new(0x1a, "call"),
            _Opcode::new(0x1b, "ret"),
            _Opcode::new(0x1c, "cache"),
            _Opcode::new(0x1d, "restore"),
        ]);
        Self { table }
    }
    pub fn decode(&self, code: u8) -> Result<&_Opcode, VMError> {
        match self.table.get(&code) {
            Some(op) => Ok(op),
            None => Err(VMError {
                code: VMErrorCode::InvalidOperationCode,
                reason: format!("{code:#x} is not a valid opcode"),
            }),
        }
    }
}
