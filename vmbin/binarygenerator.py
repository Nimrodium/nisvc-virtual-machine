# simple nimcode binary generator
def assemble(name:str,data:list[int],program:list[int]):
    with open(name, "wb") as f:
        # Write signature
        f.write(b"NISVC-EF")
        f.write((len(data)).to_bytes(8, byteorder='little'))
        f.write((len(program)).to_bytes(8, byteorder='little'))
        f.write(bytes(data))
        f.write(bytes(program))


data = []
program = [
    0x02,0x00,  # movim
    0x01,0x00,  # r1
    0x08,0x0a,0x0a,    # u16 2570
    0x01,0x00,  # mov
    0x02,0x00,  # r2
    0x01,0x00,  # r1
    0xFF,0xFF   # end
]

assemble("test.bin",data,program)
