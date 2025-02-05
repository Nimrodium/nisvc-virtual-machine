data {;}

program {
	movim r1,$10
	movim r2,$1

	!jmp_label
	sub r1,r1, r2 // r3 will decrease
	pr r1
	jifnz r1,!jmp_label
	// r1 should be 0
}
