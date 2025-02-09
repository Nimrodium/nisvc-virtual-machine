# Nim Instruction Set Virtual Computer Architecture

A 64-bit RISC CPU with Harvard architecture, featuring 20 general purpose registers, 12 special registers, and MMIO support.

## Architecture Specifications
  - Endianness: Little-endian
  - Register Width: 64 bits
  - Memory: Harvard Architecture
  - Word Size: 64 bit

### Registers
  - General Purpose (r1-r20): 64-bit general use registers
  - Special Purpose:
    - pc: Program Counter
    - sp: Stack Pointer
    - o1-o10: argument registers


# Memory Layout
  ## binary image:
    [SIGNATURE][PROGRAM_LENGTH][DATA_LENGTH][PROGRAM][DATA]
  ## runtime memory map:
    [MMIO][PROGRAM][DATA]
# Shell Commands
  - `ls <dir>`
  	> lists contents of dir. defaults to . if no dir is specified
  - `info`
    > prints revision and architecture version
  - `load <binary file path>`
    > loads the binary file into the virtual machine
  - `exec`
    > executes the loaded binary
  - `pr-reg <register>`
    > prints register in decimal and hexidecimal
  - `dump <program/ram>`
    > dumps program rom or data ram
  - `reset`
    > resets program to be executed again
  - `exit`
    > exits shell


# instruction Set
  - mov
  - movim
  - load
  - store
  - add
  - sub
  - mult
  - div
  - or
  - xor
  - and
  - not
  - shl
  - shr
  - rotl
  - rotr
  - neg
  - jmp
  - jifnz
  - jifz
  - pr
  - inc
  - dec
  - push
  - pop
  - call
  - ret

# Assembly Syntax
- sections are enclosed in brackets
- comments are denoted by two backslashes `//`
- instructions may be seperated by a newline or semicolon (inline code is allowed)
- literals are prefixed with `$`
- addresses relative to ram_base are prefixed with `@`
- labels are prefixed with`!`

all operations are register based, the only instructions which accepts a non-register are `movim` and the jump instructions

```assembly
data {
	// data stuff here
}

program {
	// 	program instructions here
	movim r1,$10
}
```
