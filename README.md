# Nim Instruction Set Virtual Computer Architecture

a 64bit RISC CPU with 20 general purpose registers and 12 special registers
using a harvard architecture and support for MMIO

# Memory Layout
binary image:
  ```
  [SIGNATURE][DATA_LENGTH][PROGRAM_LENGTH][DATA][PROGRAM]
  ```
runtime memory map:
  ```
  [MMIO][PROGRAM][DATA]
  ```
# instruction Set
- mov
- movim
- load
- store
- add
- sub
- mult
- div
