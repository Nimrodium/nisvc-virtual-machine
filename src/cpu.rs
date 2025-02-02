// cpu.rs
//

use std::{fs::File, io::Read, path::Path};

use crate::{
    constant::{self, OpcodeSize},
    memory::{Memory, MemoryAddress, Pool},
    opcode::Opcode,
};

enum Data {
    GeneralRegisters,
    SpecialRegisters,
    Ram,
    Rom,
}
pub struct GeneralPurposeRegisters {
    r1: u64,
    r2: u64,
    r3: u64,
    r4: u64,
    r5: u64,
    r6: u64,
    r7: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    r16: u64,
    r17: u64,
    r18: u64,
    r19: u64,
    r20: u64,
}

pub struct SpecialPurposeRegisters {
    pub pc: u64,
    sp: u64,

    o1: u64,
    o2: u64,
    o3: u64,
    o4: u64,
    o5: u64,
    o6: u64,
    o7: u64,
    o8: u64,
    o9: u64,
    o10: u64,
}

impl GeneralPurposeRegisters {
    pub fn new() -> Self {
        GeneralPurposeRegisters {
            r1: 0xDEADBEEF,
            r2: 0xDEADBEEF,
            r3: 0xDEADBEEF,
            r4: 0xDEADBEEF,
            r5: 0xDEADBEEF,
            r6: 0xDEADBEEF,
            r7: 0xDEADBEEF,
            r8: 0xDEADBEEF,
            r9: 0xDEADBEEF,
            r10: 0xDEADBEEF,
            r11: 0xDEADBEEF,
            r12: 0xDEADBEEF,
            r13: 0xDEADBEEF,
            r14: 0xDEADBEEF,
            r15: 0xDEADBEEF,
            r16: 0xDEADBEEF,
            r17: 0xDEADBEEF,
            r18: 0xDEADBEEF,
            r19: 0xDEADBEEF,
            r20: 0xDEADBEEF,
        }
    }
}
impl SpecialPurposeRegisters {
    pub fn new() -> Self {
        SpecialPurposeRegisters {
            pc: 0xDEADBEEF,
            sp: 0xDEADBEEF,
            o1: 0xDEADBEEF,
            o2: 0xDEADBEEF,
            o3: 0xDEADBEEF,
            o4: 0xDEADBEEF,
            o5: 0xDEADBEEF,
            o6: 0xDEADBEEF,
            o7: 0xDEADBEEF,
            o8: 0xDEADBEEF,
            o9: 0xDEADBEEF,
            o10: 0xDEADBEEF,
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
    pub fn new(debug: bool) -> Self {
        Runtime {
            memory: Memory::new(),
            gpr: GeneralPurposeRegisters::new(),
            spr: SpecialPurposeRegisters::new(),
            state: State::NoProgramLoaded,
            debug,
        }
    }
    /// returns mutable reference to a register
    fn get_mut_reg(&mut self, reg: usize) -> Result<&mut u64, String> {
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
    fn get_reg(&self, reg: usize) -> Result<u64, String> {
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
        }
        Ok(())
    }
    /// step through one cycle
    pub fn step(&mut self) -> Result<(), String> {
        let opcode = self.decode_opcode()?;
        let operation_result = match opcode {
            Opcode::Nop => Inst::nop(self),
            Opcode::Mov => Inst::mov(self),
            Opcode::Load => Inst::load(self),
            Opcode::Store => Inst::store(self),
            Opcode::Add => Inst::add(self),
            Opcode::Sub => Inst::sub(self),
            Opcode::Mult => Inst::mult(self),
            Opcode::Div => Inst::div(self),
            Opcode::End_of_exec_section => Inst::end_of_exec_section(self),
        };
        match operation_result {
            Ok(increment) => self.spr.pc += increment as u64,
            Err(runtime_error) => return Err(format!("runtime error: {}", runtime_error)),
        }
        Ok(())
    }

    /// dumps state
    pub fn dump(&self, data: Data) -> String {
        todo!()
    }
    pub fn throw_runtime_error(self, why: &str) {
        println!("runtime error!! :: {}", why)
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
        let mut rom: Vec<u8> = vec![];
        match binary_file.read_to_end(&mut rom) {
            Ok(_) => (),
            Err(why) => {
                let error = format!("failed to read file into rom :: {}", why);
                return Err(error);
            }
        };
        let header_len = constant::SIGNATURE.len() + (8 * 2);
        if rom.len() <= header_len {
            return Err(format!(
                "{} formatted file has incomplete header",
                constant::SIGNATURE,
            ));
        }
        let mut head = 0;
        // read header data -- VVV --
        //
        // read data length
        // first u64 after the signature is size of data section in bytes

        let data_rom_length = u64::from_le_bytes(match &rom[head..head + 8].try_into() {
            Ok(array) => *array,
            Err(why) => {
                let error = format!("failed to read datarom length :: {}", why);
                return Err(error);
            }
        });
        println!("data_rom_length = {}", data_rom_length);
        head += 8; // pass the datarom length
                   // read exec length
                   // next 8 bytes after datarom length

        let exec_rom_length = u64::from_le_bytes(match &rom[head..head + 8].try_into() {
            Ok(array) => *array,
            Err(why) => {
                let error = format!("failed to read execrom length :: {}", why);
                return Err(error);
            }
        });
        println!("exec_rom_length = {}", exec_rom_length);
        head += 8;

        self.memory.start_of_exec = head + data_rom_length as usize;
        println!("start_of_exec = {:#x?}", self.memory.start_of_exec);

        self.memory.end_of_exec = head + exec_rom_length as usize;
        println!("end_of_exec = {:#x?}", self.memory.end_of_exec);
        self.memory.rom = rom;

        self.spr.pc = self.memory.start_of_exec as u64;
        self.state = State::ProgramLoadedNotStarted;
        Ok(())
    }

    fn decode_opcode(&self) -> Result<Opcode, String> {
        let opcode_bytes = self.memory.byte_slice(
            &MemoryAddress {
                pool: Pool::Rom,
                address: self.spr.pc,
            },
            size_of::<OpcodeSize>(),
        )?;
        let opcode_code = OpcodeSize::from_le_bytes(match opcode_bytes.try_into() {
            Ok(array) => array,
            Err(why) => {
                let error = format!("failed to read datarom length :: {}", why);
                return Err(error);
            }
        });
        match opcode_code.try_into() {
            Ok(opcode) => Ok(opcode),
            Err(()) => return Err(format!("opcode {:#x?} not recognized", opcode_code)),
        }
    }
    // .0 = operands
    // .1 = read
    // Err
    fn decode_operands(&self, expected: usize) -> Result<(Vec<u64>, usize), String> {
        let read_bytes = 8;
        let mut operands: Vec<u64> = Vec::with_capacity(expected);
        let mut address = MemoryAddress {
            pool: Pool::Rom,
            address: self.spr.pc,
        };
        for _ in 0..expected {
            // let operand_u8: &[u8] = ;
            let operand_u64: u64 = u64::from_le_bytes(
                match self.memory.byte_slice(&address, read_bytes)?.try_into() {
                    Ok(array) => array,
                    Err(why) => {
                        let error = format!("failed to read operand :: {}", why);
                        return Err(error);
                    }
                },
            );
            operands.push(operand_u64);
            address.address += read_bytes as u64;
        }
        Ok((operands, (address.address - self.spr.pc) as usize))
    }
}
fn inc_pc(bytes: usize) -> usize {
    let inc = size_of::<OpcodeSize>();
    inc + bytes
}
struct Inst;
// return of all instructions are Ok(increment program counter),Err(instruction Error)
impl Inst {
    fn nop(runtime: &mut Runtime) -> Result<usize, String> {
        println!("nop");

        Ok(inc_pc(0))
    }
    fn mov(runtime: &mut Runtime) -> Result<usize, String> {
        println!("mov");
        let operands = runtime.decode_operands(2)?;

        let src = runtime.get_reg(*operands.0.get(1).ok_or("failed to read dest reg")? as usize)?;
        let dest =
            runtime.get_mut_reg(*operands.0.get(0).ok_or("failed to read dest reg")? as usize)?;

        *dest = src;
        Ok(inc_pc(operands.1))
    }
    fn movi(runtime: &mut Runtime) -> Result<usize, String> {
        println!("movi");
        let operands = runtime.decode_operands(2)?;
        let dest =
            runtime.get_mut_reg(*operands.0.get(0).ok_or("failed to read dest reg")? as usize)?;
        *dest = *operands.0.get(1).ok_or("failed to read src imm")? as u64;
        Ok(inc_pc(operands.1))
    }
    fn load(runtime: &mut Runtime) -> Result<usize, String> {
        println!("load");

        let operands = runtime.decode_operands(2)?;
        let dest =
            runtime.get_mut_reg(*operands.0.get(0).ok_or("failed to read dest reg")? as usize)?;
        // let src = runtime
        Ok(inc_pc(operands.1))
    }
    fn store(runtime: &mut Runtime) -> Result<usize, String> {
        println!("store");
        Ok(inc_pc(0))
    }
    fn add(runtime: &mut Runtime) -> Result<usize, String> {
        println!("add");
        let operands = runtime.decode_operands(3)?;

        let addend1 =
            runtime.get_reg(*operands.0.get(1).ok_or("failed to read dest reg")? as usize)?;
        let addend2 =
            runtime.get_reg(*operands.0.get(2).ok_or("failed to read dest reg")? as usize)?;
        let sum =
            runtime.get_mut_reg(*operands.0.get(0).ok_or("failed to read dest reg")? as usize)?;
        *sum = addend1 + addend2;
        Ok(inc_pc(operands.1))
    }
    fn sub(runtime: &mut Runtime) -> Result<usize, String> {
        println!("sub");
        let operands = runtime.decode_operands(3)?;

        let minuend =
            runtime.get_reg(*operands.0.get(1).ok_or("failed to read dest reg")? as usize)?;
        let subtrahend =
            runtime.get_reg(*operands.0.get(2).ok_or("failed to read dest reg")? as usize)?;
        let difference =
            runtime.get_mut_reg(*operands.0.get(0).ok_or("failed to read dest reg")? as usize)?;
        *difference = minuend - subtrahend;
        Ok(inc_pc(operands.1))
    }
    fn mult(runtime: &mut Runtime) -> Result<usize, String> {
        println!("mult");
        let operands = runtime.decode_operands(3)?;

        let multiplicand =
            runtime.get_reg(*operands.0.get(1).ok_or("failed to read dest reg")? as usize)?;
        let multipler =
            runtime.get_reg(*operands.0.get(2).ok_or("failed to read dest reg")? as usize)?;
        let result =
            runtime.get_mut_reg(*operands.0.get(0).ok_or("failed to read dest reg")? as usize)?;
        *result = multiplicand * multipler;
        Ok(inc_pc(operands.1))
    }
    fn div(runtime: &mut Runtime) -> Result<usize, String> {
        println!("div");
        let operands = runtime.decode_operands(4)?;

        let dividend =
            runtime.get_reg(*operands.0.get(2).ok_or("failed to read dest reg")? as usize)?;
        let divisor =
            runtime.get_reg(*operands.0.get(3).ok_or("failed to read dest reg")? as usize)?;

        let qotient =
            runtime.get_mut_reg(*operands.0.get(0).ok_or("failed to read dest reg")? as usize)?;
        *qotient = dividend / divisor;

        let remainder =
            runtime.get_mut_reg(*operands.0.get(1).ok_or("failed to read dest reg")? as usize)?;
        *remainder = dividend % divisor;

        Ok(inc_pc(operands.1))
    }
    fn end_of_exec_section(runtime: &mut Runtime) -> Result<usize, String> {
        println!("end_of_exec_section");
        runtime.state = State::ProgramExitedSuccess;
        Ok(0)
    }
}
