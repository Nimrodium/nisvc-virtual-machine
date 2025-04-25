use core::fmt;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, Stderr, Stdin, Stdout, Write},
    process::exit,
};

use crate::{
    constant::{PROGRAM_COUNTER, STACK_POINTER, UNINITIALIZED_REGISTER},
    log_input, log_output,
    memory::Memory,
    opcode::Operation,
    verbose_println, very_verbose_println, very_very_verbose_println, ExecutionError,
};

// pub enum Window {
//     Byte1,
//     Byte2,
//     Byte3,
//     Byte4,
//     Byte5,
//     Byte6,
//     Byte7,
//     Byte8,

//     Quarter1,
//     Quarter2,
//     Quarter3,
//     Quarter4,
//     Low,
//     High,
//     Full,
// }

#[derive(Clone)]
pub struct Register {
    pub value: u64,
    pub base_name: String,
    pub code: u8,
    pub locked: bool,
    window: SubRegisterWindow,
}
#[derive(Clone)]
enum SubRegisterWindow {
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
impl SubRegisterWindow {
    fn to_suffix(&self) -> &str {
        match self {
            SubRegisterWindow::B1 => "b1",
            SubRegisterWindow::B2 => "b2",
            SubRegisterWindow::B3 => "b3",
            SubRegisterWindow::B4 => "b4",
            SubRegisterWindow::B5 => "b5",
            SubRegisterWindow::B6 => "b6",
            SubRegisterWindow::B7 => "b7",
            SubRegisterWindow::B8 => "b8",
            SubRegisterWindow::Q1 => "q1",
            SubRegisterWindow::Q2 => "q2",
            SubRegisterWindow::Q3 => "q3",
            SubRegisterWindow::Q4 => "q4",
            SubRegisterWindow::L => "l",
            SubRegisterWindow::H => "h",
            SubRegisterWindow::F => "f",
        }
    }
    fn from_suffix(suffix: &str) -> Self {
        match suffix {
            "b1" => SubRegisterWindow::B1,
            "b2" => SubRegisterWindow::B2,
            "b3" => SubRegisterWindow::B3,
            "b4" => SubRegisterWindow::B4,
            "b5" => SubRegisterWindow::B5,
            "b6" => SubRegisterWindow::B6,
            "b7" => SubRegisterWindow::B7,
            "b8" => SubRegisterWindow::B8,
            "q1" => SubRegisterWindow::Q1,
            "q2" => SubRegisterWindow::Q2,
            "q3" => SubRegisterWindow::Q3,
            "q4" => SubRegisterWindow::Q4,
            "l" => SubRegisterWindow::L,
            "h" => SubRegisterWindow::H,
            "f" => SubRegisterWindow::F,
            _ => SubRegisterWindow::F,
        }
    }
}
impl Register {
    fn new(name: &str, code: u8) -> Self {
        Self {
            value: UNINITIALIZED_REGISTER,
            base_name: name.to_string(),
            locked: false,
            window: SubRegisterWindow::F,
            code,
        }
    }
    pub fn name(&self) -> String {
        if self.code < 4 {
            self.base_name.clone()
        } else {
            self.base_name.clone() + self.window.to_suffix()
        }
    }
    pub fn extract(&self) -> (String, u64) {
        (self.name(), self.read())
    }

    pub fn write_at_byte(&mut self, value: u64, i: u8) {
        if i > 8 || i <= 0 {
            panic!("attempted to read at an invalid byte index {i} > 8")
        }
        let i = i - 1;
        let byte_mask = 0x00_00_00_00_00_00_00_FF;
        let clean_value = value & byte_mask;
        let byte_offset = i * 8;
        let byte_to_be_inserted = clean_value << byte_offset;
        let inverse_clear_dest_mask = !byte_mask.rotate_left(byte_offset as u32);
        let masked_reg = self.value & inverse_clear_dest_mask;
        self.value = masked_reg | byte_to_be_inserted;
    }

    pub fn write_at_quarter(&mut self, value: u64, i: u8) {
        if i > 4 || i <= 0 {
            panic!("attempted to read at an invalid quarter index {i} > 4")
        }
        let i = i - 1;

        let byte_offset = i * 16;

        let byte_mask = 0x00_00_00_00_00_00_FF_FF;
        let clean_value = value & byte_mask;

        let quarter_to_be_inserted = clean_value << byte_offset;

        let inverse_clear_dest_mask = !byte_mask.rotate_left(byte_offset as u32);
        let masked_reg = (self.value & inverse_clear_dest_mask);

        self.value = masked_reg | quarter_to_be_inserted;
    }

    pub fn write_at_half(&mut self, value: u64, i: u8) {
        if i > 2 || i <= 0 {
            panic!("attempted to read at an invalid half index {i} > 2")
        }
        let i = i - 1;
        let byte_offset = i * 32;
        let byte_mask = 0x00_00_00_00_FF_FF_FF_FF;
        let clean_value = value & byte_mask;
        let half_to_be_inserted = clean_value << byte_offset;
        let inverse_clear_dest_mask = !byte_mask.rotate_left(byte_offset as u32);
        self.value = (self.value & inverse_clear_dest_mask) | half_to_be_inserted;
    }

    pub fn read_at_byte(&self, i: u8) -> u64 {
        if i > 8 || i <= 0 {
            panic!("attempted to read at an invalid byte index {i} > 7")
        }
        let i = i - 1; // turn to real index
        let byte_mask = 0x00_00_00_00_00_00_00_FF << (i * 8);
        let masked_value = self.value & byte_mask;
        let shifted_value = masked_value >> (i * 8);
        shifted_value
    }
    pub fn read_at_quarter(&self, i: u8) -> u64 {
        if i > 4 || i <= 0 {
            panic!("attempted to read at an invalid byte index {i} > 3")
        }
        let i = i - 1;
        let byte_mask = 0x00_00_00_00_00_00_FF_FF << (i * 16);
        let masked_value = self.value & byte_mask;
        let shifted_value = masked_value >> (i * 16);
        shifted_value
    }
    pub fn read_at_half(&self, i: u8) -> u64 {
        if i > 2 || i <= 0 {
            panic!("attempted to read at an invalid byte index {i} > 1")
        }
        let i = i - 1;

        let byte_mask = 0x00_00_00_00_FF_FF_FF_FF << (i * 32);
        let masked_value = self.value & byte_mask;
        let shifted_value = masked_value >> (i * 32);
        shifted_value
    }
    pub fn write(&mut self, value: u64) {
        log_input!("{} <- {}", self.name(), value);
        if !self.locked {
            match self.window {
                SubRegisterWindow::B1 => self.write_at_byte(value, 1),
                SubRegisterWindow::B2 => self.write_at_byte(value, 2),
                SubRegisterWindow::B3 => self.write_at_byte(value, 3),
                SubRegisterWindow::B4 => self.write_at_byte(value, 4),
                SubRegisterWindow::B5 => self.write_at_byte(value, 5),
                SubRegisterWindow::B6 => self.write_at_byte(value, 6),
                SubRegisterWindow::B7 => self.write_at_byte(value, 7),
                SubRegisterWindow::B8 => self.write_at_byte(value, 8),
                SubRegisterWindow::Q1 => self.write_at_quarter(value, 1),
                SubRegisterWindow::Q2 => self.write_at_quarter(value, 2),
                SubRegisterWindow::Q3 => self.write_at_quarter(value, 3),
                SubRegisterWindow::Q4 => self.write_at_quarter(value, 4),
                SubRegisterWindow::L => self.write_at_half(value, 1),
                SubRegisterWindow::H => self.write_at_half(value, 2),
                SubRegisterWindow::F => self.value = value,
            };
            // self.value = value as RegisterWidth;
        } else {
            very_verbose_println!("attempted to write to locked register {}", self.name())
        }
    }
    pub fn read(&self) -> u64 {
        let value = match self.window {
            SubRegisterWindow::B1 => self.read_at_byte(1),
            SubRegisterWindow::B2 => self.read_at_byte(2),
            SubRegisterWindow::B3 => self.read_at_byte(3),
            SubRegisterWindow::B4 => self.read_at_byte(4),
            SubRegisterWindow::B5 => self.read_at_byte(5),
            SubRegisterWindow::B6 => self.read_at_byte(6),
            SubRegisterWindow::B7 => self.read_at_byte(7),
            SubRegisterWindow::B8 => self.read_at_byte(8),
            SubRegisterWindow::Q1 => self.read_at_quarter(1),
            SubRegisterWindow::Q2 => self.read_at_quarter(2),
            SubRegisterWindow::Q3 => self.read_at_quarter(3),
            SubRegisterWindow::Q4 => self.read_at_quarter(4),
            SubRegisterWindow::L => self.read_at_half(1),
            SubRegisterWindow::H => self.read_at_half(2),
            SubRegisterWindow::F => self.value,
        };
        log_output!("{} -> {}", self.name(), value);
        value
    }
    fn as_window_mut(&mut self, window: SubRegisterWindow) -> &mut Self {
        self.window = window;
        self
    }
    fn as_window(&mut self, window: SubRegisterWindow) -> &Self {
        self.window = window;
        self
    }
}
impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.read();
        write!(
            f,
            "[ {} ({:#})|({:#x})|({:#b}) ]",
            self.name(),
            value,
            value,
            value
        )
    }
}

pub struct CPURegisters {
    registers: Vec<Register>,
}
impl fmt::Display for CPURegisters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();
        for r in &self.registers {
            string.push_str(&(r.to_string() + r.code.to_string().as_str()));
            string.push('\n')
        }
        write!(f, "{string}")
    }
}

impl CPURegisters {
    fn new() -> Self {
        verbose_println!("initializing registers...");
        let mut registers: Vec<Register> = vec![
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
        registers[0].write(0);
        registers[0].locked = true;
        Self { registers }
    }
    pub fn get_register(&mut self, code: u8) -> Result<&Register, ExecutionError> {
        let base = code & 0x0F; // mask out subregister field
        let sub = (code & 0xF0) >> 4; // mask out register field
        very_very_verbose_println!("getting register {base:#x} sub {sub:#x}");
        if base as usize > self.registers.len() {
            panic!("invalid register code");
        }

        let register: &mut Register = if let Some(r) = self.registers.get_mut(base as usize) {
            r
        } else {
            panic!("register does not exist");
        };
        // not subdivided

        let window: SubRegisterWindow = if base <= 3 {
            SubRegisterWindow::F
        } else {
            match sub {
                0 => SubRegisterWindow::F,
                1 => SubRegisterWindow::B1,
                2 => SubRegisterWindow::B2,
                3 => SubRegisterWindow::B3,
                4 => SubRegisterWindow::B4,
                5 => SubRegisterWindow::B5,
                6 => SubRegisterWindow::B6,
                7 => SubRegisterWindow::B7,
                8 => SubRegisterWindow::B8,
                9 => SubRegisterWindow::Q1,
                10 => SubRegisterWindow::Q2,
                11 => SubRegisterWindow::Q3,
                12 => SubRegisterWindow::Q4,
                13 => SubRegisterWindow::L,
                14 => SubRegisterWindow::H,
                15 => SubRegisterWindow::F,
                16 => SubRegisterWindow::F,
                _ => panic!("invalid code"),
            }
        };
        let windowed_register = register.as_window(window);
        very_very_verbose_println!("passing register as {}", windowed_register.name());
        Ok(windowed_register)
    }
    pub fn get_mut_register(&mut self, code: u8) -> Result<&mut Register, ExecutionError> {
        let base = code & 0x0F; // mask out subregister field
        let sub = (code & 0xF0) >> 4; // mask out register field
        very_verbose_println!("getting register {base:#x} sub {sub:#x}");
        if base as usize > self.registers.len() {
            panic!("invalid register code");
        }

        let register: &mut Register = if let Some(r) = self.registers.get_mut(base as usize) {
            r
        } else {
            panic!("register does not exist");
        };
        // not subdivided

        let window: SubRegisterWindow = if base <= 3 {
            SubRegisterWindow::F
        } else {
            match sub {
                0 => SubRegisterWindow::F,
                1 => SubRegisterWindow::B1,
                2 => SubRegisterWindow::B2,
                3 => SubRegisterWindow::B3,
                4 => SubRegisterWindow::B4,
                5 => SubRegisterWindow::B5,
                6 => SubRegisterWindow::B6,
                7 => SubRegisterWindow::B7,
                8 => SubRegisterWindow::B8,
                9 => SubRegisterWindow::Q1,
                10 => SubRegisterWindow::Q2,
                11 => SubRegisterWindow::Q3,
                12 => SubRegisterWindow::Q4,
                13 => SubRegisterWindow::L,
                14 => SubRegisterWindow::H,
                15 => SubRegisterWindow::F,
                16 => SubRegisterWindow::F,
                _ => panic!("invalid code"),
            }
        };
        let windowed_register = register.as_window_mut(window);
        very_very_verbose_println!("passing register as {}", windowed_register.name());
        Ok(windowed_register)
    }
    pub fn get_register_via_reverse_lookup(
        &mut self,
        register_name: &str,
    ) -> Result<&mut Register, ExecutionError> {
        let valid_sub_names = [
            "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "q1", "q2", "q3", "q4", "l", "h", "f",
        ];
        let (base_name, window) = match register_name {
            "null" | "pc" | "sp" | "fp" => (register_name, "f"),

            _ => (&register_name[..2], {
                let sub = &register_name[2..];
                if sub.is_empty() {
                    "f"
                } else {
                    if valid_sub_names.contains(&sub) {
                        sub
                    } else {
                        return Err(ExecutionError::new(format!(
                            "{sub} is not a valid subregister"
                        )));
                    }
                }
            }),
        };
        println!("{base_name}|{window}");
        let mut reg: Option<&mut Register> = None;
        for r in &mut self.registers {
            if r.base_name.as_str() == base_name {
                reg = Some(r);
            }
        }
        if let Some(r) = reg {
            let window = SubRegisterWindow::from_suffix(window);
            Ok(r.as_window_mut(window))
        } else {
            Err(ExecutionError::new(format!(
                "{register_name} is not a valid register"
            )))
        }
    }
}

union RegisterUnion {
    full: u64,
    half: [u32; 2],
    quarter: [u16; 4],
    byte: [u8; 8],
}

// reimplemented
struct _Register {
    base_name: String,
    internal: RegisterUnion,
    window: SubRegisterWindow,
    immutable: bool,
}
impl _Register {
    fn new(name: &str) -> Self {
        Self {
            base_name: name.to_string(),
            internal: RegisterUnion {
                full: UNINITIALIZED_REGISTER,
            },
            window: SubRegisterWindow::F,
            immutable: false,
        }
    }

    pub fn write(&mut self, value: u64) {
        if !self.immutable {
            match self.window {
                SubRegisterWindow::B1 => unsafe { self.internal.byte[0] = value as u8 },
                SubRegisterWindow::B2 => unsafe { self.internal.byte[1] = value as u8 },
                SubRegisterWindow::B3 => unsafe { self.internal.byte[2] = value as u8 },
                SubRegisterWindow::B4 => unsafe { self.internal.byte[3] = value as u8 },
                SubRegisterWindow::B5 => unsafe { self.internal.byte[4] = value as u8 },
                SubRegisterWindow::B6 => unsafe { self.internal.byte[5] = value as u8 },
                SubRegisterWindow::B7 => unsafe { self.internal.byte[6] = value as u8 },
                SubRegisterWindow::B8 => unsafe { self.internal.byte[7] = value as u8 },
                SubRegisterWindow::Q1 => unsafe { self.internal.quarter[0] = value as u16 },
                SubRegisterWindow::Q2 => unsafe { self.internal.quarter[1] = value as u16 },
                SubRegisterWindow::Q3 => unsafe { self.internal.quarter[2] = value as u16 },
                SubRegisterWindow::Q4 => unsafe { self.internal.quarter[3] = value as u16 },
                SubRegisterWindow::L => unsafe { self.internal.half[0] = value as u32 },
                SubRegisterWindow::H => unsafe { self.internal.half[1] = value as u32 },
                SubRegisterWindow::F => self.internal.full = value,
            }
        }
    }
    pub fn read(&self) -> u64 {
        match self.window {
            SubRegisterWindow::B1 => unsafe { self.internal.byte[0] as u64 },
            SubRegisterWindow::B2 => unsafe { self.internal.byte[1] as u64 },
            SubRegisterWindow::B3 => unsafe { self.internal.byte[2] as u64 },
            SubRegisterWindow::B4 => unsafe { self.internal.byte[3] as u64 },
            SubRegisterWindow::B5 => unsafe { self.internal.byte[4] as u64 },
            SubRegisterWindow::B6 => unsafe { self.internal.byte[5] as u64 },
            SubRegisterWindow::B7 => unsafe { self.internal.byte[6] as u64 },
            SubRegisterWindow::B8 => unsafe { self.internal.byte[7] as u64 },
            SubRegisterWindow::Q1 => unsafe { self.internal.quarter[0] as u64 },
            SubRegisterWindow::Q2 => unsafe { self.internal.quarter[1] as u64 },
            SubRegisterWindow::Q3 => unsafe { self.internal.quarter[2] as u64 },
            SubRegisterWindow::Q4 => unsafe { self.internal.quarter[3] as u64 },
            SubRegisterWindow::L => unsafe { self.internal.half[0] as u64 },
            SubRegisterWindow::H => unsafe { self.internal.half[1] as u64 },
            SubRegisterWindow::F => unsafe { self.internal.full },
        }
    }
    fn as_window_mut(&mut self, window: SubRegisterWindow) -> &mut Self {
        self.window = window;
        self
    }
    fn as_window(&mut self, window: SubRegisterWindow) -> &Self {
        self.window = window;
        self
    }
}

pub struct _CPURegisters {
    registers: [_Register; 16],
}
impl _CPURegisters {
    fn new() -> Self {
        let mut registers = [
            _Register::new("null"),
            _Register::new("pc"),
            _Register::new("sp"),
            _Register::new("fp"),
            _Register::new("r1"),
            _Register::new("r2"),
            _Register::new("r3"),
            _Register::new("r4"),
            _Register::new("r5"),
            _Register::new("r6"),
            _Register::new("r7"),
            _Register::new("r8"),
            _Register::new("r9"),
            _Register::new("r10"),
            _Register::new("r11"),
            _Register::new("r12"),
        ];
        registers[0].write(0);
        registers[0].immutable = true;
        Self { registers }
    }
    pub fn get_register(&mut self, code: u8) -> &_Register {
        let base = code & 0x0f;
        let sub = (code & 0xf0) >> 4;
        let register = &mut self.registers[base as usize];
        let window = match sub {
            0 => SubRegisterWindow::F,
            1 => SubRegisterWindow::B1,
            2 => SubRegisterWindow::B2,
            3 => SubRegisterWindow::B3,
            4 => SubRegisterWindow::B4,
            5 => SubRegisterWindow::B5,
            6 => SubRegisterWindow::B6,
            7 => SubRegisterWindow::B7,
            8 => SubRegisterWindow::B8,
            9 => SubRegisterWindow::Q1,
            10 => SubRegisterWindow::Q2,
            11 => SubRegisterWindow::Q3,
            12 => SubRegisterWindow::Q4,
            13 => SubRegisterWindow::L,
            14 => SubRegisterWindow::H,
            15 => SubRegisterWindow::F, // potential to reroute to other register
            16 => SubRegisterWindow::F, // potential to reroute to other register
            _ => unreachable!(),
        };
        register.as_window(window)
    }
}

pub struct CPU {
    pub registers: CPURegisters,
    pub memory: Memory,
    pub vm_host_bridge: VMHostBridge,
}

impl CPU {
    fn new(mem_b: u64) -> Self {
        Self {
            registers: CPURegisters::new(),
            memory: Memory::new(mem_b),
            vm_host_bridge: VMHostBridge::new(),
        }
    }
    fn fetch(&mut self) -> Result<Operation, ExecutionError> {
        let pc = self.registers.get_mut_register(PROGRAM_COUNTER)?;
        let byte = self.memory.read_byte(pc.read())?;
        pc.write(pc.read() + 1);
        if let Some(opcode) = Operation::decode(byte) {
            Ok(opcode)
        } else {
            Err(ExecutionError::new(format!(
                "unrecognized operation {byte:#x}"
            )))
        }
    }
    fn decode(&mut self, operation: Operation) -> Result<DecodedInstruction, ExecutionError> {
        let mut immutable_registers: Vec<Register> = Vec::new();
        let mut mutable_registers: Vec<u8> = Vec::new();
        let mut addresses: Vec<u64> = Vec::new();
        let mut immediates: Vec<u64> = Vec::new();
        let mut pc = self.registers.get_register(PROGRAM_COUNTER)?.read();

        for kind in operation.get_operand_map() {
            match kind {
                Kind::Register => immutable_registers.push({
                    let code = self.memory.read_byte(pc)?;
                    pc += 1;
                    self.registers.get_register(code)?.clone()
                }),
                Kind::MutableRegister => mutable_registers.push({
                    let code = self.memory.read_byte(pc)?;
                    pc += 1;
                    code
                }),
                Kind::Immediate => immediates.push({
                    let (v, mv_pc) = self.memory.read_immediate(pc)?;
                    pc += mv_pc;
                    v
                }),
                Kind::Address => addresses.push({
                    let address = self.memory.read_address(pc)?;
                    pc += size_of::<u64>() as u64;
                    address
                }),
            }
        }
        self.registers.get_mut_register(PROGRAM_COUNTER)?.write(pc);
        Ok(DecodedInstruction {
            immutable_registers,
            mutable_registers,
            addresses,
            immediates,
        })
    }

    pub fn decode_trinary_register_operation(
        decoded: &DecodedInstruction,
    ) -> (u8, &Register, &Register) {
        (
            decoded.mutable_registers[0],
            &decoded.immutable_registers[0],
            &decoded.immutable_registers[1],
        )
    }

    pub fn decode_binay_register_operation(mut decoded: DecodedInstruction) -> (u8, Register) {
        (
            decoded.mutable_registers[0],
            decoded.immutable_registers.remove(0),
        )
    }

    fn execute(
        &mut self,
        operation: Operation,
        decoded: DecodedInstruction,
    ) -> Result<(), ExecutionError> {
        match operation {
            Operation::Nop => self.op_nop(decoded),
            Operation::Cpy => self.op_cpy(decoded),
            Operation::Ldi => self.op_ldi(decoded),
            Operation::Load => self.op_load(decoded),
            Operation::Store => self.op_store(decoded),
            Operation::Add => self.op_add(decoded),
            Operation::Sub => self.op_sub(decoded),
            Operation::Mult => self.op_mult(decoded),
            Operation::Div => self.op_div(decoded),
            Operation::Or => self.op_or(decoded),
            Operation::Xor => self.op_xor(decoded),
            Operation::And => self.op_and(decoded),
            Operation::Not => self.op_not(decoded),
            Operation::Shl => self.op_shl(decoded),
            Operation::Shr => self.op_shr(decoded),
            Operation::Rotl => self.op_rotl(decoded),
            Operation::Rotr => self.op_rotr(decoded),
            Operation::Neg => self.op_neg(decoded),
            Operation::Jmp => self.op_jmp(decoded),
            Operation::Jifz => self.op_jifz(decoded),
            Operation::Jifnz => self.op_jifnz(decoded),
            Operation::Inc => self.op_inc(decoded),
            Operation::Dec => self.op_dec(decoded),
            Operation::Push => self.op_push(decoded),
            Operation::Pop => self.op_pop(decoded),
            Operation::Call => self.op_call(decoded),
            Operation::Ret => self.op_ret(decoded),
            Operation::Fopen => self.op_fopen(decoded),
            Operation::Fread => self.op_fread(decoded),
            Operation::Fwrite => self.op_fwrite(decoded),
            Operation::Fseek => self.op_fseek(decoded),
            Operation::Fclose => self.op_fclose(decoded),
            Operation::Malloc => self.op_malloc(decoded),
            Operation::Realloc => self.op_realloc(decoded),
            Operation::Free => self.op_free(decoded),
            Operation::Memcpy => self.op_memcpy(decoded),
            Operation::Memset => self.op_memset(decoded),
            Operation::Itof => self.op_itof(decoded),
            Operation::Ftoi => self.op_ftoi(decoded),
            Operation::Fadd => self.op_fadd(decoded),
            Operation::Fsub => self.op_fsub(decoded),
            Operation::Fmult => self.op_fmult(decoded),
            Operation::Fdiv => self.op_fdiv(decoded),
            Operation::Breakpoint => self.break_point(),
            Operation::HaltExe => unreachable!(),
        }?;
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), ExecutionError> {
        todo!()
    }

    pub fn exec_loop(&mut self) -> Result<(), ExecutionError> {
        todo!()
    }
    fn break_point(&mut self) -> Result<(), ExecutionError> {
        Ok(())
    }

    pub fn push(&mut self, value: u64) -> Result<(), ExecutionError> {
        let sp = self.registers.get_mut_register(STACK_POINTER)?;
        sp.write(self.memory.push(sp.read(), value)?);
        Ok(())
    }
    pub fn pop(&mut self) -> Result<u64, ExecutionError> {
        let sp = self.registers.get_mut_register(STACK_POINTER)?;
        let (new_sp, popped_value) = self.memory.pop(sp.read())?;
        sp.write(new_sp);
        Ok(popped_value)
    }
}

pub enum Kind {
    Register,
    MutableRegister,
    Immediate,
    Address,
}

pub struct DecodedInstruction {
    pub immutable_registers: Vec<Register>,
    pub mutable_registers: Vec<u8>,

    pub addresses: Vec<u64>,
    pub immediates: Vec<u64>,
}

type VMFD = usize;
pub struct VMHostBridge {
    stdin: Stdin,
    stdout: Stdout,
    stderr: Stderr,
    open_file_vector: HashMap<VMFD, (File, String)>,
    next_vmfd: usize,
}

// bridge isa
// fopen fd_store filep_ptr filep_len
// fwrite fd str_ptr str_len
// fread fd buf_ptr buf_len
// fclose fd

impl VMHostBridge {
    fn new() -> Self {
        // setup stdin & stdout
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let stderr = std::io::stderr();
        Self {
            stdin,
            stdout,
            stderr,
            open_file_vector: HashMap::new(),
            next_vmfd: 3,
        }
    }
    fn get_file_from_vmfd(&mut self, vmfd: VMFD) -> Result<&mut (File, String), ExecutionError> {
        let open_files = self.open_file_vector.len();
        match self.open_file_vector.get_mut(&vmfd) {
            Some(f) => Ok(f),
            None => Err(ExecutionError::new(format!(
                "vmfd-{vmfd} does not exist, there are {} open files",
                open_files
            ))),
        }
    }
    pub fn fopen(&mut self, file_path: &str) -> Result<VMFD, ExecutionError> {
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(why) => {
                return Err(ExecutionError::new(format!(
                    "failed to open host file {file_path} :: {why}"
                )))
            }
        };
        let vmfd = self.next_vmfd;
        self.next_vmfd += 1;

        self.open_file_vector
            .insert(vmfd, (file, file_path.to_string()));

        verbose_println!("opened file {file_path} as vmfd-{vmfd}");
        Ok(vmfd)
    }
    pub fn fclose(&mut self, vmfd: VMFD) -> Result<(), ExecutionError> {
        match vmfd {
            0 => return Err(ExecutionError::new(format!("cannot close stdin"))),
            1 => return Err(ExecutionError::new(format!("cannot cloes stdout"))),
            2 => return Err(ExecutionError::new(format!("cannot close stderr"))),
            _ => {
                // let (file, file_path) = self.get_file_from_vmfd(vmfd)?;
                let (file, file_path) = match self.open_file_vector.remove(&vmfd) {
                    Some(entry) => entry,
                    None => {
                        return Err(ExecutionError::new(format!(
                            "{vmfd} is not an open file handle"
                        )))
                    }
                };
                drop(file);
                verbose_println!("closed vmfd-{vmfd} :: {file_path}");
                Ok(())
            }
        }
    }
    pub fn fwrite(&mut self, vmfd: VMFD, buf: &[u8]) -> Result<(), ExecutionError> {
        match vmfd {
            0 => return Err(ExecutionError::new(format!("cannot write to stdin"))),
            1 => match self.stdout.write_all(buf) {
                Ok(_) => Ok(()),
                Err(e) => {
                    return Err(ExecutionError::new(format!(
                        "failed to write to stdout :: {e}"
                    )))
                }
            },
            2 => match self.stderr.write_all(buf) {
                Ok(_) => Ok(()),
                Err(e) => {
                    return Err(ExecutionError::new(format!(
                        "failed to write to stderr :: {e}"
                    )))
                }
            },
            _ => {
                let (file, file_path) = self.get_file_from_vmfd(vmfd)?;
                match file.write_all(buf) {
                    Ok(_) => Ok(()),

                    Err(e) => {
                        return Err(ExecutionError::new(format!(
                            "failed to write to {file_path} :: {e}"
                        )))
                    }
                }
            }
        }
    }

    pub fn fread(&mut self, vmfd: VMFD, length: usize) -> Result<Vec<u8>, ExecutionError> {
        let mut buf: Vec<u8> = vec![0u8; length];
        match vmfd {
            0 => match self.stdin.read(&mut buf) {
                Ok(bytes_read) => (),
                Err(e) => {
                    return Err(ExecutionError::new(format!(
                        "failed to read from stdin :: {e}"
                    )))
                }
            },
            1 => return Err(ExecutionError::new(format!("cannot read from stdout"))),
            2 => return Err(ExecutionError::new(format!("cannot read from stderr"))),
            _ => {
                let (file, file_path) = self.get_file_from_vmfd(vmfd)?;
                match file.read(&mut buf) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(ExecutionError::new(format!(
                            "failed to read from {file_path} :: {e}"
                        )))
                    }
                }
            }
        };
        Ok(buf)
    }

    pub fn fseek(
        &mut self,
        vmfd: VMFD,
        amount: usize,
        direction: u8,
    ) -> Result<(), ExecutionError> {
        let offset: i64 = if direction == 1 {
            amount as i64 * -1
        } else if direction == 0 {
            amount as i64
        } else {
            return Err(ExecutionError::new(format!("invalid seek direction")));
        };
        match vmfd {
            0 => return Err(ExecutionError::new(format!("cannot seek stdin"))),
            1 => return Err(ExecutionError::new(format!("cannot seek stdout"))),
            2 => return Err(ExecutionError::new(format!("cannot seek stderr"))),
            _ => {
                let (file, file_path) = self.get_file_from_vmfd(vmfd)?;
                match file.seek_relative(offset) {
                    Ok(_) => Ok(()),

                    Err(e) => {
                        return Err(ExecutionError::new(format!(
                            "failed to write to {file_path} :: {e}"
                        )))
                    }
                }
            }
        }
    }
    pub fn ftell(&mut self, vmfd: VMFD) -> Result<usize, ExecutionError> {
        match vmfd {
            0 => return Err(ExecutionError::new(format!("cannot tell stdin"))),
            1 => return Err(ExecutionError::new(format!("cannot tell stdout"))),
            2 => return Err(ExecutionError::new(format!("cannot tell stderr"))),
            _ => {
                let (file, file_path) = self.get_file_from_vmfd(vmfd)?;
                match file.stream_position() {
                    Ok(pos) => Ok(pos as usize),
                    Err(err) => Err(ExecutionError::new(format!(
                        "failed to read stream position of vmfd-{vmfd} :: {err}"
                    ))),
                }
            }
        }
    }
    pub fn exit(code: i32) -> ! {
        exit(code)
    }
    pub fn sleep(ns: u64) {
        todo!()
    }
    pub fn get_system_time() -> usize {
        todo!()
    }
}
