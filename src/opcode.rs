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
        src: RegHandle,
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
        condition_reg: RegHandle,
    },
    Jifnz {
        addr: u64,
        condition_reg: RegHandle,
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
