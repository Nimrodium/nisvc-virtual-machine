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
mov_test = [
    0x02,0x00,  # movim
    0x01,0x00,  # r1
    0x02,0x0a,0x0a,    # u16 2570
    0x01,0x00,  # mov
    0x02,0x00,  # r2
    0x01,0x00,  # r1
    0xFF,0xFF   # end
]

add_test = [

    0x02,0x00, # movim
    0x01,0x00, # r1
    0x01,0x02, # 1b#2

    0x02,0x00, # movim
    0x02,0x00, # r2
    0x01,0x03, # 1b#3

    0x05,0x00, # add
    0x03,0x00, # r3
    0x01,0x00, # r1
    0x02,0x00, # r2
    0xff,0xff, # end

# output
# r1 = 2
# r2 = 3
# r3 = 5
]

arithmetic_test = [
        # movim r1,2
        # movim r2,3
        # add r3,r1,r2
        # sub r4,r2,r1
        # mult r5,r1,r2

        0x02,0x00, # movim
        0x01,0x00, # r1
        0x01,0x02, # 1b#2

        0x02,0x00, # movim
        0x02,0x00, # r2
        0x01,0x03, # 1b#3

        0x05,0x00, # add
        0x03,0x00, # r3
        0x01,0x00, # r1
        0x02,0x00, # r2

        0x06,0x00, # sub
        0x04,0x00, # r4
        0x02,0x00, # r2
        0x01,0x00, # r1

        0x07,0x00, # mult
        0x05,0x00, # r5
        0x01,0x00, # r1
        0x02,0x00, # r2
        0xff,0xff, # end

    # output
    # r1 = 2
    # r2 = 3
    # r3 = 5
    # r4 = 1
    # r5 = 6

]


assemble("arith.bin",data,arithmetic_test)
