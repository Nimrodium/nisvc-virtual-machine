pub const SIGNATURE: &[u8] = b"NISVC-EF";
pub const NAME: &str = "nisvc-system";
pub const RUNTIME_VER: &str = "0.4";
pub const SHELL_PROMPT: &str = ":: ~> ";
pub const CLOCK_SPEED_HZ: usize = 100;


pub type OpcodeSize = u8;
pub type RegisterWidth = u64;
pub type RegisterCode = u8;
pub type VMAddress = RegisterWidth;
pub const INIT_VALUE: RegisterWidth = 0xFF;
pub const REGISTER_BYTES: usize = 1;
pub const OPCODE_BYTES: usize = 1;
pub const ADDRESS_BYTES: usize = 8;

// MMIO MAP
pub const MMIO_ADDRESS_SPACE: usize = 42; // give the lowest 10 addresses to mmio

pub const KEYBOARD_MMIO_ADDRESS: RegisterWidth = 0x0;

// display has 40 pixels
pub const DISPLAY_MMIO_ADDRESS_START: RegisterWidth = 0x1; // inclusive
pub const DISPLAY_MMIO_ADDRESS_END: RegisterWidth = 0x29; // inclusive


pub const DEBUG_PRINT: bool = true;

pub const INIT_RAM_VALUE: u8 = 0xFF;
pub const RAM_SIZE: RegisterWidth = 1000;
pub const CLOCK_SPEED_MS: usize = 1 / CLOCK_SPEED_HZ * 1000; //milliseconds between clock cycle
