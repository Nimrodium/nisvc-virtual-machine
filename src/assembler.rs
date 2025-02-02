// assembler.rs

use std::path::Path;

use crate::constant;

fn build_header(datarom: &Vec<u8>, program_rom: &Vec<u8>) -> Vec<u8> {
    let mut header: Vec<u8> = vec![];
    let signature = constant::SIGNATURE.as_bytes();
    header.extend_from_slice(signature);
    let datarom_length = u64::to_le_bytes(datarom.len() as u64);
    let programrom_length = u64::to_le_bytes(program_rom.len() as u64);
    header.extend_from_slice(&datarom_length);
    header.extend_from_slice(&programrom_length);
    header
}

fn parse_data_section() -> Vec<u8> {
    todo!()
}

fn parse_program_section() -> Vec<u8> {
    todo!()
}

fn assemble(source: &Path) {}
