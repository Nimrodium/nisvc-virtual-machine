data {;}
program {

    movim r1,$1
    shl r2,r1,r1 // 2
    shl r3,r2,r1 // 4
    shl r4,r3,r1 // 8
    shl r5,r4,r1 // 16
    shl r6,r5,r1 // 32
    shl r7,r6,r1 // 64
    shl r8,r7,r1 // 128
    shl r9,r8,r1 // 256
    shl r10,r9,r1 // 512
    neg r11,r10
}
