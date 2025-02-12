// cpu.rs
//

use std::{fs::File, io::Read, path::Path};

use crate::{
    constant::{self, OpcodeSize, RegisterWidth, INIT_VALUE},
    memory::Memory,
    opcode::Opcode,
};

pub struct GeneralPurposeRegisters {
    pub r1: RegisterWidth,
    pub r2: RegisterWidth,
    pub r3: RegisterWidth,
    pub r4: RegisterWidth,
    pub r5: RegisterWidth,
    pub r6: RegisterWidth,
    pub r7: RegisterWidth,
    pub r8: RegisterWidth,
    pub r9: RegisterWidth,
    pub r10: RegisterWidth,
    pub r11: RegisterWidth,
    pub r12: RegisterWidth,
    pub r13: RegisterWidth,
    pub r14: RegisterWidth,
    pub r15: RegisterWidth,
    pub r16: RegisterWidth,
    pub r17: RegisterWidth,
    pub r18: RegisterWidth,
    pub r19: RegisterWidth,
    pub r20: RegisterWidth,
}

pub struct SpecialPurposeRegisters {
    pub pc: RegisterWidth,
    pub sp: RegisterWidth,

    pub o1: RegisterWidth,
    pub o2: RegisterWidth,
    pub o3: RegisterWidth,
    pub o4: RegisterWidth,
    pub o5: RegisterWidth,
    pub o6: RegisterWidth,
    pub o7: RegisterWidth,
    pub o8: RegisterWidth,
    pub o9: RegisterWidth,
    pub o10: RegisterWidth,
}

impl GeneralPurposeRegisters {
    pub fn new() -> Self {
        GeneralPurposeRegisters {
            r1: INIT_VALUE,
            r2: INIT_VALUE,
            r3: INIT_VALUE,
            r4: INIT_VALUE,
            r5: INIT_VALUE,
            r6: INIT_VALUE,
            r7: INIT_VALUE,
            r8: INIT_VALUE,
            r9: INIT_VALUE,
            r10: INIT_VALUE,
            r11: INIT_VALUE,
            r12: INIT_VALUE,
            r13: INIT_VALUE,
            r14: INIT_VALUE,
            r15: INIT_VALUE,
            r16: INIT_VALUE,
            r17: INIT_VALUE,
            r18: INIT_VALUE,
            r19: INIT_VALUE,
            r20: INIT_VALUE,
        }
    }
}
impl SpecialPurposeRegisters {
    pub fn new() -> Self {
        SpecialPurposeRegisters {
            pc: INIT_VALUE,
            sp: INIT_VALUE,
            o1: INIT_VALUE,
            o2: INIT_VALUE,
            o3: INIT_VALUE,
            o4: INIT_VALUE,
            o5: INIT_VALUE,
            o6: INIT_VALUE,
            o7: INIT_VALUE,
            o8: INIT_VALUE,
            o9: INIT_VALUE,
            o10: INIT_VALUE,
        }
    }
}
pub enum State {
    NoProgramLoaded,
    ProgramLoadedNotStarted,
    ProgramRunning,
    ProgramFatalError,
    ProgramExitedSuccess,
    ProgramHalted,
}
pub struct Runtime {
    pub gpr: GeneralPurposeRegisters,
    pub spr: SpecialPurposeRegisters,
    pub memory: Memory,
    pub state: State,
    pub debug: bool,
}
/// init a new runtime with program loaded
impl Runtime {
    pub fn new(debug: bool) -> Result<Self, String> {
        Ok(Runtime {
            memory: Memory::new()?,
            gpr: GeneralPurposeRegisters::new(),
            spr: SpecialPurposeRegisters::new(),
            state: State::NoProgramLoaded,
            debug,
        })
    }
    /// returns mutable reference to a register
    fn get_mut_reg(&mut self, reg_bytes: Vec<u8>) -> Result<&mut RegisterWidth, String> {
        let mut bytes = reg_bytes.clone();
        let reg = bytes_to_register_width(&mut bytes)?;
        // println!("reg_code: {reg}");
        match reg {
            1 => Ok(&mut self.gpr.r1),
            2 => Ok(&mut self.gpr.r2),
            3 => Ok(&mut self.gpr.r3),
            4 => Ok(&mut self.gpr.r4),
            5 => Ok(&mut self.gpr.r5),
            6 => Ok(&mut self.gpr.r6),
            7 => Ok(&mut self.gpr.r7),
            8 => Ok(&mut self.gpr.r8),
            9 => Ok(&mut self.gpr.r9),
            10 => Ok(&mut self.gpr.r10),
            11 => Ok(&mut self.gpr.r11),
            12 => Ok(&mut self.gpr.r12),
            13 => Ok(&mut self.gpr.r13),
            14 => Ok(&mut self.gpr.r14),
            15 => Ok(&mut self.gpr.r15),
            16 => Ok(&mut self.gpr.r16),
            17 => Ok(&mut self.gpr.r17),
            18 => Ok(&mut self.gpr.r18),
            19 => Ok(&mut self.gpr.r19),
            20 => Ok(&mut self.gpr.r20),
            21 => Ok(&mut self.spr.pc),
            22 => Ok(&mut self.spr.sp),
            23 => Ok(&mut self.spr.o1),
            24 => Ok(&mut self.spr.o2),
            25 => Ok(&mut self.spr.o3),
            26 => Ok(&mut self.spr.o4),
            27 => Ok(&mut self.spr.o5),
            28 => Ok(&mut self.spr.o6),
            29 => Ok(&mut self.spr.o7),
            30 => Ok(&mut self.spr.o8),
            31 => Ok(&mut self.spr.o9),
            32 => Ok(&mut self.spr.o10),
            _ => Err(format!("invalid register code [{reg:#x?}]")),
        }
    }
    /// returns the value inside register
    fn get_reg(&self, reg_bytes: Vec<u8>) -> Result<RegisterWidth, String> {
        let mut bytes = reg_bytes.clone();
        let reg = bytes_to_register_width(&mut bytes)?;
        // println!("reg_code: {reg}");
        match reg {
            1 => Ok(self.gpr.r1),
            2 => Ok(self.gpr.r2),
            3 => Ok(self.gpr.r3),
            4 => Ok(self.gpr.r4),
            5 => Ok(self.gpr.r5),
            6 => Ok(self.gpr.r6),
            7 => Ok(self.gpr.r7),
            8 => Ok(self.gpr.r8),
            9 => Ok(self.gpr.r9),
            10 => Ok(self.gpr.r10),
            11 => Ok(self.gpr.r11),
            12 => Ok(self.gpr.r12),
            13 => Ok(self.gpr.r13),
            14 => Ok(self.gpr.r14),
            15 => Ok(self.gpr.r15),
            16 => Ok(self.gpr.r16),
            17 => Ok(self.gpr.r17),
            18 => Ok(self.gpr.r18),
            19 => Ok(self.gpr.r19),
            20 => Ok(self.gpr.r20),
            21 => Ok(self.spr.pc),
            22 => Ok(self.spr.sp),
            23 => Ok(self.spr.o1),
            24 => Ok(self.spr.o2),
            25 => Ok(self.spr.o3),
            26 => Ok(self.spr.o4),
            27 => Ok(self.spr.o5),
            28 => Ok(self.spr.o6),
            29 => Ok(self.spr.o7),
            30 => Ok(self.spr.o8),
            31 => Ok(self.spr.o9),
            32 => Ok(self.spr.o10),
            _ => Err(format!("invalid register code [{reg:#x?}]")),
        }
    }
    /// execute runtime at PC
    pub fn exec(&mut self) -> Result<(), String> {
        // program loop
        match self.state {
            State::NoProgramLoaded => return Err("cannot start execution, no program loaded".to_string()),
            State::ProgramFatalError => return Err("cannot start execution, program suffered fatal error, further execution is unpredictable".to_string()),
            State::ProgramExitedSuccess => return Err("cannot start execution, program finished execution".to_string()),
            State::ProgramRunning => return Err("cannot start execution, program already running. how did you get here?!?!".to_string()),

            _ => {
                println!("executing...");
            },
        };
        self.state = State::ProgramRunning;
        let clock_sleep = std::time::Duration::from_millis(constant::CLOCK_SPEED_MS);
        loop {
            match self.state {
                State::ProgramRunning => (),
                _ => break,
            }
            match self.step() {
                Ok(()) => (),
                Err(why) => {
                    let error = format!("error in execution :: {}", why);
                    self.state = State::ProgramFatalError;
                    return Err(error);
                }
            }
            std::thread::sleep(clock_sleep);
        }
        Ok(())
    }
    /// step through one cycle
    pub fn step(&mut self) -> Result<(), String> {
        let opcode = self.decode_opcode()?;
        if constant::DEBUG_PRINT {
            println!("{opcode:?}");
        }
        let operation_result = match opcode {
            Opcode::Nop => self.nop(),

            Opcode::Mov => self.op_mov(),
            Opcode::Movim => self.op_movim(),

            Opcode::Load => self.op_load(),
            Opcode::Store => self.op_store(),

            Opcode::Add => self.op_add(),
            Opcode::Sub => self.op_sub(),
            Opcode::Mult => self.op_mult(),
            Opcode::Div => self.op_div(),
            Opcode::Neg => self.op_neg(),

            Opcode::Or => self.op_or(),
            Opcode::Xor => self.op_xor(),
            Opcode::And => self.op_and(),
            Opcode::Not => self.op_not(),
            Opcode::Shl => self.op_shl(),
            Opcode::Shr => self.op_shr(),
            Opcode::Rotl => self.op_rotl(),
            Opcode::Rotr => self.op_rotr(),

            Opcode::End_of_exec_section => self.op_end_of_exec_section(),
            Opcode::Jmp => self.op_jmp(),
            Opcode::Jifz => self.op_jifz(),
            Opcode::Jifnz => self.op_jifnz(),
            Opcode::Pr => self.op_pr(),

            Opcode::Inc => self.op_inc(),
            Opcode::Dec => self.op_dec(),

            Opcode::Push => self.op_push(),
            Opcode::Pop => self.op_pop(),

            Opcode::Call => self.op_call(),
            Opcode::Ret => self.op_ret(),
        };
        match operation_result {
            Ok(increment) => self.spr.pc += increment as RegisterWidth,
            Err(runtime_error) => return Err(format!("runtime error :: {}", runtime_error)),
        }
        Ok(())
    }
    pub fn load(&mut self, binary: &Path) -> Result<(), String> {
        // verify signature
        // locate start and end of exec
        // mark start of execution section
        //

        let mut binary_file: File = match File::open(binary) {
            Ok(file) => file,
            Err(why) => return Err(why.to_string()),
        };

        let mut program_signature_buffer = vec![0; constant::SIGNATURE.len()];
        match binary_file.read_exact(&mut program_signature_buffer) {
            Ok(_) => (),
            Err(why) => {
                let error = format!("could not read signature :: {}", why);
                return Err(error);
            }
        };

        let program_signature = match String::from_utf8(program_signature_buffer) {
            Ok(string) => string,
            Err(why) => {
                let error = format!("could not convert signature to string :: {}", why);
                return Err(error);
            }
        };

        if constant::SIGNATURE != program_signature {
            let why = format!(
                "exec format error: signature not valid, {} != {}",
                constant::SIGNATURE,
                program_signature
            );
            return Err(why);
        } else {
            println!("valid exec format");
        }

        // signature verified -- gonna move this to after loading into memory

        let mut binary_image: Vec<u8> = vec![];
        match binary_file.read_to_end(&mut binary_image) {
            Ok(_) => (),
            Err(why) => {
                let error = format!("failed to read file into rom :: {}", why);
                return Err(error);
            }
        };
        let header_len = size_of::<RegisterWidth>() * 2;
        if binary_image.len() < header_len {
            return Err(format!(
                "{} formatted file has incomplete header {} bytes, expected {} bytes ",
                constant::SIGNATURE,
                binary_image.len(),
                header_len,
            ));
        }

        // image loaded into memory and verified to have a full header

        let mut head = 0;
        // read header data -- VVV --
        //
        // read data length
        // first u64 after the signature is size of data section in bytes

        let program_length = RegisterWidth::from_le_bytes(
            match &binary_image[head..head + size_of::<RegisterWidth>()].try_into() {
                Ok(array) => *array,
                Err(why) => {
                    let error = format!("failed to read program length :: {}", why);
                    return Err(error);
                }
            },
        ) as usize;
        head += size_of::<RegisterWidth>(); // pass the datarom length
                                            // read exec length
                                            // next 8 bytes after datarom length

        let data_rom_length = RegisterWidth::from_le_bytes(
            match &binary_image[head..head + size_of::<RegisterWidth>()].try_into() {
                Ok(array) => *array,
                Err(why) => {
                    let error = format!("failed to read ram_image length :: {}", why);
                    return Err(error);
                }
            },
        ) as usize;
        head += size_of::<RegisterWidth>();
        // data image and program image length u64s read successfully

        self.memory.program = binary_image[head..head + program_length].to_vec();

        let ram_image =
            &binary_image[head + program_length..head + program_length + data_rom_length];
        self.memory.ram_base = (constant::MMIO_ADDRESS_SPACE + program_length) as RegisterWidth; // program/ram address boundary
        self.memory.flash_ram(ram_image)?;
        self.spr.pc = self.memory.program_base;
        self.state = State::ProgramLoadedNotStarted;
        if constant::DEBUG_PRINT {
            println!("MMIO size = {}", constant::MMIO_ADDRESS_SPACE);
            println!("data_length/initram_size = {}", data_rom_length);
            println!("program_length = {}", program_length);
            println!("program_real_length = {}", self.memory.program.len());
            println!("rom_base = {:#x?}", self.memory.program_base);
            println!("ram_base = {:#x?}", self.memory.ram_base);
        }
        Ok(())
    }

    fn decode_opcode(&mut self) -> Result<Opcode, String> {
        let opcode_bytes = self
            .memory
            .read_bytes(self.spr.pc, constant::OPCODE_BYTES)?;
        let opcode_code = OpcodeSize::from_le_bytes(match opcode_bytes.try_into() {
            Ok(array) => array,
            Err(why) => {
                let error = format!("failed to read opcode :: {:?}", why);
                return Err(error);
            }
        });
        match opcode_code.try_into() {
            Ok(opcode) => {
                // println!("decoded {opcode:?}");
                Ok(opcode)
            }
            Err(()) => return Err(format!("opcode {:#x?} not recognized", opcode_code)),
        }
    }
    fn trinary_operand_decode(
        &mut self,
    ) -> Result<(&mut RegisterWidth, RegisterWidth, RegisterWidth, usize), String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES * 3;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let src1 = self.get_reg(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES * 2]
                .to_vec(),
        )?;
        let src2 = self.get_reg(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES * 2
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES * 3]
                .to_vec(),
        )?; // 2+(2*2)..2+(2*3) 6..9
        let dest = self.get_mut_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        Ok((dest, src1, src2, bytes_read))
    }
    fn extract_first_operand_as_register_mut(
        &mut self,
        bytes: &Vec<u8>,
    ) -> Result<&mut RegisterWidth, String> {
        let first_register = self.get_mut_reg(
            bytes[constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        Ok(first_register)
    }
    fn binary_operand_decode(
        &mut self,
    ) -> Result<(&mut RegisterWidth, RegisterWidth, usize), String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES * 2;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let src = self.get_reg(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES * 2]
                .to_vec(),
        )?;
        let dest = self.extract_first_operand_as_register_mut(&operand_bytes)?;
        Ok((dest, src, bytes_read))
    }
}
/// converts a vector of bytes into a u64 and pads if theres not enough errors if too many bytes are passed
fn bytes_to_register_width(bytes: &mut Vec<u8>) -> Result<RegisterWidth, String> {
    if bytes.len() > size_of::<RegisterWidth>() {
        return Err(format!(
            "too many bytes to pack into a {} byte integer",
            size_of::<RegisterWidth>()
        ));
    }
    bytes.resize(size_of::<RegisterWidth>(), 0x0);
    let bytes_array: [u8; size_of::<RegisterWidth>()] = match bytes.as_slice().try_into() {
        Ok(arr) => arr,
        Err(why) => {
            return Err(format!(
                "error building {} byte integer from bytes :: {why:?}",
                size_of::<RegisterWidth>()
            ))
        }
    };
    Ok(RegisterWidth::from_le_bytes(bytes_array))
}
// return of all instructions are Ok(increment program counter),Err(instruction Error)
impl Runtime {
    fn nop(&self) -> Result<usize, String> {
        // println!("nop");

        Ok(constant::OPCODE_BYTES)
    }

    fn op_mov(&mut self) -> Result<usize, String> {
        // let operand_bytes = self.fetch_operand_bytes(constant::REGISTER_BYTES * 2)?;
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES * 2;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;

        let src_reg = self.get_reg(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES * 2]
                .to_vec(),
        )?;
        println!("src_reg: {src_reg:#x?}");
        let dest_reg = self.get_mut_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        println!("dest_reg: {dest_reg:#x?}");
        *dest_reg = src_reg;

        Ok(bytes_read)
    }
    // movim dest_reg,imm (assembler places a byte before imm to indicate its size)
    fn op_movim(&mut self) -> Result<usize, String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES + 1;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let size: usize = *operand_bytes
            .get(constant::OPCODE_BYTES + constant::REGISTER_BYTES)
            .ok_or("could not read immediate size")? as usize;
        if size > size_of::<RegisterWidth>() || size == 0 {
            return Err(format!(
                "immediate is too large to load into register :: {}",
                size
            ));
        }
        let mut immediate = self.memory.read_bytes(
            self.spr.pc
                + constant::OPCODE_BYTES as RegisterWidth
                + constant::REGISTER_BYTES as RegisterWidth
                + 1,
            size,
        )?;
        let immediate_reg_width = bytes_to_register_width(&mut immediate)?;
        let dest_bytes = operand_bytes
            [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
            .to_vec();
        let dest_reg = self.get_mut_reg(dest_bytes)?;
        *dest_reg = immediate_reg_width;

        println!("movim instruction:\n\tdest_regid: {dest_reg}\n\tsize: {size}\n\timmediate: {immediate_reg_width}");
        Ok(bytes_read + size)
    }
    /// `load r1,r2,buffer`
    /// - r1 -- dest register
    /// - r2 -- size to read in (up to 8 bytes)
    /// - buffer -- start of memory address range
    fn op_load(&mut self) -> Result<usize, String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES * 3; // + constant::ADDRESS_BYTES; // reg,reg,addr
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let address: RegisterWidth = self.get_reg(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES * 2
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES * 3]
                .to_vec(), // 4..12
        )?;
        let size = self.get_reg(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES * 2]
                .to_vec(), // 2..4
        )? as usize;
        if size > size_of::<RegisterWidth>() {
            return Err(format!(
                "requested bytes are too large to read into register :: {size} bytes cannot fit into an {} byte register",size_of::<RegisterWidth>()
            ));
        }
        let dereferenced_value =
            bytes_to_register_width(&mut self.memory.read_bytes(address, size)?)?;
        let dest_reg = self.get_mut_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?; // 0..2

        *dest_reg = dereferenced_value;
        Ok(bytes_read)
    }
    fn op_store(&mut self) -> Result<usize, String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES * 3;
        // addr,reg,reg
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;

        let address = self.get_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        let size = self.get_reg(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        let src = self.get_reg(
            operand_bytes[constant::OPCODE_BYTES
                + constant::REGISTER_BYTES
                + constant::REGISTER_BYTES
                ..constant::OPCODE_BYTES
                    + constant::REGISTER_BYTES
                    + constant::REGISTER_BYTES
                    + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        if size as usize > size_of::<RegisterWidth>() {
            return Err(format!(
                "requested bytes are too large to read into register :: {size} bytes cannot fit into an {} byte register",size_of::<RegisterWidth>()
            ));
        }
        let src_bytes = &RegisterWidth::to_le_bytes(src)[0..size as usize]; // need to then trunicate src_bytes by size
        self.memory.write_bytes(address, src_bytes)?;
        Ok(bytes_read)
    }

    fn op_add(&mut self) -> Result<usize, String> {
        let (sum, addend1, addend2, bytes_read) = self.trinary_operand_decode()?;
        // *sum = addend1 + addend2;
        *sum = addend1.wrapping_add(addend2);
        Ok(bytes_read)
    }

    fn op_sub(&mut self) -> Result<usize, String> {
        let (difference, minuend, subtrahend, bytes_read) = self.trinary_operand_decode()?;
        *difference = minuend.wrapping_sub(subtrahend);
        Ok(bytes_read)
    }

    fn op_mult(&mut self) -> Result<usize, String> {
        let (result, multiplier, multiplicland, bytes_read) = self.trinary_operand_decode()?;
        *result = multiplier * multiplicland;
        *result = multiplier.wrapping_mul(multiplicland);
        Ok(bytes_read)
    }
    /// div quotient,remainder,dividend,divisor
    fn op_div(&mut self) -> Result<usize, String> {
        // let (quotient, dividend, divisor, bytes_read) = self.dest_src1_src2_format_decode()?;
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES * 4;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;

        let dividend = self.get_reg(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES * 2
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES * 3]
                .to_vec(),
        )?; // 2+(2*2)..2+(2*3) 6..9
        let divisor = self.get_reg(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES * 3
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES * 4]
                .to_vec(),
        )?;
        let quotient = self.get_mut_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        *quotient = dividend.wrapping_div(divisor);

        let remainder = self.get_mut_reg(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES * 2]
                .to_vec(),
        )?;

        *remainder = dividend.wrapping_rem(divisor);

        Ok(bytes_read)
    }
    fn op_neg(&mut self) -> Result<usize, String> {
        let (dest, src, bytes_read) = self.binary_operand_decode()?;
        *dest = src.wrapping_neg();
        Ok(bytes_read)
    }
    /// or dest(reg),src1(reg),src2(reg)
    fn op_or(&mut self) -> Result<usize, String> {
        let (result, byte1, byte2, bytes_read) = self.trinary_operand_decode()?;
        *result = byte1 | byte2;
        Ok(bytes_read)
    }
    /// xor dest(reg),src1(reg),src2(reg)
    fn op_xor(&mut self) -> Result<usize, String> {
        let (result, byte1, byte2, bytes_read) = self.trinary_operand_decode()?;
        *result = byte1 ^ byte2;
        Ok(bytes_read)
    }
    /// and dest(reg),src1(reg),src2(reg)
    fn op_and(&mut self) -> Result<usize, String> {
        let (result, byte1, byte2, bytes_read) = self.trinary_operand_decode()?;
        *result = byte1 & byte2;
        Ok(bytes_read)
    }
    /// not dest(reg),src(reg)
    fn op_not(&mut self) -> Result<usize, String> {
        let (result, byte, bytes_read) = self.binary_operand_decode()?;
        *result = !byte;
        Ok(bytes_read)
    }
    /// shl dest(reg),src(reg),amount(reg)
    fn op_shl(&mut self) -> Result<usize, String> {
        let (result, byte1, amount, bytes_read) = self.trinary_operand_decode()?;
        *result = byte1.wrapping_shl(amount as u32);
        Ok(bytes_read)
    }
    /// shr dest(reg),src(reg),amount(reg)
    fn op_shr(&mut self) -> Result<usize, String> {
        let (result, byte1, amount, bytes_read) = self.trinary_operand_decode()?;
        *result = byte1.wrapping_shr(amount as u32);
        Ok(bytes_read)
    }
    /// rotl dest(reg),src(reg),amount(reg)
    fn op_rotl(&mut self) -> Result<usize, String> {
        let (result, byte1, amount, bytes_read) = self.trinary_operand_decode()?;
        *result = byte1.rotate_left(amount as u32);
        Ok(bytes_read)
    }
    /// rotr dest(reg),src(reg),amount(reg)
    fn op_rotr(&mut self) -> Result<usize, String> {
        let (result, byte1, amount, bytes_read) = self.trinary_operand_decode()?;
        *result = byte1.rotate_right(amount as u32);
        Ok(bytes_read)
    }
    /// jmp address
    fn op_jmp(&mut self) -> Result<usize, String> {
        let bytes_read = constant::OPCODE_BYTES + constant::ADDRESS_BYTES;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;

        let address: RegisterWidth = Memory::address_from_bytes(
            operand_bytes[constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::ADDRESS_BYTES]
                .to_vec(), // 4..12
        )?;

        self.spr.pc = address;
        // println!("jmp address : {address:#x}\npc {:#x}", self.spr.pc);
        Ok(0) // returns zero because we modified the program counter
    }

    /// jifz condition address
    fn op_jifz(&mut self) -> Result<usize, String> {
        let bytes_read =
            constant::OPCODE_BYTES + constant::REGISTER_BYTES + constant::ADDRESS_BYTES;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let condition = self.get_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        let address = Memory::address_from_bytes(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES + constant::ADDRESS_BYTES]
                .to_vec(),
        )?;
        let return_value;
        if condition == 0 {
            self.spr.pc = address;
            if constant::DEBUG_PRINT {
                println!("jifz condition is zero, jumping to {address}");
            }
            return_value = 0;
        } else {
            if constant::DEBUG_PRINT {
                println!("jifz condition not zero no action taken");
            }
            return_value = bytes_read;
        }
        Ok(return_value) // return zero if no action taken
    }
    /// jifnz condition address
    fn op_jifnz(&mut self) -> Result<usize, String> {
        let bytes_read =
            constant::OPCODE_BYTES + constant::REGISTER_BYTES + constant::ADDRESS_BYTES;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let condition = self.get_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        let address = Memory::address_from_bytes(
            operand_bytes[constant::OPCODE_BYTES + constant::REGISTER_BYTES
                ..constant::OPCODE_BYTES + constant::REGISTER_BYTES + constant::ADDRESS_BYTES]
                .to_vec(),
        )?;
        let return_value;
        if condition != 0 {
            self.spr.pc = address;
            if constant::DEBUG_PRINT {
                println!("jifnz condition is not zero, jumping to {address:#x}");
            }
            return_value = 0;
        } else {
            if constant::DEBUG_PRINT {
                println!("jifnz condition is zero no action taken");
            }
            return_value = bytes_read;
        }
        Ok(return_value) // return zero if no action taken
    }
    // pr src(reg)
    fn op_pr(&mut self) -> Result<usize, String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let register = self.get_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        println!("pr out: {register}");
        Ok(bytes_read)
    }
    fn op_end_of_exec_section(&mut self) -> Result<usize, String> {
        if constant::DEBUG_PRINT {
            println!("end_of_exec_section");
        }
        self.state = State::ProgramExitedSuccess;
        Ok(0)
    }

    fn op_inc(&mut self) -> Result<usize, String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let register = self.get_mut_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        *register += 1;
        Ok(bytes_read)
    }
    fn op_dec(&mut self) -> Result<usize, String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let register = self.get_mut_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        *register -= 1;
        Ok(bytes_read)
    }
    fn op_push(&mut self) -> Result<usize, String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let register = self.get_mut_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        todo!();
        Ok(bytes_read)
    }
    fn op_pop(&mut self) -> Result<usize, String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let register = self.get_mut_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        todo!();
        Ok(bytes_read)
    }
    fn op_call(&mut self) -> Result<usize, String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let register = self.get_mut_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        todo!();
        Ok(bytes_read)
    }
    fn op_ret(&mut self) -> Result<usize, String> {
        let bytes_read = constant::OPCODE_BYTES + constant::REGISTER_BYTES;
        let operand_bytes = self.memory.read_bytes(self.spr.pc, bytes_read)?;
        let register = self.get_mut_reg(
            operand_bytes
                [constant::OPCODE_BYTES..constant::OPCODE_BYTES + constant::REGISTER_BYTES]
                .to_vec(),
        )?;
        todo!();

        Ok(bytes_read)
    }
}
