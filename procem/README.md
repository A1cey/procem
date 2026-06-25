# procem

**procem** is a Rust library that provides a flexible processor emulator, loosely inspired by the ARM architecture. It allows you to define and execute custom instruction sets, manage registers, flags, and memory, and run assembly-like programs.

## Features

- [`Processor`](src/processor.rs): Emulates a processor with general-purpose registers, program counter, stack pointer, flags, and memory.
- [`Program`](src/program.rs): Container for a sequence of instructions to be executed by the processor.
- [`Instruction`](src/instruction.rs): Trait for defining custom instruction sets. A default instruction set is implemented in the *procasm* crate.
- [`Registers`](src/register.rs): General-purpose registers, program counter, stack pointer, and flags.
- [`Memory`](src/memory.rs): Fixed-size memory for processor operations.

## Customization

You can implement your own instruction set by implementing the `Instruction` trait. Alternatively, you can use the default instruction set and word types.

### Example: Using procasm

```rust
use procem::{processor::Processor, register::Register};
use procasm::assemble;

// Assemble program from procasm
const MEM_SIZE: usize = 1024;
let program = assemble::<MEM_SIZE>(
    "
    .code
    _start:
        mov R0, 10
        mov R1, 5
        add R0, R1
        sub R0, 3
        mul R0, 2
        div R0, 4
    ",
).unwrap();

let mut processor = Processor::builder().with_program(&program).build();

let _ = processor.run_program();

assert_eq!(processor.registers.get_reg(Register::R0), 6);
```
