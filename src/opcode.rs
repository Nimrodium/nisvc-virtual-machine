// opcode.rs
// opcode defintitions
// pub type OpcodeRegistry = HashMap<String, Opcode>;

use std::{collections::HashMap, f64::consts::TAU, fmt::Debug};

// #[derive(Debug, Clone)]
// pub struct Opcode {
//     name: String,
//     code: u16,
//     fields: usize,
// }
//
use crate::{
    constant::OpcodeSize,
    cpu::{VMError, VMErrorCode},
};
macro_rules! decode {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::TryFrom<OpcodeSize> for $name {
            type Error = ();

            fn try_from(v: OpcodeSize) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as OpcodeSize => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
    }
}
decode!(
    #[derive(Debug)]
    pub enum Opcode {
        // data loading and storing
        Nop = 0x00 as isize,
        Mov = 0x01 as isize,
        Movim = 0x02 as isize,
        Load = 0x03 as isize,
        Store = 0x04 as isize,
        // arithmetic
        Add = 0x05 as isize,
        Sub = 0x06 as isize,
        Mult = 0x07 as isize,
        Div = 0x08 as isize,

        // bitwise
        Or = 0x09 as isize,
        Xor = 0x0a as isize,
        And = 0x0b as isize,
        Not = 0x0c as isize,
        Shl = 0x0d as isize,
        Shr = 0x0e as isize,

        Rotl = 0x0f as isize,
        Rotr = 0x10 as isize,
        Neg = 0x11 as isize,

        // control
        Jmp = 0x12 as isize,
        Jifz = 0x13 as isize,
        Jifnz = 0x14 as isize,

        Pr = 0x15 as isize,
        Inc = 0x16 as isize,
        Dec = 0x17 as isize,

        Push = 0x18 as isize,
        Pop = 0x19 as isize,

        Call = 0x1a as isize,
        Ret = 0x1b as isize,
        // special
        End_of_exec_section = 0xFFFF as isize,
    }
);

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
        ]);
        Self { table }
    }
    pub fn decode(&self, code: u8) -> Result<&_Opcode, VMError> {
        match self.table.get(&code) {
            Some(op) => Ok(op),
            None => Err(VMError {
                code: VMErrorCode::InvalidOperationCode,
                reason: format!("{code} is not a valid opcode"),
            }),
        }
    }
}
