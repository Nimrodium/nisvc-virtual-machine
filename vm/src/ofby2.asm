data {;}
program {
    movim r1,$1
    movim r2,$10

    !lbl
        pr r2
        sub r2,r2,r1 // decrement 10 by 1
    jifnz r2,!lbl
}
