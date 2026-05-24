# procasm

**procasm** is a toy Rust library that provides a default implementation of the `Instruction` trait of the **procem** library.

## Instruction set

### Syntax

All assembly is interpreted as ASCII.
All instructions, registers and immediate values can be written in mixed case.
All operations that can be suffixed with an 'S' set the flag registers depending on the operation.

- *Labels* (**\<LABEL>**) are used to mark specific locations in the program. They are denoted by a string of alphanumeric or underscore ('_') or dash ('-') characters followed by a colon (':') (e.g., 'label:'). Labels are case-sensitive.
- *Registers* (**\<REG>**) must be a valid register name (e.g., 'R0', 'r1', 'R2', 'PC', 'sp').
- *Literals* (**\<LIT>**) are decimal, binary, hexadecimal, octal, boolean or char constants.
  - Decimal values start with '0d' (optional), followed by a sequence of '0's through '9's.
  - Binary values start with '0b', followed by a sequence of '0's and '1's.
  - Hexadecimal values start with '0x', followed by a sequence of digits from '0' through '9' and letters from 'a' through 'f'.
  - Octal values start with '0o', followed by a sequence of '0's through '7's.
  - Boolean values are either 'true' or 'false'.
  - Character values are enclosed in single quotes, e.g., 'a', 'B', '5'.
- *Operands* (**\<OP>**) can be a register name or a literal.
- Compiler *directives* are special instructions that the assembler uses to control the assembly process. They start with a '.' followed by a string of alphanumeric or underscore ('_') or dash ('-') characters.
  - There can be three *Sections* in a program: 
    - *.code*: This section is mandatory and contains executable instructions.
    - *.data*: This section is optional and contains data declarations.
    - *.bss*: This section is optional and contains uninitialized data declarations.
    Sections can be in any order and occur multiple times. 
    A valid program must have at least one *.code* section.

'END' marks the end of the program. It is only used as a guide for the assembler and not part of the assembled program.

### Directives

- *.data*: This section is optional and contains data declarations.
- *.bss*: This section is optional and contains uninitialized data declarations.
- *.code*: This section is mandatory and contains executable instructions.
- *.word*: Usable only in *.data* sections. Declares a word-sized data item.
- *.ascii*: Usable only in *.data* sections. Declares an ASCII string.
- *.space*: Usable only in *.bss* sections. Declares a block of memory.

### Data Section

Use *.word* or *.ascii* followed by a *Literal* to declare data in the *.data* section. To declare an array of data multiple literals divided by spaces can be used.

Example:
```
.data
  .word 10, 20, 30, 40, 50
  .ascii "Hello, World!"
```

### Bss Section

Use *.space* followed by a numeric *Literal* (Decimal, Octal, Hexadecimal, Binary) to declare uninitialized data in the *.bss* section. The *Literal* specifies the number of words to allocate.

### Operations

#### **ADD**

Add the value of the operand to the register. The result is stored in the register.

`ADD`\[`S`\] `<REG>` `,` `<OP>`

#### **AND**

Perform a bitwise and operation on the value in the register with the value of the operand.

`AND` `<REG>` `,` `<OP>`

#### **CALL**

Call a subroutine at the program address specified by the operand. Pushes the current program counter onto the stack and sets the program counter to the address of the subroutine.

`CALL` `<OP>`

#### **CMP**

Compare the values of two operands and set the flags accordingly. This is the same as [SUBS](#sub) but disregards the result of the subtraction.

`CMP` `<OP>` `,` `<OP>`

#### **DEC**

Decrement the value in a register by one.

`DEC`\[`S`\] `<REG>`

#### **DIV**

Divide the value of the register by the value of the operand. The result is stored in the register.

`DIV`\[`S`\] `<REG>` `,` `<OP>`

#### **INC**

Increment the value in a register by one.

`INC`\[`S`\] `<REG>`

#### **JC**

Jump to the label if the carry flag (C) is set.

`JC` `<LABEL>`

#### **JG**

Jump to the label if the zero flag (Z) and signed flag (S) are not set.

`JG` `<LABEL>`

#### **JGE**

Jump to the label if the zero flag (Z) is set or signed flag (S) is not set.

`JGE` `<LABEL>`

#### **JL**

Jump to the label if the zero flag (Z) is not set and the signed flag (S) is set.

`JL` `<LABEL>`

#### **JLE**

Jump to the label if the zero flag (Z) or signed flag (S) is set.

`JLE` `<LABEL>`

#### **JMP**

Set program counter to the address of the label (first instruction after the label), effectively jumping to the instruction at this point in the program.

`JMP` `<LABEL>`

#### **JNC**

Jump to the label if the carry flag (C) is not set.

`JNC` `<LABEL>`

#### **JNS**

Jump to the label if the signed flag (S) is not set.

`JN` `<LABEL>`

#### **JNZ**

Jump to the label if the zero flag (Z) is not set.

`JNZ` `<LABEL>`

#### **JS**

Jump to the label if the signed flag (S) is set.

`JS` `<LABEL>`

#### **JZ**

Jump to the label if the zero flag (Z) is set.

`JZ` `<LABEL>`

#### **MOV**

Copy a value from the operand to the register.

`MOV` `<REG>` `,` `<OP>`

#### **MUL**

Multiply the value of the operand with the value of the register. The result is stored in the register.

`MUL`\[`S`\] `<REG>` `,` `<OP>`

#### **NOP**

No operation.

`NOP`

#### **NOT**

Perform a bitwise not operation on the value in the register.

`NOT` `<REG>`

#### **OR**

Perform a bitwise or operation on the value in the register with the value of the operand.

`OR` `<REG>` `,` `<OP>`

#### **POP**

Pop a value from the stack to the register.

`POP` `<REG>`

#### **PUSH**

Push a value from the operand to the stack.

`PUSH` `<OP>`

#### **RET**

Return from a subroutine. Pops the return address from the stack and sets the program counter to the popped value.

`RET`

#### **ROL**

Rotate the value in the register left by the specified number of bits. Only use values between 1 and the number of bits of the Word size minus 1.

`ROL` `<REG>` `,` `<LIT>`

#### **ROR**

Rotate the value in the register right by the specified number of bits. Only use values between 1 and the number of bits of the Word size minus 1.

`ROR` `<REG>` `,` `<LIT>`

#### **SHL**

Shift the value in the register left by the specified number of bits. Only use values between 1 and the number of bits of the Word size minus 1.

`SHL` `<REG>` `,` `<LIT>`

#### **SHR**

Shift the value in the register right by the specified number of bits. Only use values between 1 and the number of bits of the Word size minus 1.

`SHR` `<REG>` `,` `<LIT>`

#### **SUB**

Subtract the value of the operand from the register. The result is stored in the register.

`SUB`\[`S`\] `<REG>` `,` `<OP>`

#### **XOR**

Perform a bitwise xor operation on the value in the register with the value of the operand.

`XOR` `<REG>` `,` `<OP>`


## Usage

To assemble a program from assembly code use the **assemble** function.

### Example

```rust
use procem::{processor::Processor, register::Register, word::I32};
use procasm::assemble;

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
const MEM_SIZE: usize = 1024;

let mut processor = Processor::<MEM_SIZE, _, _, _>::builder()
    .with_program(&program)
    .build();

let _ = processor.run_program();

// Inspect register values
assert_eq!(processor.registers.get_reg(Register::R0), 6.into());
```

