data {;}
program {

    movim r1,$1
    load r2,r1,300 // guaranteed to be well within ram
    pr r2 // should be 0xFF
    sub r3,r2,r1 // r3 = 0xFE
    store 301,r1,r3 // write 0xFE to address 301
    load r4,r1,301 // read value at address 301 to register 4
    pr r4 // should be 0xFE (254)
}
