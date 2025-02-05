data {;}
program {
    movim r1,#10
    movim r2,#11
    sub r3,r1,r2 // will underflow
}
