// opcode.rs
// opcode defintitions
// pub type OpcodeRegistry = HashMap<String, Opcode>;

// #[derive(Debug, Clone)]
// pub struct Opcode {
//     name: String,
//     code: u16,
//     fields: usize,
// }
//
use crate::constant::OpcodeSize;
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
