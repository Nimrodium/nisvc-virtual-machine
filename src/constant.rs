pub const END_EXEC: u8 = 0xff;
pub const HALT_EXE: u8 = 0xfe;
pub const UNINITIALIZED_MEMORY: u8 = 0xfd;
pub const UNINITIALIZED_REGISTER: u64 = 0xfdfdfdfdfdfdfdfd;
pub const NAME: &str = "nisvc-system";
pub const PROGRAM_COUNTER: u8 = 1;
pub const STACK_POINTER: u8 = 2;
pub const FRAME_POINTER: u8 = 3;
pub const SIGNATURE: &[u8] = b"NISVC-EF";
pub const STACK_SIZE: u64 = 1000;
pub const DEFAULT_CLOCK_SPEED: u64 = 1000; // steps per second

pub const MEM_STATIC: u8 = 0;
pub const MEM_HEAP: u8 = 1;
pub const MEM_STACK: u8 = 2;
pub const MEM_INVALID: u8 = 3;
