# simple nimcode binary generator
#

import enum

U8_MAX = 255
U16_MAX = 65_535
U32_MAX = 4_294_967_295
U64_MAX = 18_446_744_073_709_551_615

REGISTER_BYTES = 2
OPCODE_BYTES = 2
ADDRESS_BYTES = 8

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
    "load":(0x02,3,[Tp.Reg,Tp.Addr]),
    "store":(0x04,3,[Tp.Addr,Tp.Reg]),

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



def is_immediate(operand: str) -> bool:
    return operand.startswith('#') and operand[1:].isdigit()
# list[list[tuple[int,tuple[int]]]]
def assemble_program(source:str,labels:dict[str,int]) -> list[list[tuple[int,Tp]]]:
    program : list[list[tuple[int,Tp]]] = []
    for line in source.strip().split(";"):
        if len(line) == 0:
            print("empty line")
            continue
        print(f"line :{line}")
        # print(f"line len {len(line)}")
        line = line.lower().strip().split(" ") # "mov r1,r2" -> ["mov","r1,r2"]
        operation_str : str = line[0] # "mov"
        operands_str : list[str] = line[1].strip().split(",") # ["r1","r2"]


        if operation_str not in opcode_table: # resolve operation
            raise Exception(f"unrecognized operation {operation_str}")

        operation_code,fields,types = opcode_table[operation_str]

        # print(f"operation: {operation_str}\noperands: {operands_str}")

        if len(operands_str) != fields:
            raise Exception("incorrect amount of operands")
        instruction = [(operation_code,Tp.Op)]
        # resolved_operands : list[tuple[int,Tp]] = []
        for i,operand in enumerate(operands_str):

            data_type : Tp = types[i]
            match data_type:
                case Tp.Reg:
                    if operand not in register_table.keys():
                        raise ValueError(f"{operand} not a valid register which is expected by {operation_str} operation at position {i}")
                    resolved_register = register_table[operand]
                    instruction.append((resolved_register,Tp.Reg))

                case Tp.Addr:
                    if operand not in labels.keys():
                        raise ValueError(f"{operand} not a valid label which is expected by {operation_str} operation at position {i}")
                        resolved_label : int = labels[operand]
                        instruction.append((resolved_label,Tp.Addr))

                case Tp.Imm:
                    if not is_immediate(operand):
                        raise ValueError(f"{operand} not a valid immediate which is expected by {operation_str} operation at position {i}")
                        # needs to find smallest size immediate can be skrunkled down to
                        # for now for testing, assume all are u8
                    immediate : int = int(operand[1:])
                    # print(f"immediate {immediate}")
                    instruction.append((immediate,Tp.Imm))
                case Tp.Op:
                    raise ValueError("definition format error")

        print(f"instruction : {instruction}")
        program.append(instruction)
    return program


def expand_bytes(intermediate_program:list[list[tuple[int,Tp]]]) -> bytes:
    resolved_program_bytes : list[int] = []
    for instruction in intermediate_program:
        # First element is opcode, second is type, third is operands
        for object,data_type in instruction:
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

            expanded = object.to_bytes(expand,"little")
            # resolved_program_bytes.append(expanded)
            resolved_program_bytes.extend(list(expanded))
    # add end of execution marker
    resolved_program_bytes.extend([0xFF, 0xFF])
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

data : bytes = bytes([0])
test_program = "                \
                movim r1,#10;  \
                mov r2,r1;      \
                add r3,r1,r2;   \
                "

intermediate = assemble_program(test_program,{})
program = expand_bytes(intermediate)
print(program)
nisvc_ef_build("assembler_test.bin",data,program)
