// runtime rewrite
//

use std::{
    fmt::{self},
    fs::File,
    io::Read,
};

use colorize::AnsiColor;
use sdl2::sys::GenericEvent;

use crate::{
    constant::{
        RegisterCode, RegisterWidth, VMAddress, ADDRESS_BYTES, CLOCK_SPEED_MS, INIT_VALUE,
        MMIO_ADDRESS_SPACE, NAME, OPCODE_BYTES, REGISTER_BYTES, SIGNATURE,
    },
    log_disassembly,
    memory::Memory,
    opcode::OpcodeTable,
    verbose_println, very_verbose_println, very_very_verbose_println,
};

#[derive(Clone, Debug)]
pub enum VMErrorCode {
    MemoryAccessViolation,
    InitError,
    RegisterOverflow,
    RegisterBytesIncorrectSize,
    InvalidRegisterCode,
    MemoryInitializationError,
    ExecFormatError,
    CLIArgError,
    InvalidOperationCode,
    GenericError,
    DisplayInitializationError,
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
struct Register {
    pub value: RegisterWidth,
    pub name: String,
    pub code: RegisterCode,
}

impl Register {
    fn new(name: &str, code: RegisterCode) -> Self {
        Self {
            value: INIT_VALUE,
            name: name.to_string(),
            code,
        }
    }

    fn write(&mut self, value: usize) -> Result<(), VMError> {
        if value > RegisterWidth::MAX as usize {
            return Err(VMError {
                code: VMErrorCode::RegisterOverflow,
                reason: format!(
                    "cannot write {value} to register {} :: over max int {}",
                    self.name,
                    RegisterWidth::MAX
                ),
            });
        }
        self.value = value as RegisterWidth;
        verbose_println!("{} <- {value}", self.name);
        Ok(())
    }

    fn write_from_slice(&mut self, bytes: &[u8]) -> Result<(), VMError> {
        let value = slice_to_usize(bytes);
        self.write(value)?;
        Ok(())
    }

    fn read(&self) -> usize {
        very_verbose_println!("{} -> {}", self.name, self.value);
        self.value as usize
    }
}
impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ {} ({}) ]", self.name, self.value)
    }
}

struct CPURegisters {
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
const REGISTER_COUNT: u8 = 20;
const PROGRAM_COUNTER: u8 = REGISTER_COUNT + 1;
impl CPURegisters {
    fn new() -> Self {
        let mut registers: Vec<Register> = vec![];

        for i in 0..REGISTER_COUNT {
            let code = i + 1;
            let name = "r".to_string() + code.to_string().as_str();

            registers.push(Register::new(&name, code));
        }
        registers.push(Register::new("pc", PROGRAM_COUNTER));
        registers.push(Register::new("sp", PROGRAM_COUNTER + 1));

        Self { registers }
    }
    fn get_register(&self, code: RegisterCode) -> Result<&Register, VMError> {
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
    fn get_mut_register(&mut self, code: RegisterCode) -> Result<&mut Register, VMError> {
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
}

pub struct CPU {
    registers: CPURegisters,
    memory: Memory,
    opcode_table: OpcodeTable,
}
impl CPU {
    pub fn new() -> Result<Self, VMError> {
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
        })
    }
    pub fn load(&mut self, file_path: &str) -> Result<(), VMError> {
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

        let pc = self.registers.get_mut_register(PROGRAM_COUNTER)?;
        pc.write(nisvc_ef_file.entry_point as usize)?;

        verbose_println!("{nisvc_ef_file}");
        Ok(())
    }

    fn read_from_pc(&mut self) -> Result<u8, VMError> {
        let op_addr = self.registers.get_register(PROGRAM_COUNTER)?.read();
        let mem_byte = self.memory.mmu_read(op_addr as RegisterWidth)?;
        Ok(mem_byte)
    }

    fn read_operands(&mut self, requested_slice_length: usize) -> Result<Vec<u8>, VMError> {
        // let start_address = self.read_from_pc()? as RegisterWidth;
        let start_address = self.registers.get_register(PROGRAM_COUNTER)?.read() + 1;

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

            _ => unreachable!(),
        };
        let pc = self.registers.get_mut_register(PROGRAM_COUNTER)?;
        let new_pos = pc.read() + bytes_read;
        very_verbose_println!("advancing pc {bytes_read} byte(s)");
        pc.write(new_pos)?;
        Ok(())
    }
    pub fn exec(&mut self) -> Result<(), VMError> {
        let clock_sleep = std::time::Duration::from_millis(CLOCK_SPEED_MS);
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

    fn trinary_operation_decode(
        &mut self,
    ) -> Result<(&mut Register, Register, Register, usize), VMError> {
        let bytes_read = REGISTER_BYTES * 3;
        let operand_bytes = self.read_operands(bytes_read)?;
        let op2 = self.registers.get_register(operand_bytes[2])?.clone();
        let op1 = self.registers.get_register(operand_bytes[1])?.clone();
        let dest = self.registers.get_mut_register(operand_bytes[0])?;
        Ok((dest, op1, op2, bytes_read))
    }
    fn binary_operation_decode(&mut self) -> Result<(&mut Register, Register, usize), VMError> {
        let bytes_read = REGISTER_BYTES * 2;
        let operand_bytes = self.read_operands(bytes_read)?;
        let op = self.registers.get_register(operand_bytes[1])?.clone();
        let dest = self.registers.get_mut_register(operand_bytes[0])?;
        Ok((dest, op, bytes_read))
    }

    fn unary_operation_decode(&mut self) -> Result<(&mut Register, usize), VMError> {
        let bytes_read = REGISTER_BYTES;
        let operand_bytes = self.read_operands(bytes_read)?;
        let dest = self.registers.get_mut_register(operand_bytes[0])?;
        Ok((dest, bytes_read))
    }
    fn jif_decode(&mut self) -> Result<(&mut Register, Register, usize, usize), VMError> {
        let bytes_read = REGISTER_BYTES + ADDRESS_BYTES;
        let operand_bytes = self.read_operands(bytes_read)?;
        let condition = self.registers.get_register(operand_bytes[0])?.clone();
        let address = slice_to_usize(&operand_bytes[1..]);
        let pc = self.registers.get_mut_register(PROGRAM_COUNTER)?;
        Ok((pc, condition, address, bytes_read))
    }
    fn op_nop(&mut self) -> Result<usize, VMError> {
        log_disassembly!("nop");
        Ok(OPCODE_BYTES)
    }
    fn op_mov(&mut self) -> Result<usize, VMError> {
        let bytes_read = REGISTER_BYTES * 2;
        let operand_bytes = self.read_operands(bytes_read)?;

        let source_register = self.registers.get_register(operand_bytes[1])?;
        let source_value = source_register.read();
        let src_reg_name = source_register.name.clone();

        let destination_register = self.registers.get_mut_register(operand_bytes[0])?;
        log_disassembly!("mov {}, {src_reg_name}", destination_register.name);
        destination_register.write(source_value)?;

        Ok(OPCODE_BYTES + bytes_read)
    }
    fn op_movim(&mut self) -> Result<usize, VMError> {
        // 01 02 05 01
        let bytes_read = REGISTER_BYTES + 1;
        let operands = self.read_operands(bytes_read)?;
        let dest_reg_code = operands[0];
        let size = operands[1] as usize;
        let operands_with_immediate = self.read_operands(bytes_read + size)?;
        let immediate = slice_to_usize(&operands_with_immediate[bytes_read..]);
        let dest_reg = self.registers.get_mut_register(dest_reg_code)?;
        log_disassembly!("movim {}, ${immediate}", dest_reg.name);
        dest_reg.write(immediate)?;
        let total_bytes_read = OPCODE_BYTES + bytes_read + size;
        Ok(total_bytes_read)
    }
    /// load x,y,z
    /// loads bytes starting from z and extending out y bytes into rx up to x's maximum (8 bytes)
    fn op_load(&mut self) -> Result<usize, VMError> {
        // let (_, size, addr, bytes_read) = self.trinary_operation_decode()?;
        // cannot use dest as provided because it needs to access memory as mutable

        let bytes_read = REGISTER_BYTES * 3;
        let operand_bytes = self.read_operands(bytes_read)?;
        let addr = self.registers.get_register(operand_bytes[2])?.clone();
        let size = self.registers.get_register(operand_bytes[1])?.clone();

        let bytes = self
            .memory
            .read_bytes(addr.read() as RegisterWidth, size.read())?;
        let value = slice_to_usize(&bytes);
        verbose_println!("(load) {bytes:?} -> {value}");

        let dest = self.registers.get_mut_register(operand_bytes[0])?;
        dest.write(value)?;
        log_disassembly!("load {}, {}, {}", dest.name, size.name, addr.name);
        Ok(OPCODE_BYTES + bytes_read)
    }
    fn op_store(&mut self) -> Result<usize, VMError> {
        let bytes_read = REGISTER_BYTES * 3;
        let operand_bytes = self.read_operands(bytes_read)?;
        let src_reg = self.registers.get_register(operand_bytes[2])?.clone();
        let size = self.registers.get_register(operand_bytes[1])?.clone();

        let dest = self.registers.get_mut_register(operand_bytes[0])?;
        log_disassembly!("store {}, {}, {}", dest.name, size.name, src_reg.name);
        let bytes =
            &RegisterWidth::to_le_bytes(src_reg.read() as RegisterWidth)[0..size.read() as usize];
        self.memory
            .write_bytes(dest.read() as RegisterWidth, bytes)?;
        Ok(OPCODE_BYTES + bytes_read)
    }
    fn op_add(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        dest.write(op1.read().wrapping_add(op2.read()))?;
        log_disassembly!("add {}, {}, {}", dest.name, op1.name, op2.name);
        Ok(OPCODE_BYTES + bytes_read)
    }
    fn op_sub(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        dest.write(op1.read().wrapping_sub(op2.read()))?;
        log_disassembly!("sub {}, {}, {}", dest.name, op1.name, op2.name);
        Ok(OPCODE_BYTES + bytes_read)
    }
    fn op_mult(&mut self) -> Result<usize, VMError> {
        let (dest, op1, op2, bytes_read) = self.trinary_operation_decode()?;
        dest.write(op1.read().wrapping_mul(op2.read()))?;
        log_disassembly!("mult {}, {}, {}", dest.name, op1.name, op2.name);
        Ok(OPCODE_BYTES + bytes_read)
    }

    fn op_div(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    // fn op_neg(&mut self) -> Result<usize, VMError> {
    //     todo!()
    // }
    fn op_or(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_xor(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_and(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_not(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_shl(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_shr(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_rotl(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_rotr(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_neg(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_jmp(&mut self) -> Result<usize, VMError> {
        let address = slice_to_usize(&self.read_operands(ADDRESS_BYTES)?);
        let pc = self.registers.get_mut_register(PROGRAM_COUNTER)?;
        log_disassembly!("jmp ${}", address);

        pc.write(address)?;
        Ok(0) // jmp moved pc so return 0 so it isnt moved again
    }
    fn op_jifz(&mut self) -> Result<usize, VMError> {
        let (pc, condition, address, bytes_read) = self.jif_decode()?;
        log_disassembly!("jifz {}, ${}", condition.name, address);
        if condition.read() == 0 {
            pc.write(address)?;
            Ok(0)
        } else {
            Ok(OPCODE_BYTES + bytes_read)
        }
    }
    fn op_jifnz(&mut self) -> Result<usize, VMError> {
        let (pc, condition, address, bytes_read) = self.jif_decode()?;
        log_disassembly!("jifnz {}, ${}", condition.name, address);
        if condition.read() != 0 {
            pc.write(address)?;
            Ok(0)
        } else {
            Ok(OPCODE_BYTES + bytes_read)
        }
    }
    fn op_pr(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_inc(&mut self) -> Result<usize, VMError> {
        let (dest, bytes_read) = self.unary_operation_decode()?;
        let old = dest.read();
        let new = old + 1;
        dest.write(new)?;
        // verbose_println!("{old}+1 = {new}||{}", dest.read());
        log_disassembly!("inc {}", dest.name);
        Ok(OPCODE_BYTES + bytes_read)
    }
    fn op_dec(&mut self) -> Result<usize, VMError> {
        let (dest, bytes_read) = self.unary_operation_decode()?;
        dest.write(dest.read().wrapping_sub(1))?;
        log_disassembly!("dec {}", dest.name);
        Ok(OPCODE_BYTES + bytes_read)
    }
    fn op_push(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_pop(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_call(&mut self) -> Result<usize, VMError> {
        todo!()
    }
    fn op_ret(&mut self) -> Result<usize, VMError> {
        todo!()
    }
}

/// input a slice from start of immediate to the end of the requested operands and it will return the immediate and how many bytes were read
fn read_immediate_from_operands_slice(slice: &[u8]) -> Result<(usize, usize), VMError> {
    let size = match slice.get(0) {
        Some(b) => *b as usize,
        None => {
            return Err(VMError::new(
                VMErrorCode::GenericError,
                format!(
                    "could not read size of immediate (operands slice length passed was {})",
                    slice.len()
                ),
            ))
        }
    };
    let immediate_bytes = match slice.get(1..size) {
        Some(bytes) => bytes,
        None => {
            return Err(VMError::new(
                VMErrorCode::GenericError,
                format!(
                "could not read immediate bytes from slice. (operands slice length passed was {})",
                slice.len()
            ),
            ))
        }
    };
    let immediate = slice_to_usize(immediate_bytes);
    Ok((immediate, size))
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
        let program_length = slice_to_usize(&header[head..head + HEADER_ENTRY_LENGTH]);
        head += HEADER_ENTRY_LENGTH;
        let ram_image_length = slice_to_usize(&header[head..head + HEADER_ENTRY_LENGTH]);
        head += HEADER_ENTRY_LENGTH;
        let entry_point_address = slice_to_usize(&header[head..head + HEADER_ENTRY_LENGTH]);
        head += HEADER_ENTRY_LENGTH;
        let debug_partition_length = slice_to_usize(&header[head..head + HEADER_ENTRY_LENGTH]);
        Ok((
            program_length,
            ram_image_length,
            entry_point_address,
            debug_partition_length,
        ))
    }
}

const HEADER_ENTRY_LENGTH: usize = 8;

pub fn slice_to_usize(slice: &[u8]) -> usize {
    let target_length = size_of::<usize>();
    let mut byte_buf: Vec<u8> = Vec::with_capacity(target_length);
    byte_buf.extend_from_slice(slice);
    byte_buf.resize(target_length, 0);
    let byte_array: [u8; size_of::<u64>()] = match byte_buf.try_into() {
        Ok(arr) => arr,
        Err(err) => panic!("failed to convert sequence despite padding :: {err:?}"),
    };
    usize::from_le_bytes(byte_array)
}

struct DebugTable;
impl DebugTable {
    fn new(partition: &[u8]) -> Self {
        verbose_println!("debug table not implemented yet");
        Self
    }
}
