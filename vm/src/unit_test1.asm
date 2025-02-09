data {
    !test equ $0
}
program {

    movim r1,$1
    movim r2,@1
    load r3,r1,r2 // load one byte from address @1 to r3 should be 0xff
    sub r4,r3,r1 // 0xff - 1 = 0xfe
    add r5,r2,r1 // @1
    store r5,r1,r4 // store 1 byte from r5 to [r5]
    load r6,r1,r5 // load 1 byte from [r5] to r6
    pr r1 // 1
    pr r2 // @0
    pr r3 // 0xff || 255
    pr r4 // 0xfe || 254
    pr r5 // @1
    pr r6 // 0xfe || 254
}
