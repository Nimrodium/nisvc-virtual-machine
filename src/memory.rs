use crate::{constant::UNINITIALIZED_MEMORY, ExecutionError};
const HPA_NODE_DATA_OFFSET: u64 = 9;
const HPA_TAIL_SENTINEL_ADDRESS: u64 = 0;
pub type nisvc_ptr = u64;
pub struct Memory {
    physical: Vec<u8>,
    max: u64,
    hpa_head_ptr: u64,
    stack_start: u64,
    total_heap_allocations: u64,
}

impl Memory {
    pub fn new(max: u64) -> Self {
        Self {
            physical: Vec::with_capacity(max as usize),
            max,
            hpa_head_ptr: 0,
            stack_start: 0,
            total_heap_allocations: 0,
        }
    }
    pub fn load(&mut self, image: Vec<u8>, stack_size: u64) -> Result<(), ExecutionError> {
        let img_len = image.len() as u64;
        let undefined_bytes = self.max - img_len;
        self.hpa_head_ptr = img_len + HPA_NODE_DATA_OFFSET;
        self.stack_start = self.max - stack_size;
        if self.stack_start < img_len {
            return Err(ExecutionError::new(format!(
                "stack allocation overlaps with program allocation by {} bytes",
                img_len - self.stack_start
            )));
        }
        self.physical.extend(image);
        self.physical
            .extend(std::iter::repeat(UNINITIALIZED_MEMORY).take(undefined_bytes as usize));

        // setup heap and stack
        self.hpa_write_hpa_node(self.hpa_head_ptr, self.stack_start, false)?; // heap
        self.hpa_write_hpa_node(self.stack_start, HPA_TAIL_SENTINEL_ADDRESS, true)?;
        // stack
        Ok(())
    }
    pub fn read(&self, address: u64) -> Result<u8, ExecutionError> {
        self.physical
            .get(address as usize)
            .ok_or(ExecutionError::new(format!(
                "Memory Access Violation : address {}|{:#x} out of bounds",
                address, address
            )))
            .map(|v| *v)
    }
    pub fn write(&mut self, address: u64, value: u8) -> Result<(), ExecutionError> {
        if let Some(mem_cell) = self.physical.get_mut(address as usize) {
            *mem_cell = value;
        } else {
            return Err(ExecutionError::new(format!(
                "Memory Access Violation : address {}|{:#x} out of bounds",
                address, address
            )));
        }
        Ok(())
    }
    fn read_bytes(&self, address: u64, n: u64) -> Result<Vec<u8>, ExecutionError> {
        let mut buf = Vec::with_capacity(n as usize);
        for i in address..address + n {
            buf.push(self.read(i)?);
        }
        Ok(buf)
    }
    fn write_bytes(&mut self, address: u64, bytes: &[u8]) -> Result<(), ExecutionError> {
        for i in 0..bytes.len() as u64 {
            self.write(address + i, bytes[i as usize])?;
        }
        Ok(())
    }
    pub fn read_immediate(&self, address: u64) -> Result<(u64, u64), ExecutionError> {
        let size_byte = self.read(address)?;
        Ok((
            bytes_to_u64(&self.read_bytes(address + 1, size_byte as u64)?),
            (size_byte + 1) as u64,
        ))
    }
    pub fn read_address(&self, address: u64) -> Result<u64, ExecutionError> {
        Ok(bytes_to_u64(
            &self.read_bytes(address + 1, size_of::<u64>() as u64)?,
        ))
    }

    /*

    heap allocation operates using a linked list of booleans (free/occupied) heap regions
    heap pointer has the following layout [1byte:bool][8byte:next_node_ptr][allocated] with the ptr pointing
    to the first address of allocated and the region extending to (next_node_ptr - 9)
    thus each allocation consumes 9 more bytes at the very end

    for malloc the free region's pointer is copied to the end of the end of the range and a new pointer is created at the original location of the pointer, linked to the moved pointer.
    for free the pointer being freed is marked as free and then a defragmentation algorithm is ran, it checks if the next block is free and if so copies the ptr in the block to itself.
    for realloc it checks if the next block is free and moves the next block up by the realloc amount and relinks it with the current
    */

    // -- Heap Allocator (HPA) -- \\

    pub fn malloc(&mut self, size: u64) -> Result<u64, ExecutionError> {
        self.total_heap_allocations += 1;
        let ptr = self.hpa_get_allocation_canditate(size)?;
        self.hpa_write_hpa_node_allocation_status(ptr, true)?;
        Ok(ptr)
    }

    pub fn realloc(&mut self, ptr: u64, new_size: u64) -> Result<u64, ExecutionError> {
        let (_, current_next_ptr) = self.hpa_read_hpa_node(ptr)?;
        let (next_is_allocated, next_next) = self.hpa_read_hpa_node(current_next_ptr)?;
        let (_, next_next_next) = self.hpa_read_hpa_node(next_next)?;
        let current_size = hpa_block_size(ptr, current_next_ptr);
        if new_size < current_size {
            // create new block in deallocated space
            todo!("shrinking realloc")
        } else {
            if !next_is_allocated && hpa_block_size(next_next, next_next_next) >= new_size {
                let new_ptr = ptr + new_size + HPA_NODE_DATA_OFFSET;

                self.hpa_write_hpa_node(new_ptr, next_next, false)?; // moves high ptr
                self.hpa_write_hpa_node(ptr, new_ptr, true)?; // relinks

                Ok::<u64, ExecutionError>(ptr)
            } else {
                let new_ptr = self.malloc(new_size)?;
                self.memcpy(new_ptr, ptr, current_size)?;
                self.hpa_write_hpa_node_allocation_status(ptr, false)?;
                Ok::<u64, ExecutionError>(new_ptr)
            }
        }
    }

    pub fn free(&mut self, ptr: u64, size: u64) -> Result<(), ExecutionError> {
        self.total_heap_allocations -= 1;
        self.hpa_write_hpa_node_allocation_status(ptr, false)?;
        self.hpa_defragment(ptr, false)?;
        Ok(())
    }

    pub fn memcpy(&mut self, dest: u64, src: u64, n: u64) -> Result<(), ExecutionError> {
        todo!()
    }
    pub fn memset(&mut self, dest: u64, value: u8, n: u64) -> Result<(), ExecutionError> {
        todo!()
    }
    // /// attempts to resolve a potential oom error by defragmenting and then reattempting to search for an allocation canditate, returns an OOM error if it fails a second time
    // fn hpa_oom_recover(&mut self,size:u6) -> Result<(),ExecutionError>{

    // }

    /// returns a pointer to a free memory region that can be allocated into or None if none exists which is big enough
    fn hpa_get_allocation_canditate(&mut self, size: u64) -> Result<u64, ExecutionError> {
        if let Some(final_canditate) = self._hpa_get_allocation_canditate_internal(size)? {
            Ok(final_canditate)
        } else {
            // potential OOM error
            self.hpa_defragment(self.hpa_head_ptr, true)?;
            if let Some(final_canditate) = self._hpa_get_allocation_canditate_internal(size)? {
                Ok(final_canditate)
            } else {
                return Err(ExecutionError::new(format!(
                    "OOM error: could not allocate region of {size} bytes"
                )));
            }
        }
    }

    fn _hpa_get_allocation_canditate_internal(
        &mut self,
        size: u64,
    ) -> Result<Option<u64>, ExecutionError> {
        let mut next = self.hpa_head_ptr;
        let mut canditate: Option<u64> = None;
        loop {
            let (current_is_allocated, current_next) = self.hpa_read_hpa_node(next)?;
            if current_next == HPA_TAIL_SENTINEL_ADDRESS {
                break;
            }
            if current_is_allocated {
                next = current_next;
                continue;
            }
            let canditate_size = (current_next - HPA_NODE_DATA_OFFSET) - next;
            let former_canditate_size = if let Some(n) = canditate { n } else { 0 };
            if canditate_size >= size && canditate_size < former_canditate_size {
                canditate = Some(current_next)
            }
            next = current_next
        }
        Ok(canditate)
    }

    fn hpa_read_hpa_node(&self, ptr: u64) -> Result<(bool, u64), ExecutionError> {
        let is_allocated_ptr = ptr - HPA_NODE_DATA_OFFSET;
        let next_ptr = is_allocated_ptr + 1;
        // let next_ptr = ptr - HPA_NODE_DATA_OFFSET - 1;

        let is_allocated : bool = match self.read(is_allocated_ptr)? {
            0 => false,
            1 => true,
            _ => return Err(ExecutionError::new(format!(
                "heap allocation error: corrupt allocation mapping at block {ptr} (is_allocated flag >1)"
            ))),
        };
        let next = self.read_address(next_ptr)?;

        Ok((is_allocated, next))
    }

    fn hpa_write_hpa_node(
        &mut self,
        ptr: u64,
        next: u64,
        is_allocated: bool,
    ) -> Result<(), ExecutionError> {
        let is_allocated_ptr = ptr - HPA_NODE_DATA_OFFSET;
        let next_ptr = is_allocated_ptr + 1;

        self.write(is_allocated_ptr, is_allocated as u8)?;
        self.write_bytes(next_ptr, &next.to_le_bytes())?;
        Ok(())
    }
    fn hpa_write_hpa_node_allocation_status(
        &mut self,
        ptr: u64,
        is_allocated: bool,
    ) -> Result<(), ExecutionError> {
        let is_allocated_ptr = ptr - HPA_NODE_DATA_OFFSET;
        self.write(is_allocated_ptr, is_allocated as u8)?;
        Ok(())
    }

    fn hpa_read_next_hpa_node(&self, ptr: u64) -> Result<(bool, u64), ExecutionError> {
        self.hpa_read_hpa_node(self.hpa_read_hpa_node(ptr)?.1)
    }
    /// defragments heap allocator block linked list
    /// - `skip_allocated_blocks = false`   | performs a local defragmentation
    /// - `skip_allocated_blocks = true`    | performs a global defragmentation (best if performed at the head)
    fn hpa_defragment(
        &mut self,
        current_ptr: u64,
        skip_allocated_blocks: bool,
    ) -> Result<(), ExecutionError> {
        let (current_is_allocated, current_next) = self.hpa_read_hpa_node(current_ptr)?;
        let (next_is_allocated, next_next) = self.hpa_read_hpa_node(current_next)?;
        if next_is_allocated {
            if skip_allocated_blocks && next_next != HPA_TAIL_SENTINEL_ADDRESS {
                // defragment next block
                self.hpa_defragment(next_next, skip_allocated_blocks)?
            }
            // end
        } else {
            self.hpa_write_hpa_node(current_ptr, next_next, current_is_allocated)?;
        }
        Ok(())
    }
}

fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let target_len = size_of::<u64>();
    let mut byte_buf: Vec<u8> = Vec::with_capacity(target_len);
    byte_buf.extend_from_slice(bytes);
    byte_buf.resize(target_len, 0);
    let byte_array: [u8; size_of::<u64>()] = match byte_buf.try_into() {
        Ok(v) => v,
        Err(_) => panic!("failed to build usize"),
    };
    u64::from_le_bytes(byte_array)
}

fn hpa_block_size(ptr: u64, next_ptr: u64) -> u64 {
    next_ptr - HPA_NODE_DATA_OFFSET - ptr
}

// fn byte_to_bool(byte:u8) -> Result<bool,ExecutionError>{
//     let b = match byte {
//         0 => false,
//         1 => true,
//         _ => return Err(ExecutionError::new(format!(
//             "heap allocation error: corrupt allocation mapping (is_allocated flag >1)"
//         ))),
//     };
//     Ok(b)
// }
