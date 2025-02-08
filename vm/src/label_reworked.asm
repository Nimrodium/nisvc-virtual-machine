data {
    !test equ $(1+2)
    !test2 equ $(!test+1)
}

program {

    movim r1,!test
    movim r2,!test2
    !jmplbl
    add r2,r2,r1
    pr r2
    movim r3,$100
    sub r4,r2,r3
    jifnz r4,!jmplbl // !jmplbl should be 50 but is being resolved as 48

}
