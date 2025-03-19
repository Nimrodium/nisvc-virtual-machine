// runtime rewrite
//

use std::{
    collections::HashMap,
    fmt::{self},
    fs::File,
    io::{self, Read, Seek, Stderr, Stdin, Stdout, Write},
    process::exit,
};

use colorize::AnsiColor;
use sdl2::libc::ERA;

use crate::{
    constant::{
        RegisterCode, RegisterWidth, VMAddress, ADDRESS_BYTES, GPR_COUNT, INIT_VALUE,
        MMIO_ADDRESS_SPACE, NAME, OPCODE_BYTES, PROGRAM_COUNTER, RAM_SIZE, REAL_STACK_POINTER,
        REGISTER_BYTES, RNULL, SIGNATURE, STACK_POINTER,
    },
    log_input, log_output,
    memory::Memory,
    opcode::OpcodeTable,
    verbose_println, very_verbose_println, very_very_verbose_println, DisplayMode, GLOBAL_CLOCK,
};

#[derive(Clone, Debug)]
pub enum VMErrorCode {
    MemoryAccessViolation,
    InitError,
    RegisterOverflow,
    InvalidRegisterCode,
    MemoryInitializationError,
    ExecFormatError,
    CLIArgError,
    InvalidOperationCode,
    GenericError,
    DisplayInitializationError,
    StackOverflow,
    StackUnderflow,
    ShellError,        // fatal
    ShellExit,         // exits shell
    ShellCommandError, // non fatal

    HostFileIOError,
    VMFileIOError,
}

#[derive(Clone)]
pub struct VMError {
    pub code: VMErrorCode,
    pub reason: String,
}
impl fmt::Debug for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = format!(
            "{NAME}: {} {} :: {}",
            "error:".red(),
            format!("{:?}", self.code).yellow(),
            self.reason
        );
        // let metadata = if let Some(md) = self.metadata.clone() {
        //     md.to_string()
        // } else {
        //     String::new()
        // };
        write!(f, "{string}")
    }
}
impl VMError {
    pub fn with_code(mut self, code: VMErrorCode) -> Self {
        self.code = code;
        self
    }
    pub fn new(code: VMErrorCode, reason: String) -> Self {
        Self { code, reason }
    }
}
impl From<String> for VMError {
    fn from(value: String) -> Self {
        Self {
            code: VMErrorCode::GenericError,
            reason: value,
        }
    }
}
#[derive(Clone)]
pub struct Register {
    pub value: RegisterWidth,
    pub base_name: String,
    pub code: RegisterCode,
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
    fn new(name: &str, code: RegisterCode) -> Self {
        Self {
            value: INIT_VALUE,
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
        // println!("pre {value:x}");
        let byte_mask = 0x00_00_00_00_00_00_00_FF;
        let clean_value = value & byte_mask;
        let byte_offset = i * 8;
        let byte_to_be_inserted = clean_value << byte_offset;
        // println!("{byte_to_be_inserted:0>16x} =\n{clean_value:0>16x} << {byte_offset}",);
        // println!("shift {byte_to_be_inserted:x} offset to byte {i}");
        // (b & ~a) | a
        let inverse_clear_dest_mask = !byte_mask.rotate_left(byte_offset as u32);
        let masked_reg = self.value & inverse_clear_dest_mask;
        // println!(
        //     "masked {} = \n{:0>16x}\n{:0>16x} &\n{masked_reg:0>16x}\n",
        //     self.name(),
        //     self.value,
        //     !byte_mask
        // );
        // println!("masked reg {masked_reg:#x}");

        self.value = masked_reg | byte_to_be_inserted;
        // println!(
        //     "{:0>16} =\n{masked_reg:0>16x} |\n{byte_to_be_inserted:0>16x}",
        //     self.value
        // );
        // println!("inserted {:x}", self.value);
    }

    pub fn write_at_quarter(&mut self, value: u64, i: u8) {
        if i > 4 || i <= 0 {
            panic!("attempted to read at an invalid quarter index {i} > 4")
        }
        let i = i - 1;
        // println!("inserting {value:#x}");
        let byte_offset = i * 16;

        let byte_mask = 0x00_00_00_00_00_00_FF_FF;
        let clean_value = value & byte_mask;
        // println!("masked_u64 = {clean_value:#x}");
        let quarter_to_be_inserted = clean_value << byte_offset;

        // println!("masked_u64 (shifted) = {quarter_to_be_inserted:#x}");
        let inverse_clear_dest_mask = !byte_mask.rotate_left(byte_offset as u32);
        let masked_reg = (self.value & inverse_clear_dest_mask);

        self.value = masked_reg | quarter_to_be_inserted;

        // println!("inserted {:x}", self.value);
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
        // println!("inserted {:x}", self.value);
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
    // fn write_byte_sub(&mut self, byte: u8, byte_index: u8) -> Result<(), VMError> {

    //         _ => {
    //             return Err(VMError::new(
    //                 VMErrorCode::RegisterOverflow,
    //                 format!("invalid byte index"),
    //             ))
    //         }
    //     };
    // }
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

    // pub fn _write(&mut self, value: RegisterWidth) -> Result<(), VMError> {
    //     if value > RegisterWidth::MAX {
    //         return Err(VMError {
    //             code: VMErrorCode::RegisterOverflow,
    //             reason: format!(
    //                 "cannot write {value} to register {} :: over max int {}",
    //                 self.name(),
    //                 RegisterWidth::MAX
    //             ),
    //         });
    //     }
    //     if !self.locked {
    //         self.value = value as RegisterWidth;
    //     } else {
    //         very_verbose_println!("attempted to write to locked register {}", self.name())
    //     }
    //     log_input!("{} <- {value}", self.name());
    //     Ok(())
    // }

    // fn write_from_slice(&mut self, bytes: &[u8]) -> Result<(), VMError> {
    //     let value = register_value_from_slice(bytes) as RegisterWidth;
    //     self.write(value);
    //     Ok(())
    // }

    // pub fn _read(&self) -> u64 {
    //     log_output!("{} -> {}", self.name(), self.value);
    //     self.value
    // }
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
    fn _new() -> Self {
        let mut registers: Vec<Register> = vec![];
        verbose_println!("initializing registers...");
        for n in 1..=GPR_COUNT {
            // let code = i + 1;
            let name = "r".to_string() + n.to_string().as_str();

            registers.push(Register::new(&name, n));
        }
        registers.push(Register::new("pc", PROGRAM_COUNTER));
        registers.push(Register::new("sp", STACK_POINTER));
        registers.push(Register::new("rsp", REAL_STACK_POINTER));
        let mut rnull = Register::new("null", RNULL);
        rnull.write(0);
        rnull.locked = true;
        registers.push(rnull);
        Self { registers }
    }
    fn new() -> Self {
        verbose_println!("initializing registers...");
        let mut registers: Vec<Register> = vec![
            Register::new("null", 0),
            Register::new("pc", 1),
            Register::new("sp", 2),
            Register::new("rsp", 3),
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
            Register::new("r13", 16),
            Register::new("r14", 17),
            Register::new("r15", 18),
        ];
        Self { registers }
    }
    pub fn get_register(&mut self, code: RegisterCode) -> Result<&Register, VMError> {
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
    pub fn get_mut_register(&mut self, code: RegisterCode) -> Result<&mut Register, VMError> {
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
        let windowed_register = register.as_window_mut(window);
        very_very_verbose_println!("passing register as {}", windowed_register.name());
        Ok(windowed_register)
    }
    pub fn get_register_via_reverse_lookup(
        &mut self,
        register_name: &str,
    ) -> Result<&mut Register, VMError> {
        let valid_sub_names = [
            "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "q1", "q2", "q3", "q4", "l", "h", "f",
        ];
        let (base_name, window) = match register_name {
            "null" | "pc" | "sp" | "rsp" => (register_name, "f"),

            _ => (&register_name[..2], {
                let sub = &register_name[2..];
                if sub.is_empty() {
                    "f"
                } else {
                    if valid_sub_names.contains(&sub) {
                        sub
                    } else {
                        return Err(VMError::new(
                            VMErrorCode::ShellCommandError,
                            format!("{sub} is not a valid subregister"),
                        ));
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
            Err(VMError::new(
                VMErrorCode::ShellCommandError,
                format!("{register_name} is not a valid register"),
            ))
        }
    }
    // pub fn _get_register(&self, code: RegisterCode) -> Result<&Register, VMError> {
    //     if code == 0 {
    //         return Err(VMError::new(
    //             VMErrorCode::InvalidRegisterCode,
    //             format!("{code} evaluates to -1 which is not a valid register code"),
    //         ));
    //     }
    //     let code_index = (code - 1) as usize;
    //     let register = match self.registers.get(code_index) {
    //         Some(r) => r,
    //         None => {
    //             return Err(VMError {
    //                 code: VMErrorCode::InvalidRegisterCode,
    //                 reason: format!("{code_index:#x} is not a valid register code"),
    //             })
    //         }
    //     };
    //     Ok(register)
    // }
    // pub fn _get_mut_register(&mut self, code: RegisterCode) -> Result<&mut Register, VMError> {
    //     very_very_verbose_println!("accessing register code {code}");
    //     if code == 0 {
    //         return Err(VMError::new(
    //             VMErrorCode::InvalidRegisterCode,
    //             format!("{code} evaluates to -1 which is not a valid register code"),
    //         ));
    //     }
    //     let code_index = (code - 1) as usize;
    //     let register = match self.registers.get_mut(code_index) {
    //         Some(r) => r,
    //         None => {
    //             return Err(VMError {
    //                 code: VMErrorCode::InvalidRegisterCode,
    //                 reason: format!("{code_index:#x} is not a valid register code"),
    //             })
    //         }
    //     };
    //     Ok(register)
    // }

    // pub fn get_register_via_reverse_lookup(
    //     &mut self,
    //     register_name: &str,
    // ) -> Result<&mut Register, VMError> {
    //     let mut reg_buf: Option<&mut Register> = None;
    //     for register in &mut self.registers {
    //         if register.name() == register_name {
    //             reg_buf = Some(register)
    //         }
    //     }
    //     if let Some(reg) = reg_buf {
    //         return Ok(reg);
    //     } else {
    //         return Err(VMError::new(
    //             VMErrorCode::ShellCommandError,
    //             format!("register {register_name} is not a valid register"),
    //         ));
    //     }
    // }
}

pub struct CPU {
    pub registers: CPURegisters,
    pub memory: Memory,
    opcode_table: OpcodeTable,
    pub stack_base: RegisterWidth,
    pub stack_max: RegisterWidth,
    pub vm_host_bridge: VMHostBridge,
    clock_speed: usize,
    pub ignore_breakpoints: bool,
    pub default_breakpoint_behavior: bool,
}
impl CPU {
    pub fn new(
        clock_speed: usize,
        display: DisplayMode,
        ignore_breakpoints: bool,
    ) -> Result<Self, VMError> {
        let registers = CPURegisters::new();
        very_very_verbose_println!("registers:\n{registers}");
        let memory = match Memory::new(display) {
            Ok(m) => m,
            Err(err) => {
                return Err(VMError {
                    code: VMErrorCode::MemoryInitializationError,
                    reason: format!("TMP MESSAGE memory failed to initialize :: {err}"),
                })
            }
        };
        let opcode_table = OpcodeTable::new();
        Ok(Self {
            registers,
            memory,
            opcode_table,
            stack_base: RAM_SIZE,
            stack_max: 0,
            vm_host_bridge: VMHostBridge::new(),
            clock_speed,
            ignore_breakpoints,
            default_breakpoint_behavior: ignore_breakpoints,
        })
    }

    pub fn load(&mut self, file_path: &str) -> Result<(), VMError> {
        verbose_println!("loading file {file_path} ...");
        let mut file = match File::open(file_path) {
            Ok(f) => f,
            Err(err) => {
                return Err(VMError {
                    code: VMErrorCode::InitError,
                    reason: format!("could not open {file_path} :: {err}"),
                })
            }
        };
        let mut file_buf: Vec<u8> = vec![];
        match file.read_to_end(&mut file_buf) {
            Ok(bytes_read) => verbose_println!("read {bytes_read} byte(s)"),
            Err(err) => {
                return Err(VMError {
                    code: VMErrorCode::InitError,
                    reason: format!("could not read {file_path} :: {err}"),
                })
            }
        };
        let nisvc_ef_file = NISVCEF::new(&file_buf)?;
        self.memory.program = nisvc_ef_file.program_image.to_vec();
        self.memory.flash_ram(nisvc_ef_file.ram_image).unwrap();
        self.memory.ram_base =
            (MMIO_ADDRESS_SPACE + nisvc_ef_file.program_image.len()) as RegisterWidth;

        self.stack_max =
            self.memory.ram_base + (nisvc_ef_file.ram_image.len() + 1) as RegisterWidth;
        self.registers
            .get_mut_register(STACK_POINTER)?
            .write(self.stack_base);
        self.registers
            .get_mut_register(PROGRAM_COUNTER)?
            .write(nisvc_ef_file.entry_point);

        verbose_println!("{nisvc_ef_file}");
        verbose_println!(
            "stack address range:  {}..{}",
            self.stack_max,
            self.stack_base
        );
        Ok(())
    }

    fn read_from_pc(&mut self) -> Result<u8, VMError> {
        let op_addr = self.registers.get_register(PROGRAM_COUNTER)?.read();
        let mem_byte = self.memory.mmu_read(op_addr as RegisterWidth)?;
        Ok(mem_byte)
    }

    pub fn read_operands(&mut self, requested_slice_length: usize) -> Result<Vec<u8>, VMError> {
        // let start_address = self.read_from_pc()? as RegisterWidth;
        let start_address =
            (self.registers.get_register(PROGRAM_COUNTER)?.read() as usize) + OPCODE_BYTES;

        let operand_bytes = self
            .memory
            .read_bytes(start_address as RegisterWidth, requested_slice_length)?;
        very_verbose_println!(
            "operand bytes read from {start_address}..{} {operand_bytes:?}",
            requested_slice_length + start_address
        );
        Ok(operand_bytes)
    }

    pub fn step(&mut self) -> Result<(), VMError> {
        // let op_addr = self.registers.get_mut_register(PROGRAM_COUNTER)?.read();
        // let op_code = self.memory.mmu_read(op_addr as RegisterWidth)?;
        let byte_at_pc = self.read_from_pc()?;
        // let op_addr = self.registers.get_register(PROGRAM_COUNTER)?.read();
        // let byte_at_pc = self.memory.mmu_read(op_addr as RegisterWidth)?;
        // verbose_println!("pc::{op_addr}");
        let operation = self.opcode_table.decode(byte_at_pc)?;
        very_very_verbose_println!("read operation {}", operation.name);
        let bytes_read = match operation.code {
            0x0 => self.op_nop()?,
            0x1 => self.op_mov()?,
            0x2 => self.op_movim()?,
            0x3 => self.op_load()?,
            0x4 => self.op_store()?,
            0x5 => self.op_add()?,
            0x6 => self.op_sub()?,
            0x7 => self.op_mult()?,
            0x8 => self.op_div()?,
            0x9 => self.op_or()?,
            0xa => self.op_xor()?,
            0xb => self.op_and()?,
            0xc => self.op_not()?,
            0xd => self.op_shl()?,
            0xe => self.op_shr()?,
            0xf => self.op_rotl()?,
            0x10 => self.op_rotr()?,
            0x11 => self.op_neg()?,
            0x12 => self.op_jmp()?,
            0x13 => self.op_jifz()?,
            0x14 => self.op_jifnz()?,
            0x15 => self.op_pr()?,
            0x16 => self.op_inc()?,
            0x17 => self.op_dec()?,
            0x18 => self.op_push()?,
            0x19 => self.op_pop()?,
            0x1a => self.op_call()?,
            0x1b => self.op_ret()?,
            0x1c => self.op_cache()?,
            0x1d => self.op_restore()?,
            0x1e => self.op_fopen()?,
            0x1f => self.op_fread()?,
            0x20 => self.op_fwrite()?,
            0x21 => self.op_fseek()?,
            0x22 => self.op_fclose()?,
            //special
            0xfe => self.op_breakpoint()?,

            _ => unreachable!(),
        };
        let pc = self.registers.get_mut_register(PROGRAM_COUNTER)?;
        let new_pos = pc.read() + (bytes_read as RegisterWidth);
        very_verbose_println!("advancing pc {bytes_read} byte(s)");
        pc.write(new_pos);
        unsafe { GLOBAL_CLOCK += 1 }
        Ok(())
    }
    pub fn exec(&mut self) -> Result<(), VMError> {
        verbose_println!("executing program ...");
        let clock_sleep = std::time::Duration::from_millis(1000 / self.clock_speed as u64);

        // verbose_println!("sleeping {clock_sleep}");
        loop {
            std::thread::sleep(clock_sleep);
            self.step()?;
            let byte_at_pc = self.read_from_pc()?;
            if byte_at_pc == 0xFF {
                self.memory.halt_exe_drop();
                break;
            };
        }
        Ok(())
    }

    pub fn trinary_operation_decode(
        &mut self,
    ) -> Result<(&mut Register, Register, Register, usize), VMError> {
        let bytes_read = REGISTER_BYTES * 3;
        let operand_bytes = self.read_operands(bytes_read)?;
        let op2 = self.registers.get_register(operand_bytes[2])?.clone();
        let op1 = self.registers.get_register(operand_bytes[1])?.clone();
        let dest = self.registers.get_mut_register(operand_bytes[0])?;
        Ok((dest, op1, op2, bytes_read))
    }
    pub fn binary_operation_decode(&mut self) -> Result<(&mut Register, Register, usize), VMError> {
        let bytes_read = REGISTER_BYTES * 2;
        let operand_bytes = self.read_operands(bytes_read)?;
        let op = self.registers.get_register(operand_bytes[1])?.clone();
        let dest = self.registers.get_mut_register(operand_bytes[0])?;
        Ok((dest, op, bytes_read))
    }

    pub fn unary_operation_decode(&mut self) -> Result<(&mut Register, usize), VMError> {
        let bytes_read = REGISTER_BYTES;
        let operand_bytes = self.read_operands(bytes_read)?;
        let dest = self.registers.get_mut_register(operand_bytes[0])?;
        Ok((dest, bytes_read))
    }
    pub fn jif_decode(
        &mut self,
    ) -> Result<(&mut Register, Register, RegisterWidth, usize), VMError> {
        let bytes_read = REGISTER_BYTES + ADDRESS_BYTES;
        let operand_bytes = self.read_operands(bytes_read)?;
        let condition = self.registers.get_register(operand_bytes[0])?.clone();
        let address = register_value_from_slice(&operand_bytes[1..]) as RegisterWidth;
        let pc = self.registers.get_mut_register(PROGRAM_COUNTER)?;
        Ok((pc, condition, address, bytes_read))
    }

    // stack grows downward
    pub fn push(&mut self, value: RegisterWidth) -> Result<(), VMError> {
        let sp_current = self.registers.get_register(STACK_POINTER)?.read();
        let value_bytes = value.to_le_bytes();
        let sp_new = sp_current - size_of::<RegisterWidth>() as RegisterWidth;
        if sp_new < self.stack_max {
            let position = self.registers.get_register(PROGRAM_COUNTER)?.read();
            return Err(VMError::new(
                VMErrorCode::StackOverflow,
                format!("stack overflow at pc {position} sp {sp_new}"),
            ));
        } else if sp_new > self.stack_base {
            let position = self.registers.get_register(PROGRAM_COUNTER)?.read();
            return Err(VMError::new(
                VMErrorCode::StackUnderflow,
                format!("stack underflow at pc {position} sp {sp_new}"),
            ));
        }
        self.memory.write_bytes(sp_current, &value_bytes)?;
        let sp = self.registers.get_mut_register(STACK_POINTER)?;
        sp.write(sp_new);
        // if sp.read() < self.stack_max {
        //     return Err(VMError::new(VMErrorCode::, reason))
        // }
        verbose_println!("pushed {value} onto the stack from sp {sp_current}");
        Ok(())
    }

    pub fn pop(&mut self) -> Result<RegisterWidth, VMError> {
        let sp = self.registers.get_mut_register(STACK_POINTER)?;
        let sp_current = sp.read();
        let sp_new = sp_current + size_of::<RegisterWidth>() as RegisterWidth;
        sp.write(sp_new);
        let value_bytes = self.memory.read_bytes(sp_new, size_of::<RegisterWidth>())?;
        let value = register_value_from_slice(&value_bytes);
        // self.registers.get_mut_register(STACK_POINTER)?.write()?;
        verbose_println!("popped {value} off the stack from sp {sp_new}");
        Ok(value)
    }
}

struct NISVCEF<'a> {
    entry_point: VMAddress,
    program_image: &'a [u8],
    ram_image: &'a [u8],
    debug_table: DebugTable,

    program_len: usize,
    ram_len: usize,
    debug_len: usize,
}
impl<'a> fmt::Display for NISVCEF<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HEADER: {}\n\tprogram_len::{}\n\tram_len::{}\n\tdebug_len::{}",
            String::from_utf8(SIGNATURE.to_vec()).unwrap(),
            self.program_len,
            self.ram_len,
            self.debug_len
        )
    }
}
impl<'a> NISVCEF<'a> {
    fn new(file: &'a [u8]) -> Result<Self, VMError> {
        verbose_println!("parsing NISVC executable format file ... ");
        let mut head = 0;
        let header_length = SIGNATURE.len() + (8 * 4);
        let header_bytes = match file.get(head..head + header_length) {
            Some(h) => h,
            None => {
                return Err(VMError {
                    code: VMErrorCode::ExecFormatError,
                    reason: "file has incomplete header or is not an executable file".to_string(),
                })
            }
        };
        let (program_length, ram_image_length, entry_point_address, debug_partition_length) =
            NISVCEF::read_header(header_bytes)?;
        head += header_length;

        let program_image = match file.get(head..head + program_length) {
            Some(p) => p,
            None => {
                return Err(VMError {
                    code: VMErrorCode::ExecFormatError,
                    reason: format!(
                        "could not read program partition from {head}..{program_length}"
                    ),
                })
            }
        };
        head += program_length;

        let ram_image = match file.get(head..head + ram_image_length) {
            Some(p) => p,
            None => {
                return Err(VMError {
                    code: VMErrorCode::ExecFormatError,
                    reason: format!(
                        "could not read ram image partition from {head}..{ram_image_length}"
                    ),
                })
            }
        };
        head += ram_image_length;

        let debug_bytes = match file.get(head..head + debug_partition_length) {
            Some(p) => p,
            None => {
                return Err(VMError {
                    code: VMErrorCode::ExecFormatError,
                    reason: format!(
                        "could not read debug partition from {head}..{debug_partition_length}"
                    ),
                })
            }
        };
        head += debug_partition_length;
        let debug_table = DebugTable::new(debug_bytes);

        Ok(Self {
            entry_point: entry_point_address as VMAddress,
            program_image,
            ram_image,
            debug_table,
            program_len: program_length,
            ram_len: ram_image_length,
            debug_len: debug_partition_length,
        })
    }
    fn read_header(header: &[u8]) -> Result<(usize, usize, usize, usize), VMError> {
        // let header_iter = header.iter();
        let mut head = 0;
        let file_signature = &header[0..SIGNATURE.len()];
        if file_signature != SIGNATURE {
            return Err(VMError {
                code: VMErrorCode::ExecFormatError,
                reason: format!("invalid header"),
            });
        }
        head += SIGNATURE.len();
        let program_length = register_value_from_slice(&header[head..head + HEADER_ENTRY_LENGTH]);
        head += HEADER_ENTRY_LENGTH;
        let ram_image_length = register_value_from_slice(&header[head..head + HEADER_ENTRY_LENGTH]);
        head += HEADER_ENTRY_LENGTH;
        let entry_point_address =
            register_value_from_slice(&header[head..head + HEADER_ENTRY_LENGTH]);
        head += HEADER_ENTRY_LENGTH;
        let debug_partition_length =
            register_value_from_slice(&header[head..head + HEADER_ENTRY_LENGTH]);
        Ok((
            program_length as usize,
            ram_image_length as usize,
            entry_point_address as usize,
            debug_partition_length as usize,
        ))
    }
}

const HEADER_ENTRY_LENGTH: usize = 8;

pub fn register_value_from_slice(slice: &[u8]) -> RegisterWidth {
    let target_length = size_of::<usize>();
    let mut byte_buf: Vec<u8> = Vec::with_capacity(target_length);
    byte_buf.extend_from_slice(slice);
    byte_buf.resize(target_length, 0);
    let byte_array: [u8; size_of::<u64>()] = match byte_buf.try_into() {
        Ok(arr) => arr,
        Err(err) => panic!("failed to convert sequence despite padding :: {err:?}"),
    };
    RegisterWidth::from_le_bytes(byte_array)
}

struct DebugTable;
impl DebugTable {
    fn new(partition: &[u8]) -> Self {
        verbose_println!("debug table not implemented yet");
        Self
    }
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
    fn get_file_from_vmfd(&mut self, vmfd: VMFD) -> Result<&mut (File, String), VMError> {
        let open_files = self.open_file_vector.len();
        match self.open_file_vector.get_mut(&vmfd) {
            Some(f) => Ok(f),
            None => Err(VMError::new(
                VMErrorCode::VMFileIOError,
                format!(
                    "vmfd-{vmfd} does not exist, there are {} open files",
                    open_files
                ),
            )),
        }
    }
    pub fn fopen(&mut self, file_path: &str) -> Result<VMFD, VMError> {
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(why) => {
                return Err(VMError::new(
                    VMErrorCode::HostFileIOError,
                    format!("failed to open host file {file_path} :: {why}"),
                ))
            }
        };
        let vmfd = self.next_vmfd;
        self.next_vmfd += 1;

        self.open_file_vector
            .insert(vmfd, (file, file_path.to_string()));

        verbose_println!("opened file {file_path} as vmfd-{vmfd}");
        Ok(vmfd)
    }
    pub fn fclose(&mut self, vmfd: VMFD) -> Result<(), VMError> {
        match vmfd {
            0 => {
                return Err(VMError::new(
                    VMErrorCode::VMFileIOError,
                    format!("cannot close stdin"),
                ))
            }
            1 => {
                return Err(VMError::new(
                    VMErrorCode::HostFileIOError,
                    format!("cannot cloes stdout"),
                ))
            }
            2 => {
                return Err(VMError::new(
                    VMErrorCode::HostFileIOError,
                    format!("cannot close stderr"),
                ))
            }
            _ => {
                // let (file, file_path) = self.get_file_from_vmfd(vmfd)?;
                let (file, file_path) = match self.open_file_vector.remove(&vmfd) {
                    Some(entry) => entry,
                    None => {
                        return Err(VMError::new(
                            VMErrorCode::VMFileIOError,
                            format!("{vmfd} is not an open file handle"),
                        ))
                    }
                };
                drop(file);
                verbose_println!("closed vmfd-{vmfd} :: {file_path}");
                Ok(())
            }
        }
    }
    pub fn fwrite(&mut self, vmfd: VMFD, buf: &[u8]) -> Result<(), VMError> {
        match vmfd {
            0 => {
                return Err(VMError::new(
                    VMErrorCode::VMFileIOError,
                    format!("cannot write to stdin"),
                ))
            }
            1 => match self.stdout.write_all(buf) {
                Ok(_) => Ok(()),
                Err(e) => {
                    return Err(VMError::new(
                        VMErrorCode::HostFileIOError,
                        format!("failed to write to stdout :: {e}"),
                    ))
                }
            },
            2 => match self.stderr.write_all(buf) {
                Ok(_) => Ok(()),
                Err(e) => {
                    return Err(VMError::new(
                        VMErrorCode::HostFileIOError,
                        format!("failed to write to stderr :: {e}"),
                    ))
                }
            },
            _ => {
                let (file, file_path) = self.get_file_from_vmfd(vmfd)?;
                match file.write_all(buf) {
                    Ok(_) => Ok(()),

                    Err(e) => {
                        return Err(VMError::new(
                            VMErrorCode::HostFileIOError,
                            format!("failed to write to {file_path} :: {e}"),
                        ))
                    }
                }
            }
        }
    }

    pub fn fread(&mut self, vmfd: VMFD, length: usize) -> Result<Vec<u8>, VMError> {
        let mut buf: Vec<u8> = vec![0u8; length];
        match vmfd {
            0 => match self.stdin.read(&mut buf) {
                Ok(bytes_read) => (),
                Err(e) => {
                    return Err(VMError::new(
                        VMErrorCode::HostFileIOError,
                        format!("failed to read from stdin :: {e}"),
                    ))
                }
            },
            1 => {
                return Err(VMError::new(
                    VMErrorCode::VMFileIOError,
                    format!("cannot read from stdout"),
                ))
            }
            2 => {
                return Err(VMError::new(
                    VMErrorCode::VMFileIOError,
                    format!("cannot read from stderr"),
                ))
            }
            _ => {
                let (file, file_path) = self.get_file_from_vmfd(vmfd)?;
                match file.read_exact(&mut buf) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(VMError::new(
                            VMErrorCode::HostFileIOError,
                            format!("failed to read from {file_path} :: {e}"),
                        ))
                    }
                }
            }
        };
        Ok(buf)
    }

    pub fn fseek(&mut self, vmfd: VMFD, amount: usize, direction: u8) -> Result<(), VMError> {
        let offset: i64 = if direction == 1 {
            amount as i64 * -1
        } else if direction == 0 {
            amount as i64
        } else {
            return Err(VMError::new(
                VMErrorCode::HostFileIOError,
                format!("invalid seek direction"),
            ));
        };
        match vmfd {
            0 => {
                return Err(VMError::new(
                    VMErrorCode::VMFileIOError,
                    format!("cannot seek stdin"),
                ))
            }
            1 => {
                return Err(VMError::new(
                    VMErrorCode::HostFileIOError,
                    format!("cannot seek stdout"),
                ))
            }
            2 => {
                return Err(VMError::new(
                    VMErrorCode::HostFileIOError,
                    format!("cannot seek stderr"),
                ))
            }
            _ => {
                let (file, file_path) = self.get_file_from_vmfd(vmfd)?;
                match file.seek_relative(offset) {
                    Ok(_) => Ok(()),

                    Err(e) => {
                        return Err(VMError::new(
                            VMErrorCode::HostFileIOError,
                            format!("failed to write to {file_path} :: {e}"),
                        ))
                    }
                }
            }
        }
    }
    pub fn ftell(&mut self, vmfd: VMFD) -> Result<usize, VMError> {
        match vmfd {
            0 => {
                return Err(VMError::new(
                    VMErrorCode::VMFileIOError,
                    format!("cannot tell stdin"),
                ))
            }
            1 => {
                return Err(VMError::new(
                    VMErrorCode::HostFileIOError,
                    format!("cannot tell stdout"),
                ))
            }
            2 => {
                return Err(VMError::new(
                    VMErrorCode::HostFileIOError,
                    format!("cannot tell stderr"),
                ))
            }
            _ => {
                let (file, file_path) = self.get_file_from_vmfd(vmfd)?;
                match file.stream_position() {
                    Ok(pos) => Ok(pos as usize),
                    Err(err) => Err(VMError::new(
                        VMErrorCode::HostFileIOError,
                        format!("failed to read stream position of vmfd-{vmfd} :: {err}"),
                    )),
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
