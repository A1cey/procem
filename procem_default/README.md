# procem_default

**procem_default** is a toy Rust library that provides a default implementation of the `Instruction` trait of the **procem** library.

## Instruction set

### Syntax

All assembly is interpreted as ASCII. 
All instructions can be written in mixed case.
All operations that can be suffixed with an 'S', set the flag registers depending on the operation.

- *Labels* (**\<LABEL>**) are used to mark specific locations in the program. They are denoted using a dot ('.') followed by a string (e.g., '.label').
- *Registers* (**\<REG>**) must be a valid register name (e.g., 'R0', 'r1', 'R2', 'PC', 'sp').
- *Literals* (**\<LIT>**) are decimal, binary, hexadecimal, octal, boolean or char constants.
  They are denoted using a '#' followed by a valid literal value.
  - Decimal values start with '0d' (optional), followed by a sequence of '0's through '9's.
  - Binary values start with '0b', followed by a sequence of '0's and '1's.
  - Hexadecimal values start with '0x', followed by a sequence of digits from '0' through '9' and letters from 'a' through 'f'.
  - Octal values start with '0o', followed by a sequence of '0's through '7's.
  - Boolean values are either 'true' or 'false'.
  - Character values are enclosed in single quotes, e.g., 'a', 'B', '5'.
- *Operands* (**\<OP>**) can be a register name or a literal.

'END' marks the end of the program. It is only used as a guide for the assembler and not part of the assembled program.

### Operations

- **NOP**: No operation.
- **MOV \<REG>, \<OP>**: Copy a value from the operand to the register.
- **PUSH \<OP>**: Push a value from the operand to the stack.
- **POP \<REG>**: Pop a value from the stack to the register.
- **CALL \<OP>**: Call a subroutine at the program address specified by the operand. Pushes the current program counter onto the stack and sets the program counter to the address of the subroutine.
- **RET**: Return from a subroutine. Pops the return address from the stack and sets the program counter to the popped value.
- **ADD\[S] \<REG>, \<OP>**: Add the value of the operand to the register. The result is stored in the register.
- **SUB\[S] \<REG>, \<OP>**: Subtract the value of the operand from the register. The result is stored in the register.
- **MUL\[S] \<REG>, \<OP>**: Multiply the value of the operand with the value of the register. The result is stored in the register.
- **DIV\[S] \<REG>, \<OP>**: Divide the value of the register by the value of the operand. The result is stored in the register.
- **INC\[S] \<REG>**: Increment the value in a register by one.
- **DEC\[S] \<REG>**: Decrement the value in a register by one.
- **JMP \<LABEL>**: Set program counter to the address of the label (first instruction after the label), effectively jumping to the instruction at this point in the program.
- **JZ \<LABEL>**: Jump to the label if the zero flag (Z) is set.
- **JNZ \<LABEL>**: Jump to the label if the zero flag (Z) is not set.
- **JC \<LABEL>**: Jump to the label if the carry flag (C) is set.
- **JNC \<LABEL>**: Jump to the label if the carry flag (C) is not set.
- **JS \<LABEL>**: Jump to the label if the signed flag (S) is set.
- **JNS \<LABEL>**: Jump to the label if the signed flag (S) is not set.
- **JG \<LABEL>**: Jump to the label if the zero flag (Z) and signed flag (S) are not set.
- **JGE \<LABEL>**: Jump to the label if the zero flag (Z) is set or signed flag (S) is not set.
- **JL \<LABEL>**: Jump to the label if the zero flag (Z) is not set and the signed flag (S) is set.
- **JLE \<LABEL>**: Jump to the label if the zero flag (Z) or signed flag (S) is set.
- **CMP \<OP>, \<OP>**: Compare the values of two operands and set the flags accordingly. This is the same as `SUBS` but disregards the result of the subtraction.
- **XOR \<REG>, \<OP>**: Perform a bitwise xor operation on the value in the register with the value of the operand.
- **AND \<REG>, \<OP>**: Perform a bitwise and operation on the value in the register with the value of the operand.
- **OR \<REG>, \<OP>**: Perform a bitwise or operation on the value in the register with the value of the operand.
- **NOT \<REG>**: Perform a bitwise not operation on the value in the register.
- **SHL \<REG>, \<LIT>**: Shift the value in the register left by the specified number of bits. Only use values between 1 and the number of bits of the Word size minus 1.
- **SHR \<REG>, \<LIT>**: Shift the value in the register right by the specified number of bits. Only use values between 1 and the number of bits of the Word size minus 1.
- **ROL \<REG>, \<LIT>**: Rotate the value in the register left by the specified number of bits. Only use values between 1 and the number of bits of the Word size minus 1.
- **ROR \<REG>, \<LIT>**: Rotate the value in the register right by the specified number of bits. Only use values between 1 and the number of bits of the Word size minus 1.

### Usage
To assemble a program from assembly code use the **assemble** function.

### Example

```rust
use procem::{processor::Processor, register::Register, word::I32};
use procem_default::assemble;

// Assemble a program from asm
let program = assemble::<I32>(
    "
    mov R0, #10
    mov R1, #5
    add R0, R1
    sub R0, #3
    mul R0, #2
    div R0, #4
    "
).unwrap();

// Create a processor and run the program
const STACK_SIZE: usize = 1024;

let mut processor = Processor::<STACK_SIZE, _, _, _>::builder()
    .with_program(&program)
    .build();

let _ = processor.run_program();

// Inspect register values
assert_eq!(processor.registers.get_reg(Register::R0), 6.into());
```