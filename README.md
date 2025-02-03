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
    [SIGNATURE][DATA_LENGTH][PROGRAM_LENGTH][DATA][PROGRAM]
  ## runtime memory map:
    [MMIO][PROGRAM][DATA]
# Shell Commands
  - `load <binary file path>`
    > loads the binary file into the virtual machine
  - `exec`
    > executes the program
  - `pr-reg <register>`
    > prints <register> in decimal and hexidecimal
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
