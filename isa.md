nop - opcode (0x00) : 1 byte
	nop
	Does not perform an operation.

Data Moving
cpy - opcode (0x01) : 3 bytes
	cpy arx brx
		Copies value from brx to arx

ldi - opcode (0x02) : 3+<=8 bytes
ldi rx $
		Loads immediate $ into rx.

load - opcode (0x03) : 11 bytes
	load arx brx @rx
		Loads value starting at memory address @rx and extending for brx	into arx.

store - opcode (0x04) : 11 bytes
	store @rx arx brx
	Stores value of brx to @rx writing up to arx

Arithmetic
add - opcode (0x05) : 4 bytes
	add arx brx crx
		Adds crx to brx and stores result in arx (brx+crx=arx)

sub - opcode (0x06) : 4 bytes
	sub arx brx crx
		Subtracts crx from brx and stores result in arx (brx-crx=arx)

mult - opcode (0x07) : 4 bytes

div - opcode (0x08) : 4 bytes
	div arx brx crx
		Divides brx by crx and stores the quotient in arx

Bitwise Arithmetic
or - opcode (0x09) : 4 bytes
	or arx brx crx
		Performs bitwise OR on brx and crx, and stores the result in arx

xor - opcode (0x0a) : 4 bytes
	xor arx brx crx
		Performs bitwise xor on brx and crx, and stores the result in arx

and - opcode (0x0b) : 4 bytes
	and arx brx crx
		Performs bitwise and on brx and crx and stores the result in arx

not - opcode (0x0c) : 3 bytes
	not arx brx
		Performs bitwise not on brx and stores the result in arx

shl - opcode (0x0d) : 4 bytes
	shl arx brx crx
		Performs bitwise shift on brx by crx places to the left, and stores the result in arx

shr - opcode (0x0e) : 4 bytes
	shl arx brx crx
		Performs bitwise shift on brx by crx places to the right, and stores the result in arx

rotl - opcode (0x0f) : 4 bytes
	rotl arx brx crx
		Performs a bitwise rotation of brx by crx places to the left, and stores the result in arx
rotr - opcode (0x10) : 4 bytes
	rotr arx brx crx
		Performs a bitwise rotation of brx by crx places to the right, and stores the result in arx
neg - opcode (0x11) : 3 bytes
	neg arx brx
		Performs a bitwise negation on brx and stores the result in arx

Control Flow
jmp - opcode (0x12)
jifz - opcode (0x13)
jifnz - opcode (0x14)

–

inc - opcode (0x16)
dec - opcode (0x17)

Stack
push - opcode (0x18)
pop - opcode (0x19)

Subroutine calling and returning
call - opcode (0x1a)
ret - opcode (0x1b)
Host File IO
fopen - opcode (0x1e)
fread - opcode (0x1f)
fwrite - opcode (0x20)
fseek - opcode (0x21)
fclose - opcode (0x22)
Special
breakpoint - opcode (0xfe) : 1 byte
	breakpoint
		Drops into debug shell unless breakpoints are set to ignored

haltexe - opode (0xff) : 1 byte
	haltexe
		Immediately halts execution of program, implicitly placed at the end of program by the assembler to ensure data is not executed.
