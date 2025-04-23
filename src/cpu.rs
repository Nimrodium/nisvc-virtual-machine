use core::fmt;

use crate::{
    constant::{PROGRAM_COUNTER, UNINITIALIZED_REGISTER},
    log_input, log_output,
    memory::Memory,
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
pub struct CPU {
    registers: CPURegisters,
    memory: Memory,
}

impl CPU {
    fn fetch(&mut self) -> Result<(), ExecutionError> {
        let mut pc = self.registers.get_register(PROGRAM_COUNTER)?;
        let opcode = pc.read();

        Ok(())
    }
    fn decode(&mut self, encoding: &[Kind]) -> Result<DecodedInstruction, ExecutionError> {
        let mut immutable_registers: Vec<Register> = Vec::new();
        let mut mutable_registers: Vec<u8> = Vec::new();
        let mut addresses: Vec<u64> = Vec::new();
        let mut immediates: Vec<u64> = Vec::new();
        let mut pc = self.registers.get_register(PROGRAM_COUNTER)?.read();
        for kind in encoding {
            match kind {
                Kind::Register => immutable_registers.push({
                    let code = self.memory.read(pc)?;
                    pc += 1;
                    self.registers.get_register(code)?.clone()
                }),
                Kind::MutableRegister => mutable_registers.push({
                    let code = self.memory.read(pc)?;
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

    fn execute(&mut self) -> Result<(), ExecutionError> {
        todo!()
    }

    pub fn step(&mut self) -> Result<(), ExecutionError> {
        todo!()
    }

    pub fn exec_loop(&mut self) -> Result<(), ExecutionError> {
        todo!()
    }
}

pub enum Kind {
    Register,
    MutableRegister,
    Immediate,
    Address,
}

pub struct DecodedInstruction {
    immutable_registers: Vec<Register>,
    mutable_registers: Vec<u8>,

    addresses: Vec<u64>,
    immediates: Vec<u64>,
}
// impl DecodedInstruction {
//     fn new(memory:&mut Memory,encoding:&[Kind]) -> Self {

//     }
// }
