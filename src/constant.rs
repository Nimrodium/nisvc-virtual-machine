pub const SIGNATURE: &[u8] = b"NISVC-EF";
pub const NAME: &str = "nisvc-system";
pub const RUNTIME_VER: &str = "0.4";
pub const SHELL_PROMPT: &str = ":: ~> ";

pub type OpcodeSize = u8;
pub type RegisterWidth = u64;
pub type RegisterCode = u8;
pub type VMAddress = RegisterWidth;
pub const INIT_VALUE: RegisterWidth = 0xFF;
pub const REGISTER_BYTES: usize = 1;
pub const OPCODE_BYTES: usize = 1;
pub const ADDRESS_BYTES: usize = 8;
pub const DEFAULT_CLOCK_SPEED: usize = 100;

// MMIO MAP
pub const MMIO_ADDRESS_SPACE: usize = 42; // give the lowest 10 addresses to mmio

pub const KEYBOARD_MMIO_ADDRESS: RegisterWidth = 0x0;

// display has 40 pixels
pub const DISPLAY_MMIO_ADDRESS_START: RegisterWidth = 0x1; // inclusive
pub const DISPLAY_MMIO_ADDRESS_END: RegisterWidth = 0x29; // inclusive

pub const DEBUG_PRINT: bool = true;

pub const INIT_RAM_VALUE: u8 = 0xFF;
pub const RAM_SIZE: RegisterWidth = 1000;

pub const GPR_COUNT: u8 = 15;
pub const GPR_START: u8 = 4;
pub const GPR_END: u8 = GPR_START + GPR_COUNT;
pub const PROGRAM_COUNTER: u8 = RNULL + 1;
pub const STACK_POINTER: u8 = PROGRAM_COUNTER + 1;
pub const REAL_STACK_POINTER: u8 = STACK_POINTER + 1;
pub const RNULL: u8 = 0;

// pub const CLOCK_SPEED_MS: usize = 1 / CLOCK_SPEED_HZ * 1000; //milliseconds between clock cycle
// pub const CLOCK_SPEED_MS: usize = (1000 / CLOCK_SPEED_HZ) as usize;
