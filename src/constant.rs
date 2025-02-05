pub const SIGNATURE: &str = "NISVC-EF";
pub const RUNTIME_VER: &str = "0.3";
pub const SHELL_PROMPT: &str = ":: ~> ";

pub type OpcodeSize = u8;
pub type RegisterSize = u8;

pub const REGISTER_BYTES: usize = 1;
pub const OPCODE_BYTES: usize = 1;
pub const ADDRESS_BYTES: usize = 8;

// MMIO MAP
pub const MMIO_ADDRESS_SPACE: usize = 42; // give the lowest 10 addresses to mmio

pub const KEYBOARD_MMIO_ADDRESS: u64 = 0x0;

// display has 40 pixels
pub const DISPLAY_MMIO_ADDRESS_START: u64 = 0x1; // inclusive
pub const DISPLAY_MMIO_ADDRESS_END: u64 = 0x29; // inclusive

pub const CLOCK_SPEED_MS: u64 = 0; //milliseconds between clock cycle
