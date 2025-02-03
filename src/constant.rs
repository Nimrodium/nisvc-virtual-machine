pub const SIGNATURE: &str = "NISVC-EF";
pub const RUNTIME_VER: &str = "0.3";
pub const SHELL_PROMPT: &str = ":: ~> ";

pub type OpcodeSize = u16;
pub type RegisterSize = u16;

pub const REGISTER_BYTES: usize = 2;
pub const OPCODE_BYTES: usize = 2;
pub const ADDRESS_BYTES: usize = 8;

pub const MMIO_ADDRESS_SPACE: usize = 10; // give the lowest 10 addresses to mmio
