use std::{
    fs::File,
    io::Read,
    mem::transmute,
    ops::{Shl, Shr},
};

pub type RegHandle = u8;

use crossterm::style::Stylize;

use crate::{
    constant::{FRAME_POINTER, PROGRAM_COUNTER, STACK_POINTER, UNINITIALIZED_REGISTER},
    loader::NISVCEF,
    log_disassembly,
    memory::{bytes_to_u64, Memory},
    opcode::Operation,
    very_verbose_println, very_very_verbose_println, ExecutionError, GLOBAL_PROGRAM_COUNTER,
};

#[derive(Clone)]
enum RegWindow {
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
    B8,
    Q1,
    Q2,
    Q3,
    Q4,
    L,
    H,
    F,
}
impl RegWindow {
    fn to_suffix(&self) -> &str {
        match self {
            RegWindow::B1 => "b1",
            RegWindow::B2 => "b2",
            RegWindow::B3 => "b3",
            RegWindow::B4 => "b4",
            RegWindow::B5 => "b5",
            RegWindow::B6 => "b6",
            RegWindow::B7 => "b7",
            RegWindow::B8 => "b8",
            RegWindow::Q1 => "q1",
            RegWindow::Q2 => "q2",
            RegWindow::Q3 => "q3",
            RegWindow::Q4 => "q4",
            RegWindow::L => "l",
            RegWindow::H => "h",
            RegWindow::F => "f",
        }
    }
    fn from_suffix(suffix: &str) -> Self {
        match suffix {
            "b1" => RegWindow::B1,
            "b2" => RegWindow::B2,
            "b3" => RegWindow::B3,
            "b4" => RegWindow::B4,
            "b5" => RegWindow::B5,
            "b6" => RegWindow::B6,
            "b7" => RegWindow::B7,
            "b8" => RegWindow::B8,
            "q1" => RegWindow::Q1,
            "q2" => RegWindow::Q2,
            "q3" => RegWindow::Q3,
            "q4" => RegWindow::Q4,
            "l" => RegWindow::L,
            "h" => RegWindow::H,
            "f" => RegWindow::F,
            _ => RegWindow::F,
        }
    }
}

/// decodes into register index and window
fn decode_register(code: RegHandle) -> (u8, RegWindow) {
    let base = code & 0x0f;
    let sub = (code & 0xf0) >> 4;

    match sub {
        0 => (base, RegWindow::F),
        1 => (base, RegWindow::B1),
        2 => (base, RegWindow::B2),
        3 => (base, RegWindow::B3),
        4 => (base, RegWindow::B4),
        5 => (base, RegWindow::B5),
        6 => (base, RegWindow::B6),
        7 => (base, RegWindow::B7),
        8 => (base, RegWindow::B8),
        9 => (base, RegWindow::Q1),
        10 => (base, RegWindow::Q2),
        11 => (base, RegWindow::Q3),
        12 => (base, RegWindow::Q4),
        13 => (base, RegWindow::L),
        14 => (base, RegWindow::H),
        15 => (base, RegWindow::F), // potential to reroute to other registers
        16 => (base, RegWindow::F), // potential to reroute to other registers
        _ => unreachable!(),
    }
}

union RegisterUnion {
    full: u64,
    half: [u32; 2],
    quarter: [u16; 4],
    byte: [u8; 8],
}

// reimplemented
struct Register {
    base_name: String,
    internal: RegisterUnion,
    code: RegHandle,
    immutable: bool,
}
impl Register {
    fn new(name: &str, code: RegHandle) -> Self {
        Self {
            base_name: name.to_string(),
            internal: RegisterUnion {
                full: UNINITIALIZED_REGISTER,
            },
            code,
            immutable: false,
        }
    }

    fn name(&self, window: RegWindow) -> String {
        if (self.code & 0x0f) <= 4 {
            self.base_name.clone()
        } else {
            let s = self.base_name.clone() + window.to_suffix();
            s
        }
    }

    fn write(&mut self, window: RegWindow, value: u64) {
        if !self.immutable {
            match window {
                RegWindow::B1 => unsafe { self.internal.byte[0] = value as u8 },
                RegWindow::B2 => unsafe { self.internal.byte[1] = value as u8 },
                RegWindow::B3 => unsafe { self.internal.byte[2] = value as u8 },
                RegWindow::B4 => unsafe { self.internal.byte[3] = value as u8 },
                RegWindow::B5 => unsafe { self.internal.byte[4] = value as u8 },
                RegWindow::B6 => unsafe { self.internal.byte[5] = value as u8 },
                RegWindow::B7 => unsafe { self.internal.byte[6] = value as u8 },
                RegWindow::B8 => unsafe { self.internal.byte[7] = value as u8 },
                RegWindow::Q1 => unsafe { self.internal.quarter[0] = value as u16 },
                RegWindow::Q2 => unsafe { self.internal.quarter[1] = value as u16 },
                RegWindow::Q3 => unsafe { self.internal.quarter[2] = value as u16 },
                RegWindow::Q4 => unsafe { self.internal.quarter[3] = value as u16 },
                RegWindow::L => unsafe { self.internal.half[0] = value as u32 },
                RegWindow::H => unsafe { self.internal.half[1] = value as u32 },
                RegWindow::F => self.internal.full = value,
            }
        }
    }
    fn read(&self, window: RegWindow) -> u64 {
        match window {
            RegWindow::B1 => unsafe { self.internal.byte[0] as u64 },
            RegWindow::B2 => unsafe { self.internal.byte[1] as u64 },
            RegWindow::B3 => unsafe { self.internal.byte[2] as u64 },
            RegWindow::B4 => unsafe { self.internal.byte[3] as u64 },
            RegWindow::B5 => unsafe { self.internal.byte[4] as u64 },
            RegWindow::B6 => unsafe { self.internal.byte[5] as u64 },
            RegWindow::B7 => unsafe { self.internal.byte[6] as u64 },
            RegWindow::B8 => unsafe { self.internal.byte[7] as u64 },
            RegWindow::Q1 => unsafe { self.internal.quarter[0] as u64 },
            RegWindow::Q2 => unsafe { self.internal.quarter[1] as u64 },
            RegWindow::Q3 => unsafe { self.internal.quarter[2] as u64 },
            RegWindow::Q4 => unsafe { self.internal.quarter[3] as u64 },
            RegWindow::L => unsafe { self.internal.half[0] as u64 },
            RegWindow::H => unsafe { self.internal.half[1] as u64 },
            RegWindow::F => unsafe { self.internal.full },
        }
    }
    // fn get_size(&self,window:RegWindow) -> usize {
    //     match window {

    //     }
    // }
    // fn as_window_mut(&mut self, window: RegWindow) -> &mut Self {
    //     self.window = window;
    //     self
    // }
    // fn as_window(&mut self, window: RegWindow) -> &Self {
    //     self.window = window;
    //     self
    // }
}

pub struct CPURegisters {
    registers: [Register; 16],
}
impl CPURegisters {
    fn new() -> Self {
        let mut registers = [
            Register::new("null", 0),
            Register::new("pc", 1),
            Register::new("sp", 2),
            Register::new("fp", 3),
            Register::new("r1", 4),
            Register::new("r2", 5),
            Register::new("r3", 6),
            Register::new("r4", 7),
            Register::new("r5", 8),
            Register::new("r6", 9),
            Register::new("r7", 10),
            Register::new("r8", 11),
            Register::new("r9", 12),
            Register::new("r10", 13),
            Register::new("r11", 14),
            Register::new("r12", 15),
        ];
        registers[0].write(RegWindow::F, 0);
        registers[0].immutable = true;
        Self { registers }
    }

    fn get_register(&self, idx: RegHandle) -> &Register {
        &self.registers[idx as usize]
    }

    fn get_mut_register(&mut self, idx: RegHandle) -> &mut Register {
        &mut self.registers[idx as usize]
    }
    pub fn read(&mut self, register_handle: RegHandle) -> u64 {
        let (idx, window) = decode_register(register_handle);
        self.get_register(idx).read(window)
    }
    pub fn write(&mut self, register_handle: RegHandle, value: u64) {
        let (idx, window) = decode_register(register_handle);
        self.get_mut_register(idx).write(window, value)
    }

    pub fn print(&mut self, register_handle: RegHandle) -> String {
        let (idx, window) = decode_register(register_handle);
        let name = self.get_register(idx).name(window);
        let val = self.read(register_handle);
        format!(
            "{}{}",
            name.red(),
            format!("(0x{:0>2x}|{})", val, val).dark_blue()
        )
    }
    pub fn print_float(&mut self, register_handle: RegHandle) -> String {
        let (idx, window) = decode_register(register_handle);
        let name = self.get_register(idx).name(window);
        let val: f64 = unsafe { transmute(self.read(register_handle)) };
        format!("{}{}", name.red(), format!("(0x{:0>2})", val).dark_blue())
    }

    pub fn get_bytelength(&self, register_handle: RegHandle) -> u64 {
        let (_, window) = decode_register(register_handle);
        match window {
            RegWindow::B1 => 1,
            RegWindow::B2 => 1,
            RegWindow::B3 => 1,
            RegWindow::B4 => 1,
            RegWindow::B5 => 1,
            RegWindow::B6 => 1,
            RegWindow::B7 => 1,
            RegWindow::B8 => 1,
            RegWindow::Q1 => 2,
            RegWindow::Q2 => 2,
            RegWindow::Q3 => 2,
            RegWindow::Q4 => 2,
            RegWindow::L => 4,
            RegWindow::H => 4,
            RegWindow::F => 8,
        }
    }
}

pub struct CPU {
    pub registers: CPURegisters,
    pub memory: Memory,
    // pub vm_host_bridge: VMHostBridge,
    pub pending_interrupt: u8,
}

impl CPU {
    pub fn new(heap: u64, stack: u64) -> Self {
        Self {
            registers: CPURegisters::new(),
            memory: Memory::new(heap, stack),
            // vm_host_bridge: VMHostBridge::new(),
            pending_interrupt: 0,
        }
    }
    pub fn load(&mut self, file_path: &str) -> Result<(), ExecutionError> {
        let mut file = File::open(file_path)
            .map_err(|e| ExecutionError::new(format!("cannot open file `{file_path}`: {e}",)))?;
        let mut contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(|e| ExecutionError::new(format!("cannot read file to memory: {e}")))?;
        let nisvc_executable_package = NISVCEF::load(contents)?;
        self.memory.load(nisvc_executable_package.image)?;
        self.registers
            .write(PROGRAM_COUNTER, nisvc_executable_package.entry_point);
        self.registers.write(STACK_POINTER, self.memory.stack_start);
        self.registers.write(FRAME_POINTER, self.memory.stack_start);
        Ok(())
    }
    /// advances pc and returns consumed byte
    fn consume_byte(&mut self) -> Result<u8, ExecutionError> {
        let pc = self.registers.read(PROGRAM_COUNTER);
        let byte = self.memory.read_byte(pc)?;
        self.registers.write(PROGRAM_COUNTER, pc + 1);
        very_verbose_println!("byte at {pc:#x} consumed: {:#x}", byte);

        Ok(byte)
    }
    /// advances pc and returns consumed address (double word u64)
    fn consume_constant(&mut self) -> Result<u64, ExecutionError> {
        let pc = self.registers.read(PROGRAM_COUNTER);
        let double_word = self.memory.read_address(pc)?;
        self.registers.write(PROGRAM_COUNTER, pc + 8);
        very_verbose_println!(
            "byte at {pc:#x}..{:#x} consumed: {:#x}",
            pc + 8,
            double_word
        );

        Ok(double_word)
    }

    fn fetch_decode(&mut self) -> Result<Operation, ExecutionError> {
        let opcode = self.consume_byte()?;
        let operation = match opcode {
            0x00 => Operation::Nop,
            0x01 => Operation::Cpy {
                dest: self.consume_byte()?,
                src: self.consume_byte()?,
            },
            0x02 => Operation::Ldi {
                dest: self.consume_byte()?,
                src: self.consume_constant()?,
            },
            0x03 => Operation::Load {
                dest: self.consume_byte()?,
                n: self.consume_byte()?,
                src: self.consume_byte()?,
            },
            0x04 => Operation::Store {
                dest: self.consume_byte()?,
                n: self.consume_byte()?,
                src: self.consume_byte()?,
            },
            0x05 => Operation::Add {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x06 => Operation::Sub {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x07 => Operation::Mult {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x08 => Operation::Div {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x09 => Operation::Or {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x0a => Operation::Xor {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x0b => Operation::And {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x0c => Operation::Not {
                dest: self.consume_byte()?,
                op: self.consume_byte()?,
            },
            0x0d => Operation::Shl {
                dest: self.consume_byte()?,
                n: self.consume_byte()?,
                src: self.consume_byte()?,
            },
            0x0e => Operation::Shr {
                dest: self.consume_byte()?,
                n: self.consume_byte()?,
                src: self.consume_byte()?,
            },
            0x0f => Operation::Rotl {
                dest: self.consume_byte()?,
                n: self.consume_byte()?,
                src: self.consume_byte()?,
            },
            0x10 => Operation::Rotr {
                dest: self.consume_byte()?,
                n: self.consume_byte()?,
                src: self.consume_byte()?,
            },
            0x11 => Operation::Neg {
                dest: self.consume_byte()?,
                op: self.consume_byte()?,
            },
            0x12 => Operation::Jmp {
                addr: self.consume_constant()?,
            },
            0x13 => Operation::Jifz {
                condition: self.consume_byte()?,
                addr: self.consume_constant()?,
            },
            0x14 => Operation::Jifnz {
                condition: self.consume_byte()?,
                addr: self.consume_constant()?,
            },
            // pr
            0x16 => Operation::Inc {
                reg: self.consume_byte()?,
            },
            0x17 => Operation::Dec {
                reg: self.consume_byte()?,
            },
            0x18 => Operation::Push {
                src: self.consume_byte()?,
            },
            0x19 => Operation::Pop {
                dest: self.consume_byte()?,
            },
            0x1a => Operation::Call {
                addr: self.consume_constant()?,
            },
            0x1b => Operation::Ret,
            0x1c => Operation::Itof {
                destf: self.consume_byte()?,
                srci: self.consume_byte()?,
            },
            0x1d => Operation::Ftoi {
                desti: self.consume_byte()?,
                srcf: self.consume_byte()?,
            },
            0x1e => Operation::Fadd {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x1f => Operation::Fsub {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x20 => Operation::Fmult {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x21 => Operation::Fdiv {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x22 => Operation::Fmod {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x23 => Operation::Mod {
                dest: self.consume_byte()?,
                op1: self.consume_byte()?,
                op2: self.consume_byte()?,
            },
            0x24 => Operation::Int {
                code: self.consume_constant()?,
            },
            0x25 => Operation::Pushi {
                immediate: self.consume_constant()?,
            },
            0xfd => {
                return Err(ExecutionError::new(format!(
                    "malformed binary: attempted to execute uninitialized memory (opcode 0xfd)"
                )))
            }
            0xff => todo!(),
            _ => panic!("unrecognized opcode {opcode:#x}"),
        };
        Ok(operation)
    }

    fn execute(&mut self, operation: Operation) -> Result<(), ExecutionError> {
        very_verbose_println!("exec {:?}", operation);
        match operation {
            Operation::Nop => log_disassembly!("nop"),
            Operation::Cpy { dest, src } => {
                let value = self.registers.read(src);
                self.registers.write(dest, value);
                // println!("you here? ");
                log_disassembly!(
                    "cpy {}, {}",
                    self.registers.print(dest),
                    self.registers.print(src)
                );
            }
            Operation::Ldi { dest, src } => {
                self.registers.write(dest, src);
                log_disassembly!("ldi {}, ${}", self.registers.print(dest), src);
            }

            Operation::Load { dest, n, src } => {
                let n_val = self.registers.read(n);
                log_disassembly!(
                    "load {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(n),
                    self.registers.print(src),
                );
                let bytes = bytes_to_u64(&self.memory.read(self.registers.read(src), n_val)?);
                self.registers.write(dest, bytes);
            }
            Operation::Store { dest, n, src } => {
                let bytes = self.registers.read(src).to_le_bytes();
                let n_val = self.registers.read(n);
                let addr = self.registers.read(dest);
                let max = self.registers.get_bytelength(src);
                if n_val > max {
                    let name = self.registers.print(src);
                    return Err(ExecutionError::new(format!("Attempted to store {n_val} bytes from {name} to ${addr:#x} which are more bytes than are present in the register ({max}) ")));
                }

                log_disassembly!(
                    "store {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(n),
                    self.registers.print(src)
                );
                self.memory.write(addr, &bytes[0..n_val as usize])?;
            }
            Operation::Add { dest, op1, op2 } => {
                let sum = self
                    .registers
                    .read(op1)
                    .wrapping_add(self.registers.read(op2));
                self.registers.write(dest, sum);

                log_disassembly!(
                    "add {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(op1),
                    self.registers.print(op2)
                );
            }
            Operation::Sub { dest, op1, op2 } => {
                let diff = self
                    .registers
                    .read(op1)
                    .wrapping_sub(self.registers.read(op2));
                self.registers.write(dest, diff);

                log_disassembly!(
                    "sub {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(op1),
                    self.registers.print(op2)
                );
            }
            Operation::Mult { dest, op1, op2 } => {
                let result = self
                    .registers
                    .read(op1)
                    .wrapping_mul(self.registers.read(op2));
                self.registers.write(dest, result);

                log_disassembly!(
                    "mult {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(op1),
                    self.registers.print(op2)
                );
            }
            Operation::Div { dest, op1, op2 } => {
                // let op1_val = self.registers.read(op1);
                let op2_val = self.registers.read(op2);
                if op2_val == 0 {
                    return Err(ExecutionError::new(format!(
                        "division by zero error {op1} / {op2}",
                    )));
                }
                let quotient = self.registers.read(op1).wrapping_div(op2_val);
                self.registers.write(dest, quotient);

                log_disassembly!(
                    "div {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(op1),
                    self.registers.print(op2)
                );
            }
            Operation::Or { dest, op1, op2 } => {
                let result = self.registers.read(op1) | self.registers.read(op2);
                self.registers.write(dest, result);

                log_disassembly!(
                    "or {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(op1),
                    self.registers.print(op2)
                );
            }
            Operation::Xor { dest, op1, op2 } => {
                let result = self.registers.read(op1) ^ self.registers.read(op2);
                self.registers.write(dest, result);

                log_disassembly!(
                    "xor {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(op1),
                    self.registers.print(op2)
                );
            }
            Operation::And { dest, op1, op2 } => {
                let result = self.registers.read(op1) & self.registers.read(op2);
                self.registers.write(dest, result);

                log_disassembly!(
                    "and {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(op1),
                    self.registers.print(op2)
                );
            }
            Operation::Not { dest, op } => {
                let result = !self.registers.read(op);
                self.registers.write(dest, result);

                log_disassembly!(
                    "not {}, {}",
                    self.registers.print(dest),
                    self.registers.print(op),
                );
            }
            Operation::Shl { dest, n, src } => {
                let result = self.registers.read(src).shl(self.registers.read(n));
                self.registers.write(dest, result);

                log_disassembly!(
                    "shl {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(n),
                    self.registers.print(src)
                );
            }
            Operation::Shr { dest, n, src } => {
                let result = self.registers.read(src).shr(self.registers.read(n));
                self.registers.write(dest, result);

                log_disassembly!(
                    "shr {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(n),
                    self.registers.print(src)
                );
            }
            Operation::Rotl { dest, n, src } => {
                let result = self
                    .registers
                    .read(src)
                    .rotate_left(self.registers.read(n) as u32);
                self.registers.write(dest, result);
                log_disassembly!(
                    "or {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(n),
                    self.registers.print(src)
                );
            }
            Operation::Rotr { dest, n, src } => {
                let result = self
                    .registers
                    .read(src)
                    .rotate_right(self.registers.read(n) as u32);
                self.registers.write(dest, result);
                log_disassembly!(
                    "or {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(n),
                    self.registers.print(src)
                );
            }
            Operation::Neg { dest, op } => {
                let sign_mask: u64 = 0x80_00_00_00_00_00_00_00;
                let result = self.registers.read(op) ^ sign_mask;
                self.registers.write(dest, result);
                log_disassembly!(
                    "or {}, {}",
                    self.registers.print(dest),
                    self.registers.print(op),
                );
            }
            Operation::Jmp { addr } => {
                self.registers.write(PROGRAM_COUNTER, addr);
                log_disassembly!("jmp ${addr}");
            }
            Operation::Jifz { addr, condition } => {
                if condition == 0 {
                    self.registers.write(PROGRAM_COUNTER, addr);
                }
                log_disassembly!("jifz {condition} ${addr}");
            }
            Operation::Jifnz { addr, condition } => {
                if condition != 0 {
                    self.registers.write(PROGRAM_COUNTER, addr);
                }
                log_disassembly!("jifnz {condition} ${addr}");
            }
            Operation::Inc { reg } => {
                let inc = self.registers.read(reg).wrapping_add(1);
                self.registers.write(reg, inc);
                log_disassembly!("inc {}", self.registers.print(reg));
            }
            Operation::Dec { reg } => {
                let inc = self.registers.read(reg).wrapping_sub(1);
                self.registers.write(reg, inc);
                log_disassembly!("dec {}", self.registers.print(reg));
            }

            Operation::Push { src } => {
                let value = self.registers.read(src);

                self.push(value)?;
                log_disassembly!("push {}", self.registers.print(src));
            }
            Operation::Pop { dest } => {
                let value = self.pop()?;
                self.registers.write(dest, value);
                log_disassembly!("pop {}", self.registers.print(dest));
            }

            Operation::Call { addr } => {
                log_disassembly!("call ${}", addr);
                let fp = self.registers.read(FRAME_POINTER);
                let ra = self.registers.read(PROGRAM_COUNTER);
                let sp = self.registers.read(STACK_POINTER);
                self.registers.write(FRAME_POINTER, sp);
                very_very_verbose_println!(
                    "-- frame setup -- {})",
                    self.registers.print(STACK_POINTER)
                );
                self.push(fp)?;
                very_very_verbose_println!(
                    "| fp {fp:#x} -- {} |",
                    self.registers.print(STACK_POINTER)
                );
                self.push(ra)?;
                very_very_verbose_println!(
                    "| ra {ra:#x} -- {} |",
                    self.registers.print(STACK_POINTER)
                );
                self.registers.write(PROGRAM_COUNTER, addr);
            }
            Operation::Ret => {
                log_disassembly!("ret");
                let ra = self.pop()?;
                let fp = self.pop()?;
                self.registers.write(FRAME_POINTER, fp);
                self.registers.write(PROGRAM_COUNTER, ra);
            }

            Operation::Itof { destf, srci } => {
                let f = self.registers.read(srci) as f64;

                self.registers.write(destf, unsafe { transmute(f) });

                log_disassembly!(
                    "itof {}, {}",
                    self.registers.print_float(destf),
                    self.registers.print(srci)
                )
            }
            Operation::Ftoi { desti, srcf } => {
                let i = self.registers.read(srcf) as u64;

                self.registers.write(desti, i);

                log_disassembly!(
                    "itof {}, {}",
                    self.registers.print(desti),
                    self.registers.print_float(srcf)
                )
            }
            Operation::Fadd { dest, op1, op2 } => {
                let fop1: f64 = unsafe { transmute(self.registers.read(op1)) };
                let fop2: f64 = unsafe { transmute(self.registers.read(op2)) };
                let sum = fop1 + fop2;
                self.registers.write(dest, unsafe { transmute(sum) });

                log_disassembly!(
                    "fadd {}, {}, {}",
                    self.registers.print_float(dest),
                    self.registers.print_float(op1),
                    self.registers.print_float(op2)
                );
            }
            Operation::Fsub { dest, op1, op2 } => {
                let fop1: f64 = unsafe { transmute(self.registers.read(op1)) };
                let fop2: f64 = unsafe { transmute(self.registers.read(op2)) };
                let diff = fop1 - fop2;
                self.registers.write(dest, unsafe { transmute(diff) });

                log_disassembly!(
                    "fsub {}, {}, {}",
                    self.registers.print_float(dest),
                    self.registers.print_float(op1),
                    self.registers.print_float(op2)
                );
            }
            Operation::Fmult { dest, op1, op2 } => {
                let fop1: f64 = unsafe { transmute(self.registers.read(op1)) };
                let fop2: f64 = unsafe { transmute(self.registers.read(op2)) };
                let result = fop1 * fop2;
                self.registers.write(dest, unsafe { transmute(result) });

                log_disassembly!(
                    "fmult {}, {}, {}",
                    self.registers.print_float(dest),
                    self.registers.print_float(op1),
                    self.registers.print_float(op2)
                );
            }
            Operation::Fdiv { dest, op1, op2 } => {
                let fop1: f64 = unsafe { transmute(self.registers.read(op1)) };
                let fop2: f64 = unsafe { transmute(self.registers.read(op2)) };
                let sum = fop1 + fop2;
                self.registers.write(dest, unsafe { transmute(sum) });

                log_disassembly!(
                    "fdiv {}, {}, {}",
                    self.registers.print_float(dest),
                    self.registers.print_float(op1),
                    self.registers.print_float(op2)
                );
            }
            Operation::Fmod { dest, op1, op2 } => {
                let fop1: f64 = unsafe { transmute(self.registers.read(op1)) };
                let fop2: f64 = unsafe { transmute(self.registers.read(op2)) };
                let sum = fop1 % fop2;
                self.registers.write(dest, unsafe { transmute(sum) });

                log_disassembly!(
                    "fdiv {}, {}, {}",
                    self.registers.print_float(dest),
                    self.registers.print_float(op1),
                    self.registers.print_float(op2)
                );
            }
            Operation::Mod { dest, op1, op2 } => {
                let sum = self.registers.read(op1) % self.registers.read(op2);
                self.registers.write(dest, sum);
                log_disassembly!(
                    "fdiv {}, {}, {}",
                    self.registers.print(dest),
                    self.registers.print(op1),
                    self.registers.print(op2)
                );
            }

            Operation::Breakpoint => todo!(),
            Operation::HaltExe => todo!("haltexe"),

            Operation::Int { code } => {
                log_disassembly!("int {:#x}", code);
                self.pending_interrupt = code as u8
            }
            Operation::Pushi { immediate } => {
                log_disassembly!("pushi {immediate:#x}");
                self.push(immediate)?
            }
        };
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), ExecutionError> {
        let op = self.fetch_decode()?;
        self.execute(op)?;
        unsafe { GLOBAL_PROGRAM_COUNTER = self.registers.read(PROGRAM_COUNTER) }
        Ok(())
    }

    fn break_point(&mut self) -> Result<(), ExecutionError> {
        Ok(())
    }

    pub fn push(&mut self, value: u64) -> Result<(), ExecutionError> {
        let sp = self.registers.read(STACK_POINTER);
        let sp_d = self.memory.push(sp, value)?;
        self.registers.write(STACK_POINTER, sp_d);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<u64, ExecutionError> {
        let sp = self.registers.read(STACK_POINTER);
        let (sp_d, value) = self.memory.pop(sp)?;
        self.registers.write(STACK_POINTER, sp_d);
        Ok(value)
    }
    pub fn dump_stack(&mut self) -> Vec<u64> {
        let mut stack_dump = Vec::<u64>::new();
        while self.registers.read(STACK_POINTER) != self.memory.stack_start {
            let popped = match self.pop() {
                Ok(p) => p,
                Err(_) => break,
            };
            stack_dump.push(popped);
        }
        stack_dump
    }
}
