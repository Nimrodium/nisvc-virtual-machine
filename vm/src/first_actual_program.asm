// NISVC assembly source code
data {;}
program { // as you can see comments are fully supported

    movim r1,#10;
    mov r2,r1; //add r3,r2,r1; as well as inline. however ofc the assembly highlighting is skuffed.
    movim r4,#2;
    sub r5,r3,r2

}

// r1 = 10
// r2 = 10
// r3 = 20 (10+10)
// r4 = 2
// r5 = 10 (20-10)
