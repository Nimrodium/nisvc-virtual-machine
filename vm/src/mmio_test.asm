data {
    !key equ $0
    !dis_ctrl equ $1
    !cursor equ $4
    !write equ $5
}

program {

    movim r1,$1
    movim r2,$4
    movim r3,$5
    movim r4,$1
    movim r7,$2
    movim r5,$0
    // set up display
    store r1,r4,r4 // set to show
    store r1,r4,r7
    !loop
        load r6,r4,r5 // get current input
        jifz r6,!loop // skips iteration if no input
        store r3,r4,r6 // write input to cursor
        store r2,r4,r4 // move cursor right
        store r1,r4,r7 // refresh display
    jmp !loop
}
