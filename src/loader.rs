use std::{collections::HashMap, fs::File, io::Read, mem::transmute, slice, vec};

use crate::{constant::SIGNATURE, memory::bytes_to_u64, ExecutionError};

pub struct NISVCEF {
    pub entry_point: u64,
    pub image: Vec<u8>,
    pub debug_symbols: DebugSymbols,
    // pub break_points: Vec<u64>,
}

/*

[--signature--][--entry_point--][--img_length--][program][--labels_block_length--][--labels--]

label encoding

[--address--][--label_len--][--label--]

*/

impl NISVCEF {
    pub fn load(file: Vec<u8>) -> Result<Self, ExecutionError> {
        let mut stream = file.into_iter();
        let read_signature = stream.by_ref().take(SIGNATURE.len()).collect::<Vec<u8>>();
        if SIGNATURE != &read_signature {
            return Err(ExecutionError::new(format!("Signature invalid : {}", {
                String::from_utf8_lossy(&read_signature)
            })));
        }
        let entry_point = consume_double_word_vec(&mut stream)?;
        let program_img_len = consume_double_word_vec(&mut stream)?;
        let image: Vec<u8> = stream.by_ref().take(program_img_len as usize).collect();
        let break_point_len = consume_double_word_vec(&mut stream)?;
        // let break_points: Vec<u64> =
        //     build_breakpoint_vector(stream.by_ref().take(break_point_len as usize).collect())?; // convert to Vec<u64> likely gonna do some evil unsafe thing
        let debug_symbols_len = consume_double_word_vec(&mut stream)?;
        let debug_symbols_len = 0;
        let debug_symbols_img: Vec<u8> = stream.by_ref().take(debug_symbols_len as usize).collect();
        println!("entry_point: {entry_point:#x}\nprogram_img_len: {program_img_len:#x}\ndebug_symbols_len:{debug_symbols_len}\ndebug_symbols_img:{debug_symbols_img:x?}");

        let debug_symbols = DebugSymbols::load_symbols(&debug_symbols_img)?;
        Ok(Self {
            entry_point,
            image,
            debug_symbols,
            // break_points,
        })
    }
}

struct DebugSymbols {
    labels: HashMap<u64, String>,
}

impl DebugSymbols {
    fn load_symbols(serialized_block: &[u8]) -> Result<Self, ExecutionError> {
        let mut labels: HashMap<u64, String> = HashMap::new();
        let mut block_stream = serialized_block.into_iter();
        loop {
            let entry = Self::deserialize_entry(&mut block_stream)?;
            if let Some(e) = entry {
                labels.insert(e.0, e.1);
            } else {
                break;
            }
        }
        Ok(Self { labels })
    }

    fn deserialize_entry(
        stream: &mut slice::Iter<'_, u8>,
    ) -> Result<Option<(u64, String)>, ExecutionError> {
        let addr = if let Some(dw) = consume_double_word_ref(stream) {
            dw
        } else {
            return Ok(None);
        };
        let str_len = if let Some(dw) = consume_double_word_ref(stream) {
            dw
        } else {
            return Ok(None);
        };

        let mut label = String::new();

        for i in 0..str_len {
            println!("{label} {i}");
            label.push(*stream.next().ok_or(ExecutionError::new(format!(
                "error decoding label associated with address {addr:#x}"
            )))? as char);
        }
        Ok(Some((addr, label)))
    }
}

/// returns None if dw is incomplete
fn consume_double_word_ref(stream: &mut slice::Iter<'_, u8>) -> Option<u64> {
    let mut buf: [u8; 8] = [0; 8];
    for n in 0..8 {
        buf[n] = if let Some(b) = stream.next() {
            *b
        } else {
            return None;
        }
    }
    Some(bytes_to_u64(&buf))
}
fn consume_double_word_vec(stream: &mut vec::IntoIter<u8>) -> Result<u64, ExecutionError> {
    let mut buf: [u8; 8] = [0; 8];
    for n in 0..8 {
        buf[n] = if let Some(b) = stream.next() {
            b
        } else {
            return Err(ExecutionError::new(format!(
                "entry point incomplete : {:?}",
                buf
            )));
        }
    }
    Ok(bytes_to_u64(&buf))
}

fn build_breakpoint_vector(image: Vec<u8>) -> Result<Vec<u64>, ExecutionError> {
    // check if breakpoint has 8 byte alignment
    // generate list of pointers to the vec at each 8 byte alignment and recast as u64 ptr
    // build vec by dereferencing list

    if (image.len() % 8) != 0 {
        return Err(ExecutionError::new(format!(
            "breakpoint vector does not have an 8 byte alignment"
        )));
    }

    let mut buf: Vec<u64> = Vec::new();

    for chunk in image.chunks_exact(8) {
        let mut bytes: [u8; 8] = [0; 8];
        for (i, byte) in chunk.into_iter().enumerate() {
            bytes[i] = *byte;
        }
        buf.push(u64::from_le_bytes(bytes));
    }

    // for n in (0..image.len()).step_by(8) {
    //     let ptr_u8: *const u8 = &image[n];
    //     let ptr_u64: *const u64 = unsafe { transmute(ptr_u8) };
    //     buf.push(unsafe { *ptr_u64 });
    // }

    Ok(buf)
}
