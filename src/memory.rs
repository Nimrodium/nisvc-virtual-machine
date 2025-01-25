enum PoolType {
    Data,
    Program,
    Stack,
    Heap,
}
#[derive(Debug)]
enum DataType {
    // unsigned
    Unsigned8,
    Unsigned16,
    Unsigned32,

    // signed
    Signed8,
    Signed16,
    Signed32,

    // complex
    Array,
    String,
}

struct Pool {
    memory: Vec<u8>,
}
pub struct Memory {
    data_rom: Pool,
    program_rom: Pool,
    stack_ram: Pool,
}

impl Memory {
    /// read an address from memory and return its value
    /// ONLY for literals up to 4 bytes.
    /// to read arrays use `Memory::read_array()`
    /// returns Result
    /// Ok -> dereferenced value
    /// Err -> Memory Address does not exist MemoryAccessViolation
    fn read(address: MemoryAddress) -> Result<usize, String> {
        todo!()
    }

    /// write to an address at memory and return Err on write failure
    /// write may fail due to attempting to write to program_rom or data_rom,
    /// or memory address does not exist,
    /// ONLY for literal addresses, use Memory::write_array to write arrays
    fn write(address: MemoryAddress) -> Result<(), String> {
        todo!()
    }
    /// reads an index from an array address
    fn read_array_index(address: MemoryAddress, index: usize) -> Result<usize, String> {
        todo!()
    }
    /// copies and dereferences an array to heap
    fn load_array() -> Result<(), String> {
        todo!()
    }
}

struct MemoryAddress {
    // metadata byte
    pool: PoolType,      // 2 bits
    is_absolute: bool,   //1 bit
    is_pointer: bool,    // 1 bit
    data_type: DataType, // 4 bits

    address: usize,
}

impl MemoryAddress {
    /// build `MemoryAddress` object from serialized byte representation
    /// takes in the address value (from program_rom) and `Memory` and decodes the rest of the address into an object
    fn from_bytes(address: &mut Vec<u8>, memory: Memory) -> Result<MemoryAddress, String> {
        // use bit operations to extract data
        // remove metadata byte
        let metadata = address.remove(0);
        if address.len() != 4 {
            return Err("address is not a valid size".to_string());
        }
        let address_bytes: [u8; 4] = match address.as_slice().try_into() {
            Ok(b) => b,
            Err(why) => return Err(format!("address is not a 32 bit value. {:?}", why).to_string()),
        };
        // let address_bytes: [u8; 4] = [address[0], address[1], address[2], address[3]];
        let address: u32 = u32::from_le_bytes(address_bytes);
        todo!()
    }
}

pub struct Operands {
    operands: Vec<MemoryAddress>,
}

impl Operands {
    /// builds operands object from operands vector
    fn new(bytes: Vec<u8>) -> Operands {
        todo!()
    }
    /// dereference literal address
    fn dereference(operand_idx: usize) -> usize {
        todo!()
    }
    /// writes to an operands address
    fn write(operand_idx: usize) {
        todo!()
    }
}
