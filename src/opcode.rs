use crate::cpu::Kind;

pub enum Operation {
    Nop,
    Cpy,
    Ldi,
    Load,
    Store,
    Add,
    Sub,
    Mult,
    Div,

    Or,
    Xor,
    And,
    Not,
    Shl,
    Shr,
    Rotl,
    Rotr,
    Neg,

    Jmp,
    Jifz,
    Jifnz,

    Inc,
    Dec,

    Push,
    Pop,

    Call,
    Ret,
    Fopen,
    Fread,
    Fwrite,
    Fseek,
    Fclose,
    //new

    //heap management
    Malloc,
    Realloc,
    Free,
    Memcpy,
    Memset,

    // floating point
    Itof,
    Ftoi,

    Fadd,
    Fsub,
    Fmul,
    Fdiv,
    Fmod,

    Breakpoint,
    HaltExe,
}

impl Operation {
    pub fn decode(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::Nop),
            0x01 => Some(Self::Cpy),
            0x02 => Some(Self::Ldi),
            0x03 => Some(Self::Load),
            0x04 => Some(Self::Store),
            0x05 => Some(Self::Add),
            0x06 => Some(Self::Sub),
            0x07 => Some(Self::Mult),
            0x08 => Some(Self::Div),
            0x09 => Some(Self::Or),
            0x0a => Some(Self::Xor),
            0x0b => Some(Self::And),
            0x0c => Some(Self::Not),
            0x0d => Some(Self::Shl),
            0x0e => Some(Self::Shr),
            0x0f => Some(Self::Rotl),
            0x10 => Some(Self::Rotr),
            0x11 => Some(Self::Neg),
            0x12 => Some(Self::Jmp),
            0x13 => Some(Self::Jifz),
            0x14 => Some(Self::Jifnz),
            0x16 => Some(Self::Inc),
            0x17 => Some(Self::Dec),
            0x18 => Some(Self::Push),
            0x19 => Some(Self::Pop),
            0x1a => Some(Self::Call),
            0x1b => Some(Self::Ret),
            0x1e => Some(Self::Fopen),
            0x1f => Some(Self::Fread),
            0x20 => Some(Self::Fwrite),
            0x21 => Some(Self::Fseek),
            0x22 => Some(Self::Fclose),

            // new
            0x23 => Some(Self::Malloc),
            0x24 => Some(Self::Realloc),
            0x25 => Some(Self::Free),

            0x26 => Some(Self::Memset),
            0x27 => Some(Self::Memcpy),

            0x28 => Some(Self::Itof),
            0x29 => Some(Self::Ftoi),
            0x2a => Some(Self::Fadd),
            0x2b => Some(Self::Fsub),
            0x2c => Some(Self::Fmul),
            0x2d => Some(Self::Fdiv),

            // 0x23 => Some(Self::Fadd),
            // 0x24 => Some(Self::Fsub),
            // 0x25 => Some(Self::Fmul),
            // 0x26 => Some(Self::Fdiv),
            0xfe => Some(Self::Breakpoint),
            0xff => Some(Self::HaltExe),
            _ => None,
        }
    }
    pub fn get_operand_map(&self) -> Vec<Kind> {
        match self {
            Operation::Nop => vec![],
            Operation::Cpy => vec![Kind::MutableRegister, Kind::Register],
            Operation::Ldi => vec![Kind::MutableRegister, Kind::Immediate],
            Operation::Load => todo!(),
            Operation::Store => todo!(),
            Operation::Add => todo!(),
            Operation::Sub => todo!(),
            Operation::Mult => todo!(),
            Operation::Div => todo!(),
            Operation::Or => todo!(),
            Operation::Xor => todo!(),
            Operation::And => todo!(),
            Operation::Not => todo!(),
            Operation::Shl => todo!(),
            Operation::Shr => todo!(),
            Operation::Rotl => todo!(),
            Operation::Rotr => todo!(),
            Operation::Neg => todo!(),
            Operation::Jmp => todo!(),
            Operation::Jifz => todo!(),
            Operation::Jifnz => todo!(),
            Operation::Inc => todo!(),
            Operation::Dec => todo!(),
            Operation::Push => todo!(),
            Operation::Pop => todo!(),
            Operation::Call => todo!(),
            Operation::Ret => todo!(),
            Operation::Fopen => todo!(),
            Operation::Fread => todo!(),
            Operation::Fwrite => todo!(),
            Operation::Fseek => todo!(),
            Operation::Fclose => todo!(),
            Operation::Breakpoint => todo!(),
            Operation::HaltExe => todo!(),
        }
    }
}
