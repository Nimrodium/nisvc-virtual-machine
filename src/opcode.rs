use crate::cpu::RegHandle;
#[derive(Debug)]
pub enum Operation {
    Nop,
    Cpy {
        dest: RegHandle,
        src: RegHandle,
    },
    Ldi {
        dest: RegHandle,
        src: u64,
    },
    Load {
        dest: RegHandle,
        n: RegHandle,
        addr: u64,
    },
    Store {
        dest: RegHandle,
        n: RegHandle,
        src: RegHandle,
    },
    Add {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    Sub {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    Mult {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    Div {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },

    Or {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    Xor {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    And {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    Not {
        dest: RegHandle,
        op: RegHandle,
    },
    Shl {
        dest: RegHandle,
        n: RegHandle,
        src: RegHandle,
    },
    Shr {
        dest: RegHandle,
        n: RegHandle,
        src: RegHandle,
    },
    Rotl {
        dest: RegHandle,
        n: RegHandle,
        src: RegHandle,
    },
    Rotr {
        dest: RegHandle,
        n: RegHandle,
        src: RegHandle,
    },
    Neg {
        dest: RegHandle,
        op: RegHandle,
    },

    Jmp {
        addr: u64,
    },
    Jifz {
        addr: u64,
        condition: RegHandle,
    },
    Jifnz {
        addr: u64,
        condition: RegHandle,
    },

    Inc {
        reg: RegHandle,
    },
    Dec {
        reg: RegHandle,
    },

    Push {
        src: RegHandle,
    },
    Pop {
        dest: RegHandle,
    },

    Call {
        addr: u64,
    },
    Ret,
    // fopen fd_store filep_ptr filep_len
    // fwrite fd str_ptr str_len
    // fread fd buf_ptr buf_len
    // fclose fd
    // Fopen {
    //     dest_fd: RegHandle,
    //     file_path_str_ptr: RegHandle,
    //     file_path_str_len: RegHandle,
    // },
    // Fread {
    //     fd: RegHandle,
    //     buf_ptr: RegHandle,
    //     buf_len: RegHandle,
    // },
    // Fwrite {
    //     fd: RegHandle,
    //     buf_ptr: RegHandle,
    //     buf_len: RegHandle,
    // },
    // Fseek {
    //     fd: RegHandle,
    //     seek: RegHandle,
    //     direction: RegHandle,
    // },
    // Fclose {
    //     fd: RegHandle,
    // },
    // //new

    // //heap management
    // Malloc {
    //     dest_ptr: RegHandle,
    //     size: RegHandle,
    // },
    // Realloc {
    //     dest_ptr: RegHandle,
    //     ptr: RegHandle,
    //     new_size: RegHandle,
    // },
    // Free {
    //     ptr: RegHandle,
    // },
    // Memcpy {
    //     dest: RegHandle,
    //     n: RegHandle,
    //     src: RegHandle,
    // },
    // Memset {
    //     dest: RegHandle,
    //     n: RegHandle,
    //     value: RegHandle,
    // },

    // floating point
    Itof {
        destf: RegHandle,
        srci: RegHandle,
    },
    Ftoi {
        desti: RegHandle,
        srcf: RegHandle,
    },

    Fadd {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    Fsub {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    Fmult {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    Fdiv {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    Fmod {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    Mod {
        dest: RegHandle,
        op1: RegHandle,
        op2: RegHandle,
    },
    Int {
        code: u64,
    },
    Pushi {
        immediate: u64,
    },
    Breakpoint,
    HaltExe,
}

// impl Operation {
//     pub fn decode(byte: u8) -> Option<Self> {
//         match byte {
//             0x00 => Some(Self::Nop),
//             0x01 => Some(Self::Cpy),
//             0x02 => Some(Self::Ldi),
//             0x03 => Some(Self::Load),
//             0x04 => Some(Self::Store),
//             0x05 => Some(Self::Add),
//             0x06 => Some(Self::Sub),
//             0x07 => Some(Self::Mult),
//             0x08 => Some(Self::Div),
//             0x09 => Some(Self::Or),
//             0x0a => Some(Self::Xor),
//             0x0b => Some(Self::And),
//             0x0c => Some(Self::Not),
//             0x0d => Some(Self::Shl),
//             0x0e => Some(Self::Shr),
//             0x0f => Some(Self::Rotl),
//             0x10 => Some(Self::Rotr),
//             0x11 => Some(Self::Neg),
//             0x12 => Some(Self::Jmp),
//             0x13 => Some(Self::Jifz),
//             0x14 => Some(Self::Jifnz),
//             0x16 => Some(Self::Inc),
//             0x17 => Some(Self::Dec),
//             0x18 => Some(Self::Push),
//             0x19 => Some(Self::Pop),
//             0x1a => Some(Self::Call),
//             0x1b => Some(Self::Ret),
//             0x1e => Some(Self::Fopen),
//             0x1f => Some(Self::Fread),
//             0x20 => Some(Self::Fwrite),
//             0x21 => Some(Self::Fseek),
//             0x22 => Some(Self::Fclose),

//             // new
//             0x23 => Some(Self::Malloc),
//             0x24 => Some(Self::Realloc),
//             0x25 => Some(Self::Free),

//             0x26 => Some(Self::Memset),
//             0x27 => Some(Self::Memcpy),

//             0x28 => Some(Self::Itof),
//             0x29 => Some(Self::Ftoi),
//             0x2a => Some(Self::Fadd),
//             0x2b => Some(Self::Fsub),
//             0x2c => Some(Self::Fmult),
//             0x2d => Some(Self::Fdiv),

//             // 0x23 => Some(Self::Fadd),
//             // 0x24 => Some(Self::Fsub),
//             // 0x25 => Some(Self::Fmul),
//             // 0x26 => Some(Self::Fdiv),
//             0xfe => Some(Self::Breakpoint),
//             0xff => Some(Self::HaltExe),
//             _ => None,
//         }
//     }
//     pub fn get_operand_map(&self) -> Vec<Kind> {
//         match self {
//             Operation::Nop => vec![],
//             Operation::Cpy => vec![Kind::MutableRegister, Kind::Register],
//             Operation::Ldi => vec![Kind::MutableRegister, Kind::Immediate],
//             Operation::Load => vec![Kind::MutableRegister, Kind::Register, Kind::Address],
//             Operation::Store => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Add => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Sub => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Mult => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Div => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Or => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Xor => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::And => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Not => vec![Kind::MutableRegister, Kind::Register],
//             Operation::Shl => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Shr => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Rotl => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Rotr => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Neg => vec![Kind::MutableRegister, Kind::Register],
//             Operation::Jmp => vec![Kind::Address],
//             Operation::Jifz => vec![Kind::Address, Kind::Register],
//             Operation::Jifnz => vec![Kind::Address, Kind::Register],
//             Operation::Inc => vec![Kind::MutableRegister],
//             Operation::Dec => vec![Kind::MutableRegister],
//             Operation::Push => vec![Kind::Register],
//             Operation::Pop => vec![Kind::MutableRegister],
//             Operation::Call => vec![Kind::Address],
//             Operation::Ret => vec![],
//             Operation::Fopen => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Fread => vec![
//                 Kind::Register,
//                 Kind::Register,
//                 Kind::Register,
//                 Kind::Register,
//             ],
//             Operation::Fwrite => vec![Kind::Register, Kind::Register, Kind::Register],
//             Operation::Fseek => vec![Kind::Register, Kind::Register],
//             Operation::Fclose => vec![Kind::Register],
//             Operation::Breakpoint => vec![],
//             Operation::HaltExe => vec![],
//             Operation::Malloc => vec![Kind::MutableRegister, Kind::Register],
//             Operation::Realloc => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Free => vec![Kind::Register],
//             Operation::Memcpy => vec![Kind::Register, Kind::Register, Kind::Register],
//             Operation::Memset => vec![Kind::Register, Kind::Register, Kind::Register],
//             Operation::Itof => vec![Kind::MutableRegister, Kind::Register],
//             Operation::Ftoi => vec![Kind::MutableRegister, Kind::Register],
//             Operation::Fadd => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Fsub => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Fmult => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//             Operation::Fdiv => vec![Kind::MutableRegister, Kind::Register, Kind::Register],
//         }
//     }
// }
