data {
    !lbl equ $1
}
program {

    movim r1,$1
    load r2,r1,43 // load value of ram0 to r1 should be 0xff
    movim r3,$0
    store 44,r1,r3 // store r3 (0) to ram1 (44)
    pr r1
    pr r2
    pr r3
}
