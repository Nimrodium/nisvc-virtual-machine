# simple nimcode assembler
#

import enum
import sys
from types import resolve_bases
from typing_extensions import Sequence
U8_MAX = 255
U16_MAX = 65_535
U32_MAX = 4_294_967_295
U64_MAX = 18_446_744_073_709_551_615

REGISTER_BYTES = 2
OPCODE_BYTES = 2
ADDRESS_BYTES = 8
MMIO_ADDRESS_SPACE = 10
LABEL = "!"
SEPERATOR = ";"
COMMENT = "//"
LITERAL = "#"
OPEN_SECTION = "{"
CLOSE_SECTION = "}"

class Tp(enum.Enum):
    Reg = enum.auto(),
    Addr = enum.auto(),
    Imm = enum.auto(),
    Op = enum.auto(),
def immediate_byte_size(value) -> int:
    if value > U64_MAX: # cannot be fit
        raise ValueError(f"immediate {value} over u64 limit {U64_MAX}")
    if value < 0:
        raise ValueError(f"immediate {value} is negative which is not supported in the current nimcode implementation")
    if value > U32_MAX:
        return 8
    elif value > U16_MAX:
        return 4
    elif value > U8_MAX:
        return 2
    else:
        return 1

opcode_table : dict[str,tuple[int,int,list]] = {
    "nop":(0x00,0,[]),
    "mov":(0x01,2,[Tp.Reg,Tp.Reg]),
    "movim":(0x02,2,[Tp.Reg,Tp.Imm]),
    "load":(0x02,3,[Tp.Reg,Tp.Reg,Tp.Addr]),
    "store":(0x04,3,[Tp.Addr,Tp.Reg,Tp.Reg]),

    "add":(0x05,3,[Tp.Reg,Tp.Reg,Tp.Reg]),
    "sub":(0x06,3,[Tp.Reg,Tp.Reg,Tp.Reg]),
    "mult":(0x07,3,[Tp.Reg,Tp.Reg,Tp.Reg]),
    "div":(0x08,3,[Tp.Reg,Tp.Reg,Tp.Reg]),

    "or":(0x09,3,[Tp.Reg,Tp.Reg,Tp.Reg]),
    "xor":(0x05,3,[Tp.Reg,Tp.Reg,Tp.Reg]),
    "and":(0x05,3,[Tp.Reg,Tp.Reg,Tp.Reg]),
    "not":(0x05,2,[Tp.Reg,Tp.Reg]),
    "shl":(0x05,3,[Tp.Reg,Tp.Reg,Tp.Reg]),
    "shr":(0x05,3,[Tp.Reg,Tp.Reg,Tp.Reg]),
}

register_table = {
    "r1":1,
    "r2":2,
    "r3":3,
    "r4":4,
    "r5":5,
    "r6":6,
    "r7":7,
    "r8":8,
    "r9":9,
    "r10":10,
    "r11":11,
    "r12":12,
    "r13":13,
    "r14":14,
    "r15":15,
    "r16":16,
    "r17":17,
    "r18":18,
    "r19":19,
    "r20":20,
    "pc":21,
    "sp":22,
    "o1":23,
    "o2":24,
    "o3":25,
    "o4":26,
    "o5":27,
}


def clean_source(source:str) -> list[str]:
    source_lines = source.strip().splitlines()
    cleaned_source : list[str] = []
    for line in source_lines:
        if not line:
            #empty line
            continue
        line_no_comments = line.split(COMMENT)[0].strip() # takes all chars before //
        if not line_no_comments:
            # line was entirely comment
            continue
        instructions = line_no_comments.split(SEPERATOR)
        cleaned_instructions : list[str] = []
        for line in instructions:
            clean_line = line.strip()
            if line:
               cleaned_instructions.append(clean_line)

        print(cleaned_instructions)
        cleaned_source.extend(cleaned_instructions)
    return cleaned_source

def is_immediate(operand: str) -> bool:
    return operand.startswith(LITERAL) and operand[1:].isdigit()
# list[list[tuple[int,tuple[int]]]]
#
def calc_instruction_size(instruction:list[tuple[int|str,Tp,int]]) -> int:
    size = 0
    for element in instruction:
        size+=element[2]
    return size

# returns label -> program address dictionary for the program section
def resolve_program_labels(source:list[str],intermediate:list[list[tuple[int|str,Tp,int]]]) -> dict[str,int]:
    labels : dict[str,int] = {}
    head = MMIO_ADDRESS_SPACE
    i = 0
    for line in source:

        if line.startswith(LABEL):
            # add label entry
            label_name = line.lower().strip().partition(" ")[0]
            print(f"{label_name} pointing to pc:{head}")
            labels.update({label_name:head})
        else:
            # advance head
            print(f"line [ {line} ] at pc:{head}")
            head+= calc_instruction_size(intermediate[i])
            i+=1
    return labels



# reads data section and returns data byte sequence and label table
# ram_base == last address of program + 1
def parse_data_section(source:list[str]) -> tuple[bytes,dict[str,int]]:

    return (bytes([0]),{})



def merge_program_and_data_labels(program_labels:dict[str,int],data_labels:dict[str,int],program_length) -> dict[str,int]:
    # data labels are relative to start of data
    # program labels are already absolute (accounts for MMIO address space)
    # just add the length of program to all and then merge

    # account for end of exec marker and move 1 so that data0 is not in program space
    program_length+=OPCODE_BYTES+1

    for label_name,label_address in data_labels.items():
        if label_name in program_labels:
            raise NameError(f"program and data label collision {label_name} already in program labels")
        program_labels.update({label_name:label_address+program_length})
    return program_labels

def assemble_program(source:list[str]) -> list[list[tuple[int|str,Tp,int]]]:
    program : list[list[tuple[int|str,Tp,int]]] = []
    # program_labels : dict[str,int] =


    for raw_instruction in source:
        if raw_instruction.startswith(LABEL):
            # skip labels
            continue
        raw_instruction = raw_instruction.lower().strip().split(" ") # "mov r1,r2" -> ["mov","r1,r2"]
        operation_str : str = raw_instruction[0] # "mov"


        if operation_str not in opcode_table: # resolve operation
            raise Exception(f"unrecognized operation {operation_str}")

        operation_code,fields,types = opcode_table[operation_str]

        if fields > 0:
            operands_str : list[str] = raw_instruction[1].strip().split(",") # ["r1","r2"]
        else:
            operands_str = []
        # print(f"operation: {operation_str}\noperands: {operands_str}")

        if len(operands_str) != fields:
            raise Exception(f"incorrect amount of operands :: {operation_str} expects {fields} while only {len(operands_str)} were provided")
        instruction : list[tuple[int|str,Tp,int]] = [(operation_code,Tp.Op,OPCODE_BYTES)]
        for i,operand in enumerate(operands_str):

            data_type : Tp = types[i]
            match data_type:
                case Tp.Reg:
                    if operand not in register_table.keys():
                        raise ValueError(f"{operand} not a valid register which is expected by {operation_str} operation at position {i}")
                    resolved_register = register_table[operand]
                    instruction.append((resolved_register,Tp.Reg,REGISTER_BYTES))

                case Tp.Addr:
                    print(f"found address label {operand} during assembly")
                    instruction.append((operand,Tp.Addr,ADDRESS_BYTES))

                case Tp.Imm:
                    if not is_immediate(operand):
                        raise ValueError(f"{operand} not a valid immediate which is expected by {operation_str} operation at position {i}")
                        # needs to find smallest size immediate can be skrunkled down to
                        # for now for testing, assume all are u8
                    immediate : int = int(operand[1:])
                    # print(f"immediate {immediate}")
                    instruction.append((immediate,Tp.Imm,immediate_byte_size(immediate)+1))
                case Tp.Op:
                    raise ValueError("definition format error")

        print(f"instruction : {instruction}")
        program.append(instruction)
    return program

def calculate_program_byte_length(intermediate_program:list[list[tuple[int|str,Tp,int]]]) -> int:
    length = MMIO_ADDRESS_SPACE
    for instruction in intermediate_program:
        length+=calc_instruction_size(instruction)
    return length

def insert_addresses(labels:dict[str,int],intermediate_program:list[list[tuple[str|int,Tp,int]]]) -> list[list[tuple[int,Tp,int]]]:
    resolved_intermediate : list[list[tuple[int,Tp,int]]] = []
    for instruction in intermediate_program:
        resolved_instruction : list[tuple[int,Tp,int]] = []
        for object,data_type,type in instruction:
            if data_type == Tp.Addr:
                if isinstance(object,int):
                    print("already resolved??? how did yo uget here, merge this if later")
                    continue
                if object not in labels.keys():
                    raise NameError(f"label undefined {object} was never defined")
                print(f"found label {object} resolving to {labels[object]}")
                object = labels[object]
            assert isinstance(object,int)

            new : tuple[int,Tp,int]= (object,data_type,type)
            print(new)
            resolved_instruction.append(new)

        resolved_intermediate.append(resolved_instruction)
    return resolved_intermediate


def expand_bytes(intermediate_program:list[list[tuple[int,Tp,int]]]) -> bytes:
    resolved_program_bytes : list[int] = []
    for instruction in intermediate_program:
        # First element is opcode, second is type, third is operands
        for object,data_type,_ in instruction:
            expand : int = 0
            match data_type:
                case Tp.Op:
                    # print(f"{object} is opcode expanding to {OPCODE_BYTES}")
                    expand = OPCODE_BYTES
                case Tp.Reg:
                    # print(f"{object} is register expanding to {REGISTER_BYTES}")
                    expand = REGISTER_BYTES
                case Tp.Addr:
                    # print(f"{object} is address expanding to {ADDRESS_BYTES}")
                    expand = ADDRESS_BYTES
                case Tp.Imm:
                    size = immediate_byte_size(object)
                    # print(f"{object} is immediate, expanding to {size}")
                    resolved_program_bytes.append(size) # add size byte before immediate
                    expand = size
            try:
                expanded = object.to_bytes(expand,"little")
            except AttributeError:
                raise Exception(f"{object} never dereferenced")

            # resolved_program_bytes.append(expanded)
            resolved_program_bytes.extend(list(expanded))
    # add end of execution marker with dynamic size based on opcode size
    end_of_exec = []
    for byte in range(OPCODE_BYTES):
        end_of_exec.append(0xFF)
    resolved_program_bytes.extend(end_of_exec)
    return bytes(resolved_program_bytes)


# package data and program into nisvc-ef executable binary image
def nisvc_ef_build(name:str,data:bytes,program:bytes):
    with open(name, "wb") as f:
        # Write signature
        f.write(b"NISVC-EF")

        f.write((len(program)).to_bytes(8, byteorder='little'))
        f.write((len(data)).to_bytes(8, byteorder='little'))

        f.write(program)
        f.write(data)
    print(f"assembled binary {name}")

def assemble(data_source:list[str],program_source:list[str]) -> tuple[bytes,bytes]:
    assembled_data,data_labels = parse_data_section(data_source)

    intermediate_program = assemble_program(program_source)
    program_labels = resolve_program_labels(program_source,intermediate_program)
    program_length = calculate_program_byte_length(intermediate_program)
    merged_labels = merge_program_and_data_labels(program_labels,data_labels,program_length)

    resolved_intermediate = insert_addresses(merged_labels,intermediate_program)
    program = expand_bytes(resolved_intermediate)

    return (assembled_data,program)


def load_section(section:list[str]) -> tuple[list[str],int]: #section lines, last line read
    last_char_open_check = section[0][-1] # last char
    if last_char_open_check != OPEN_SECTION:
        raise SyntaxError(f"section passed was not opened properly, expected [ {OPEN_SECTION} ] read [ {last_char_open_check} ]")
    in_section = True
    print("in section")
    section_buffer : list[str] = []
    lines_read = 1

    skip_first_iteration = True
    while in_section:
        if skip_first_iteration:
            skip_first_iteration = False
            continue
        line = section[lines_read]

        last_char_closed_check = line[-1] # last char
        if last_char_closed_check == CLOSE_SECTION: # found end
            in_section = False
            print("exited section")
            continue
        section_buffer.append(line)

        lines_read+=1
    return (section_buffer,lines_read)

# only slightly less fucked but still VERYYY skuffed
def parse_initial(source:str) -> tuple[list[str],list[str]]:
    sections : dict[str,list[str]] = {}
    cleaned_source = clean_source(source)
    section_name,_,_ = cleaned_source[0].partition(" ")
    if section_name != "program" and section_name != "data":
        raise SyntaxError(f"invalid section name {section_name}")
    sections[section_name],last_line = load_section(cleaned_source)

    second_source = cleaned_source[last_line+1:]

    section2_name,_,_ = second_source[0].partition(" ")
    if section2_name != "program" and section2_name != "data":
        raise SyntaxError(f"invalid section name {section2_name}")

    print("program source : ",second_source)
    sections[section2_name],_ = load_section(second_source)

    print(f"data:\n\n{sections["data"]}\n\nprogram:\n\n{sections["program"]}\n")
    return sections["data"],sections["program"]


in_file : str = open(sys.argv[1]).read() # currently expects raw program, no memory access
out_file : str =  sys.argv[2]
data,program = parse_initial(in_file)
data : list[str] = []
asm_data,asm_program = assemble(data,program)
nisvc_ef_build(out_file,asm_data,asm_program)
