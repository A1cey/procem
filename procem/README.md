# procem

**procem** is a toy Rust library that provides a flexible processor emulator, loosely inspired by the ARM architecture. It allows you to define and execute custom instruction sets, manage registers, flags, and memory, and run assembly-like programs.

## Features

- [`Processor`](src/processor.rs): Emulates a processor with general-purpose registers, program counter, stack pointer, flags, and memory.
- [`Program`](src/program.rs): Container for a sequence of instructions to be executed by the processor.
- [`Instruction`](src/instruction.rs): Trait for defining custom instruction sets. A default instruction set is implemented in the procasm crate.
- [`Registers`](src/register.rs): General-purpose registers, program counter, stack pointer, and flags.
- [`Memory`](src/memory.rs): Fixed-size memory for processor operations.
- [`Word`](src/word.rs): Trait for word-size types. Word is already implemented for all signed integer types.

## Customization

You can implement your own instruction set by implementing the `Instruction` trait, and support custom word types by implementing the `Word` trait. Alternatively, you can use the default instruction set and word types.

### Example: Using procasm

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

### Example: Custom Instruction Set

```rust
use procem::{instruction::Instruction,processor::Processor, program::Program, register::Register,  word::Word};

// Define your own word type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct MyWord(i64);

impl Word for MyWord {
    // Implement required methods...
}

// Define your own instruction set
enum MyInstruction {
    Mov { to: Register, from: MyWord },
    // Add more instructions...
}

impl Instruction<MyWord> for MyInstruction {
    // Implement required methods...
}

// Create a program and processor using your custom types
let program = Program::new(vec![
    MyInstruction::Mov { to: Register::R0, from: MyWord(42) },
    // Add more instructions...
]);

let mut processor = Processor::<128, _, _, _>::builder()
    .with_program(&program)
    .build();
```
