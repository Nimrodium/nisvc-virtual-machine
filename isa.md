all instructions will trunecate the value to be written
 to the least significant bytes if the destination register
 is smaller than the value being written

if an instruction mutates a register, the mutable register will be
the first register listed and be marked as (dest)


 - **ldi `<register(dest)>` `<constant>`**
 	loads a constant into dest
	```asm
	ldi r1, $1
	```
- **cpy <register(dest)> <register(src)>**
	copies a value from src to dest
 	```asm
	cpy r1,r2
	```
- **load <register(dest)> <register(length)> <register(ptr)>**
 	reads a value from memory starting from ptr to length into dest
 	```asm
 	load r1,r2,r3
	```
- **store <register(ptr)> <register(length> <register(src)>**
	writes the value of src to memory starting at ptr
	and extending for length bytes
	 ```asm
	 store r3,r2,r1
	 ```

## Arithmetic

the following operations of this section all adhere to the following signature

**\<operation> <register(dest)> <register(operand1)> <register(operand2)>**

```asm
add r3,r1,r2
```

standard:
	- *add*
	- *sub*
	- *mult*
	- *div*
	- *mod*

biwise:
	- *and*
	- *or*
	- *xor*

the following operations interpret all registers as floating point
	- *fadd*
	- *fsub*
	- *fmult*
	- *fdiv*
	- *fmod*

- *itof <register(dest)> <register(src)>*
	converts the src integer to the same value in floating point encoding

- *ftoi <register(dest)> <register(src)>*
		converts the src float to the same value in integer encoding
		(actually i think this is useless)


- *neg <register(dest)> <register(operand)>*
	negates a value by flipping its most significant bit
	```asm
	neg r2,r1
	```
- *not <register(dest) <register(operand)>>*
	inverts a value by flipping all its bits
	```asm
	not r2,r1
	```

- *shl <register(dest)> <register(n)> <register(src)>*
	shifts src by n bits left
	```asm
	shl r3,r2,r1
	```

- *shr <register(dest)> <register(n)> <register(src)>*
	shifts src by n bits right
	```asm
	shr r3,r2,r1
	```
- *rotl <register(dest)> <register(n)> <register(src)>*
	shifts src by n bits left (wrapping)
	```asm,
	rotl r3,r2,r1
	```

- *rotr <register(dest)> <register(n)> <register(src)>*
	shifts src by n bits right (wrapping)
	```asm
	rotl r3,r2,r1
	```


- *inc <register(dest/src)>*
	increments the content of register by 1
	```asm
	inc r1
	```
- *dec <register(dest/src)>*
	decrements the content of register by 1
	```asm
	dec r1
	```

control flow
- *jmp <constant>*
	jumps program to a fixed address
	```asm
	jmp $x00aa
	```
	equivelent to
	```
	ldi r1,$00aa
	cpy pc,r1
	```
- *jifz <register(condition)> <constant>*
	jumps to a fixed address if condition is zero
	```asm
	# if r1 == r2
	sub r3b1,r2,r1
	jifz r3b1,$00aa
	```
- *jifnz <register(condition)> <constant>*
 		jumps to a fixed address if condition is not zero
 		```asm
 		# if r1 != r2
 		sub r3b1,r2,r1
 		jifnz r3b1,$00aa
 		```
- *haltexe*
	immediately stop program execution
stack
the stack is 64bit aligned, any value will be stored
 as 64 bits regardless of register source size


- *push <register(src)>*
	pushes src to the stack
	```asm
	push r1
	```

- *pop <register(dest)>*
	pops a value from the stack into dest
	```asm
	pop r2
	```

heap
- *malloc <register(dest)> <register(size)>*
	allocates a block of the heap of size and returns a pointer in dest
	```asm
	malloc r1,r2
	```
- *realloc <register(dest)> <register(ptr)> <register(size)>*
	reallocates ptr to size and returns the new pointer (or same pointer if it expanded)
	```asm
	realloc r1,r1,r2
	```
- *free <register(ptr)>*
	deallocates ptr
	```asm
	free r1
	```
- *memcpy <register(dest_mem)> <register(n)> <register(src)>*
	copies memory from src to dest ptrs with length n

- *memset <register(dest_mem)> <register(n)> <register(src)>*
	sets memory starting from dest_mem to dest_mem+n with src byte


## subroutines

- *call <constant>*
	pushes fp and pc to stack and then jumps to pc
	```asm
	call $x00aa
	```
	equivilent to
	```asm
	push fp
	push pc
	ldi r1,$x00aa
	cpy pc,r1
	```
- *ret*
	pops pc and fp from stack and jumps back to return addr
	```asm
	ret
	```
	equivilent to
	```asm
	pop pc
	pop fp
	```
