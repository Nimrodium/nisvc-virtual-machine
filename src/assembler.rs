// assembler.rs

use std::{
    collections::HashMap,
    fs::File,
    io::{read_to_string, Write},
    path::Path,
};
type LabelTable = HashMap<String, u64>;
type OpcodeTable = HashMap<String, OpcodeEncoding>;
use crate::{
    constant::{self, OpcodeSize},
    opcode::Opcode,
};

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

fn parse_data_section(source: &str) -> Result<(Vec<u8>, LabelTable, usize), String> {
    todo!()
}

fn parse_program_section(
    source: &str,
    label_table: LabelTable,
    opcodes: OpcodeTable,
    head: usize,
) -> Result<Vec<u8>, String> {
    todo!()
}

fn assemble(source_path: &Path) -> Result<Vec<u8>, String> {
    let source = match File::open(source_path) {
        Ok(f) => f,
        Err(why) => return Err(format!("could not load source file :: {why}")),
    };
    let source_str = match read_to_string(source) {
        Ok(str) => str,
        Err(why) => return Err(format!("could not load source file :: {why}")),
    };
    let (datarom, label_table, bytes_read) = parse_data_section(&source_str)?;

    let opcode_table = opcode_encoding_table();

    let program_rom = parse_program_section(&source_str, label_table, opcode_table, bytes_read)?;

    let mut header = build_header(&datarom, &program_rom);
    header.extend_from_slice(&datarom);
    header.extend_from_slice(&program_rom);
    Ok(header)
}

fn write_to_file(compiled_source: Vec<u8>, file: &Path) -> Result<(), String> {
    let mut out = match File::create(file) {
        Ok(f) => f,
        Err(why) => return Err(format!("could not create output file :: {why}")),
    };
    match out.write_all(&compiled_source) {
        Ok(()) => (),
        Err(why) => return Err(format!("could not write to output file :: {why}")),
    };
    Ok(())
}
struct OpcodeEncoding {
    name: String,
    code: OpcodeSize,
    fields: usize,
}

impl OpcodeEncoding {
    fn new(name: &str, opcode_enum: Opcode, fields: usize) -> Self {
        OpcodeEncoding {
            name: name.to_string(),
            code: opcode_enum as OpcodeSize,
            fields,
        }
    }
}

/// returns lookup table for opcodes
fn opcode_encoding_table() -> OpcodeTable {
    let opcodes = [
        OpcodeEncoding::new("mov", Opcode::Mov, 2),
        OpcodeEncoding::new("load", Opcode::Load, 2),
        OpcodeEncoding::new("store", Opcode::Store, 2),
        OpcodeEncoding::new("add", Opcode::Add, 3),
        OpcodeEncoding::new("sub", Opcode::Sub, 3),
        OpcodeEncoding::new("mult", Opcode::Mult, 3),
        OpcodeEncoding::new("div", Opcode::Div, 4),
    ];
    let mut lookup_table: HashMap<String, OpcodeEncoding> = HashMap::new();
    for opcode in opcodes {
        lookup_table.insert(opcode.name.clone(), opcode);
    }
    lookup_table
}

// fn decode_line(line:&str,opcode_table:&OpcodeTable,label_table:LabelTable)
