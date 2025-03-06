// runtime rewrite
//

use std::{
    fmt::{self},
    fs::File,
    io::Read,
};

use colorize::AnsiColor;

use crate::{
    constant::{
        RegisterCode, RegisterWidth, VMAddress, ADDRESS_BYTES, INIT_VALUE, MMIO_ADDRESS_SPACE,
        NAME, OPCODE_BYTES, PROGRAM_COUNTER, RAM_SIZE, REAL_STACK_POINTER, REGISTER_BYTES,
        REGISTER_COUNT, RNULL, SIGNATURE, STACK_POINTER,
    },
    log_input, log_output,
    memory::Memory,
    opcode::OpcodeTable,
    verbose_println, very_verbose_println, very_very_verbose_println, GLOBAL_CLOCK,
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
    pub name: String,
    pub code: RegisterCode,
    pub locked: bool,
}

impl Register {
    fn new(name: &str, code: RegisterCode) -> Self {
        Self {
            value: INIT_VALUE,
            name: name.to_string(),
            locked: false,
            code,
        }
    }

    pub fn write(&mut self, value: RegisterWidth) -> Result<(), VMError> {
        if value > RegisterWidth::MAX {
            return Err(VMError {
                code: VMErrorCode::RegisterOverflow,
                reason: format!(
                    "cannot write {value} to register {} :: over max int {}",
                    self.name,
                    RegisterWidth::MAX
                ),
            });
        }
        if !self.locked {
            self.value = value as RegisterWidth;
        } else {
            very_verbose_println!("attempted to write to locked register {}", self.name)
        }
        log_input!("{} <- {value}", self.name);
        Ok(())
    }

    fn write_from_slice(&mut self, bytes: &[u8]) -> Result<(), VMError> {
        let value = register_value_from_slice(bytes) as RegisterWidth;
        self.write(value)?;
        Ok(())
    }

    pub fn read(&self) -> RegisterWidth {
        log_output!("{} -> {}", self.name, self.value);
        self.value
    }
}
impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ {} ({}) ]", self.name, self.value)
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
        let mut registers: Vec<Register> = vec![];
        verbose_println!("initializing registers...");
        for n in 1..=REGISTER_COUNT {
            // let code = i + 1;
            let name = "r".to_string() + n.to_string().as_str();

            registers.push(Register::new(&name, n));
        }
        registers.push(Register::new("pc", PROGRAM_COUNTER));
        registers.push(Register::new("sp", STACK_POINTER));
        registers.push(Register::new("rsp", REAL_STACK_POINTER));
        let mut rnull = Register::new("null", RNULL);
        rnull.write(0).unwrap();
        rnull.locked = true;
        registers.push(rnull);
        Self { registers }
    }
    pub fn get_register(&self, code: RegisterCode) -> Result<&Register, VMError> {
        if code == 0 {
            return Err(VMError::new(
                VMErrorCode::InvalidRegisterCode,
                format!("{code} evaluates to -1 which is not a valid register code"),
            ));
        }
        let code_index = (code - 1) as usize;
        let register = match self.registers.get(code_index) {
            Some(r) => r,
            None => {
                return Err(VMError {
                    code: VMErrorCode::InvalidRegisterCode,
                    reason: format!("{code_index} is not a valid register code"),
                })
            }
        };
        Ok(register)
    }
    pub fn get_mut_register(&mut self, code: RegisterCode) -> Result<&mut Register, VMError> {
        very_very_verbose_println!("accessing register code {code}");
        if code == 0 {
            return Err(VMError::new(
                VMErrorCode::InvalidRegisterCode,
                format!("{code} evaluates to -1 which is not a valid register code"),
            ));
        }
        let code_index = (code - 1) as usize;
        let register = match self.registers.get_mut(code_index) {
            Some(r) => r,
            None => {
                return Err(VMError {
                    code: VMErrorCode::InvalidRegisterCode,
                    reason: format!("{code_index} is not a valid register code"),
                })
            }
        };
        Ok(register)
    }

    pub fn get_register_via_reverse_lookup(
        &mut self,
        register_name: &str,
    ) -> Result<&mut Register, VMError> {
        let mut reg_buf: Option<&mut Register> = None;
        for register in &mut self.registers {
            if register.name == register_name {
                reg_buf = Some(register)
            }
        }
        if let Some(reg) = reg_buf {
            return Ok(reg);
        } else {
            return Err(VMError::new(
                VMErrorCode::ShellCommandError,
                format!("register {register_name} is not a valid register"),
            ));
        }
    }
}

pub struct CPU {
    pub registers: CPURegisters,
    pub memory: Memory,
    opcode_table: OpcodeTable,
    pub stack_base: RegisterWidth,
    pub stack_max: RegisterWidth,
    clock_speed: usize,
}
impl CPU {
    pub fn new(clock_speed: usize) -> Result<Self, VMError> {
        let registers = CPURegisters::new();
        very_very_verbose_println!("registers:\n{registers}");
        let memory = match Memory::new() {
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
            clock_speed,
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
            .write(self.stack_base)?;
        self.registers
            .get_mut_register(PROGRAM_COUNTER)?
            .write(nisvc_ef_file.entry_point)?;

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

            _ => unreachable!(),
        };
        let pc = self.registers.get_mut_register(PROGRAM_COUNTER)?;
        let new_pos = pc.read() + (bytes_read as RegisterWidth);
        very_verbose_println!("advancing pc {bytes_read} byte(s)");
        pc.write(new_pos)?;
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
        sp.write(sp_new)?;
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
        sp.write(sp_new)?;
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
