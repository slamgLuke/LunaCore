# LunaCore Architecture

![LunaCore Schematic](https://github.com/slamgLuke/LunaCore/blob/main/schematics/LunaCore_schematic.png)

## What is LunaCore?

LunaCore is a small and simple RISC architecture built from scratch, fully packed with features despite being in a limited 16-bit size.

## Architecture Details

Single Cycle Processor.

Eight 16-bit Registers
- 4 General Purpose (T0, T1, T2, T3)
- Base Pointer and Stack Pointer (BP, SP)
- Program Counter (PC)
- Read-Only Input Register (IN)

16-bit instruction width, with optional 32-bit instructions with a Wide 16-bit Immediate.

## Instruction Set

**See the following spreadsheet** for more details on the ISA Format, Decoding and Calling Convention:
[LunaCore ISA](https://docs.google.com/spreadsheets/d/1sSNFZnan8-FoNpeGK8PKvXXWJTQ9Smjf0fxBcI3Iy6I/edit?usp=sharing)

### 1. Data Processing `(DP)`

The following Data Processing instructions are supported:

| Instruction | CMD | Syntax | Description |
|-|-|-|-|
| ADD | 000 | `ADD Td, Tn, Tm/!imm` | `Td = Tn + Tm/!imm` |
| SUB | 001 | `SUB Td, Tn, Tm/!imm` | `Td = Tn - Tm/!imm` |
| AND | 010 | `AND Td, Tn, Tm/!imm` | `Td = Tn & Tm/!imm` |
| OR | 011 | `OR Td, Tn, Tm/!imm` | `Td = Tn \| Tm/!imm` |
| XOR | 100 | `XOR Td, Tn, Tm/!imm` | `Td = Tn ^ Tm/!imm` |
| MOV | 101 | `MOV Td, Tm/!imm` | `Td = Tm/!imm` |
| SHL | 110 | `SHL Td, Tn, Tm/!imm` | `Td = Tn << (Tn/!imm)[3:0]` |
| SHR | 111 | `SHR Td, Tn, Tm/!imm` | `Td = Tn >> (Tn/!imm)[3:0]` |

**Note:** The `NOT` instruction can be done with `XOR Tx, !-1`.

Only DP operations can change the Conditional Unit Flags `NZCV` (Negative, Zero, Carry, Overflow).

Every DP instruction overwrites the flags.

`CMP` or `TST` instructions can be done with `SUB` or `AND` with the destination being the read-only `IN` register.


### 2. Memory Instructions `(MEM)`

- `SAV{B}`: Stores Reg in Memory
- `LOD{B}`: Loads from Memory to Reg 
- `PUSH{B}` Pushes a Reg/Immediate to Stack, and decreases SP
- `POP{B}` Pops from Stack to Register and increases SP

Each instruction reads/writes a 16-bit word from/to memory, and has a `B` counterpart to use a single byte.

### 3. Branching / Jumping `(BRANCH)`

`JMP` Instructions have 16 different conditionals.

The Conditional Unit validates if the `PC` should be overwritten or not depending on the current Flags.

The new Program Counter is offseted by the value of the Immediate in the instruction.

Each instruction can either have a 9-bit Sign-Extended Immediate, or a 16-bit Wide Immediate.
