use std::mem::transmute;

use crate::{constant::UNINITIALIZED_MEMORY, kernel_log, ExecutionError};
const HPA_NODE_DATA_OFFSET: u64 = 9;
const HPA_TAIL_SENTINEL_ADDRESS: u64 = 0;
pub type nisvc_ptr = u64;
pub struct Memory {
    pub physical: Vec<u8>,
    range: u64,
    heap_size: u64,
    stack_size: u64,
    pub heap_start: u64,
    pub stack_start: u64,
    total_heap_allocations: u64,
}

impl Memory {
    pub fn new(heap: u64, stack: u64) -> Self {
        Self {
            physical: Vec::with_capacity((heap + stack) as usize),
            range: 0,
            heap_size: heap,
            stack_size: stack,
            heap_start: 0,
            stack_start: 0,
            total_heap_allocations: 0,
        }
    }
    pub fn load(&mut self, image: Vec<u8>) -> Result<(), ExecutionError> {
        let image_size = image.len() as u64;
        self.range = image_size + self.heap_size + self.stack_size;
        self.physical.reserve(image_size as usize);
        self.physical.extend(image);
        self.physical.extend(
            std::iter::repeat(UNINITIALIZED_MEMORY)
                .take((self.heap_size + self.stack_size) as usize),
        );
        self.heap_start = image_size + HPA_NODE_DATA_OFFSET;
        self.stack_start = self.heap_start + self.heap_size + HPA_NODE_DATA_OFFSET;

        // setup heap and stack
        self.hpa_write_hpa_node(self.heap_start, self.stack_start, false)?; // heap
        self.hpa_write_hpa_node(self.stack_start, HPA_TAIL_SENTINEL_ADDRESS, true)?;
        println!(
            "physical memory size: {}\nheap_ptr: {}\nstack_ptr: {}",
            self.physical.len(),
            self.heap_start,
            self.stack_start
        );
        Ok(())
    }
    pub fn read_byte(&self, address: u64) -> Result<u8, ExecutionError> {
        self.physical
            .get(address as usize)
            .ok_or(ExecutionError::new(format!(
                "Memory Access Violation : address {}|{:#x} out of bounds",
                address, address
            )))
            .map(|v| *v)
    }
    pub fn write_byte(&mut self, address: u64, value: u8) -> Result<(), ExecutionError> {
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
    pub fn read(&self, address: u64, n: u64) -> Result<Vec<u8>, ExecutionError> {
        let mut buf = Vec::with_capacity(n as usize);
        for i in address..address + n {
            buf.push(self.read_byte(i)?);
        }
        Ok(buf)
    }
    pub fn write(&mut self, address: u64, bytes: &[u8]) -> Result<u64, ExecutionError> {
        let mut bytes_wrote = 0;
        for i in 0..bytes.len() as u64 {
            self.write_byte(address + i, bytes[i as usize])?;
            bytes_wrote += 1;
        }
        Ok(bytes_wrote)
    }
    // /// (value,bytes_read)
    // pub fn read_immediate(&self, address: u64) -> Result<(u64, u64), ExecutionError> {
    //     let size_byte = self.read_byte(address)?;
    //     Ok((
    //         bytes_to_u64(&self.read(address + 1, size_byte as u64)?),
    //         (size_byte + 1) as u64,
    //     ))
    // }
    pub fn read_address(&self, address: u64) -> Result<u64, ExecutionError> {
        Ok(bytes_to_u64(&self.read(address, size_of::<u64>() as u64)?))
    }
    // returns stack pointer
    pub fn push(&mut self, stack_ptr: u64, value: u64) -> Result<u64, ExecutionError> {
        let bytes_wrote = self.write(stack_ptr, &value.to_le_bytes())?;
        Ok(stack_ptr + bytes_wrote)
    }
    // returns stack pointer and popped value
    pub fn pop(&mut self, stack_ptr: u64) -> Result<(u64, u64), ExecutionError> {
        let value_size = size_of::<u64>() as u64;
        let ptr = stack_ptr - value_size;
        let value = bytes_to_u64(&self.read(ptr, value_size)?);
        Ok((ptr, value))
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
        let new_node_ptr = ptr + size + HPA_NODE_DATA_OFFSET;
        let (_, old_next) = self.hpa_read_hpa_node(ptr)?;
        self.hpa_write_hpa_node(ptr, new_node_ptr, true)?;
        self.hpa_write_hpa_node(new_node_ptr, old_next, false)?;
        Ok(ptr)
    }

    pub fn realloc(&mut self, ptr: u64, new_size: u64) -> Result<u64, ExecutionError> {
        let (_, current_next_ptr) = self.hpa_read_hpa_node(ptr)?;
        let (next_is_allocated, next_next) = self.hpa_read_hpa_node(current_next_ptr)?;
        let (_, next_next_next) = self.hpa_read_hpa_node(next_next)?;
        let current_size = hpa_block_size(ptr, current_next_ptr);
        if current_size == new_size {
            // nop
            return Ok(ptr);
        }
        if new_size < current_size {
            // moves next ptr down, if next block is allocated then link new_ptr to the old_next, else link to allocated
            let shrink_size = current_size - new_size;
            let new_ptr = ptr + shrink_size;
            let new_ptr_link = if next_is_allocated {
                current_next_ptr
            } else {
                next_next
            };
            self.hpa_write_hpa_node(new_ptr, new_ptr_link, false)?;

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
    // fn hpa_realloc_mv(
    //     &mut self,
    //     ptr: u64,
    //     current_size: u64,
    //     new_size: u64,
    // ) -> Result<u64, ExecutionError> {
    //     let new_ptr = self.malloc(new_size)?;
    //     self.memcpy(new_ptr, ptr, current_size)?;
    //     self.hpa_write_hpa_node_allocation_status(ptr, false)?;
    //     Ok::<u64, ExecutionError>(new_ptr)
    // }
    pub fn free(&mut self, ptr: u64) -> Result<(), ExecutionError> {
        self.total_heap_allocations -= 1;
        self.hpa_write_hpa_node_allocation_status(ptr, false)?;
        self.hpa_defragment(ptr, false)?;
        Ok(())
    }

    pub fn memcpy(&mut self, dest: u64, src: u64, n: u64) -> Result<(), ExecutionError> {
        for ptr in 0..n {
            let src_ptr = src + ptr;
            let dest_ptr = dest + ptr;
            let src_byte = self.read_byte(src_ptr)?;
            self.write_byte(dest_ptr, src_byte)?;
            // println!("{dest_ptr:#x}: {}", self.read_byte(dest_ptr)? as char);
        }
        Ok(())
    }
    pub fn memset(&mut self, dest: u64, value: u8, n: u64) -> Result<(), ExecutionError> {
        for ptr in dest..dest + n {
            self.write_byte(ptr, value)?;
        }
        Ok(())
    }
    // /// attempts to resolve a potential oom error by defragmenting and then reattempting to search for an allocation canditate, returns an OOM error if it fails a second time
    // fn hpa_oom_recover(&mut self,size:u6) -> Result<(),ExecutionError>{

    // }

    /// returns a pointer to a free memory region that can be allocated into or None if none exists which is big enough
    fn hpa_get_allocation_canditate(&mut self, size: u64) -> Result<u64, ExecutionError> {
        if let Some(final_canditate) = self.hpa_get_allocation_canditate_internal(size)? {
            Ok(final_canditate)
        } else {
            // potential OOM error
            kernel_log!("under memory pressure");
            self.hpa_defragment(self.heap_start, true)?;
            if let Some(final_canditate) = self.hpa_get_allocation_canditate_internal(size)? {
                Ok(final_canditate)
            } else {
                return Err(ExecutionError::new(format!(
                    "OOM error: could not allocate region of {size} bytes"
                )));
            }
        }
    }

    fn hpa_get_allocation_canditate_internal(
        &mut self,
        size: u64,
    ) -> Result<Option<u64>, ExecutionError> {
        let mut ptr = self.heap_start;
        let mut canditate: Option<u64> = None;
        loop {
            let (current_is_allocated, current_next) = self.hpa_read_hpa_node(ptr)?;
            // println!("node: {current_is_allocated}:{current_next:#x}");
            if current_next == HPA_TAIL_SENTINEL_ADDRESS {
                break;
            }
            if current_is_allocated {
                ptr = current_next;
                continue;
            }
            let canditate_size = (current_next - HPA_NODE_DATA_OFFSET) - ptr;
            let former_canditate_size = if let Some(n) = canditate { n } else { u64::MAX };
            if canditate_size >= size && canditate_size < former_canditate_size {
                canditate = Some(ptr)
            }
            ptr = current_next
        }
        Ok(canditate)
    }

    fn hpa_read_hpa_node(&self, ptr: u64) -> Result<(bool, u64), ExecutionError> {
        let is_allocated_ptr = ptr - HPA_NODE_DATA_OFFSET;
        let next_ptr = is_allocated_ptr + 1;
        // let next_ptr = ptr - HPA_NODE_DATA_OFFSET - 1;

        let is_allocated : bool = match self.read_byte(is_allocated_ptr)? {
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

        self.write_byte(is_allocated_ptr, is_allocated as u8)?;
        self.write(next_ptr, &next.to_le_bytes())?;
        Ok(())
    }
    fn hpa_write_hpa_node_allocation_status(
        &mut self,
        ptr: u64,
        is_allocated: bool,
    ) -> Result<(), ExecutionError> {
        let is_allocated_ptr = ptr - HPA_NODE_DATA_OFFSET;
        self.write_byte(is_allocated_ptr, is_allocated as u8)?;
        Ok(())
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

pub fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut buf: [u8; 8] = [0; 8];
    buf.copy_from_slice(bytes);
    u64::from_le_bytes(buf)
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
