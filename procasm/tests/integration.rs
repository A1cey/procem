use core::panic;

use ars::fmt::slice::FmtSlice;
use procasm::{
    AssembledProgram, assemble,
    instruction::{Instruction, jump_condition::JumpCondition, operand::Operand},
};
use procem::{
    processor::Processor,
    program::{Bss, Code, Data, Header, Program},
    register::Register,
    word::I32,
};

#[test]
fn simple_5x2_multiplication() {
    const MEM_SIZE: usize = 1024;

    let program = assemble::<MEM_SIZE, I32>(
        "
        .code
        _start:
            mov R0, 2
            add R1, R0
            jmp _start
        ",
    );
    let program = match program {
        Ok(program) => program,
        Err(err) => panic!("{}", FmtSlice(&err)),
    };

    assert_eq!(
        program,
        AssembledProgram::<MEM_SIZE, I32>::new(
            Header::new(I32::from(0), I32::from((MEM_SIZE - 1) as i32)),
            Data::default(),
            Bss::default(),
            Code::from(vec![
                Instruction::Mov {
                    to: Register::R0,
                    from: Operand::Value(2.into())
                },
                Instruction::Add {
                    acc: Register::R1,
                    rhs: Operand::Register(Register::R0),
                    signed: false
                },
                Instruction::Jump {
                    to: 0.into(),
                    condition: JumpCondition::Unconditional
                }
            ])
        )
    );

    let mut processor = Processor::builder().with_program(&program).build();

    println!("{processor}");

    for _ in 0..14 {
        assert!(processor.execute_next_instruction().is_ok());
    }

    assert_eq!(processor.registers.get_reg(Register::R1), 10.into());
    assert_eq!(processor.registers.pc(), 2.into());

    assert!(processor.execute_next_instruction().is_ok());
    assert_eq!(processor.registers.pc(), 0.into());
}

#[test]
fn parse_various_literals() {
    const MEM_SIZE: usize = 1024;
    let program = assemble::<MEM_SIZE, I32>(
        "
        .code
        _start:
            mov R0, 42
            mov R1, 0b101010
            mov R2, 0x2A
            mov R3, 0o52
            mov R6, 'A'
        ",
    );
    let program = match program {
        Ok(program) => program,
        Err(err) => panic!("{}", FmtSlice(&err)),
    };

    assert_eq!(program.code().len(), 5);
    assert_eq!(
        program,
        Program::new(
            Header::new(I32::from(0), I32::from((MEM_SIZE - 1) as i32)),
            Data::default(),
            Bss::default(),
            Code::from(vec![
                Instruction::Mov {
                    to: Register::R0,
                    from: Operand::Value(42.into())
                },
                Instruction::Mov {
                    to: Register::R1,
                    from: Operand::Value(42.into())
                },
                Instruction::Mov {
                    to: Register::R2,
                    from: Operand::Value(42.into())
                },
                Instruction::Mov {
                    to: Register::R3,
                    from: Operand::Value(42.into())
                },
                Instruction::Mov {
                    to: Register::R6,
                    from: Operand::Value(65.into())
                }
            ])
        )
    )
}

#[test]
fn parse_and_execute_arithmetic() {
    const MEM_SIZE: usize = 1024;
    let program = assemble::<MEM_SIZE, I32>(
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
    );
    let program = match program {
        Ok(program) => program,
        Err(err) => panic!("{}", FmtSlice(&err)),
    };

    let mut processor = Processor::builder().with_program(&program).build();

    let _ = processor.run_program();

    assert_eq!(processor.registers.get_reg(Register::R0), 6.into());
}

#[test]
fn control_flow_and_labels() {
    const MEM_SIZE: usize = 1024;
    // Loop should run 5 times, incrementing R0 from 0 to 5
    let program = assemble::<MEM_SIZE, I32>(
        "
.code
_start:
    mov R0, 0
    mov R1, 5
loop:
    add R0, 1
    subs R1, 1
    jnz loop
",
    );
    let program = match program {
        Ok(program) => program,
        Err(err) => panic!("{}", FmtSlice(&err)),
    };

    let mut processor = Processor::builder().with_program(&program).build();

    let _ = processor.run_program();
    assert_eq!(processor.registers.get_reg(Register::R0), 5.into());
}

#[test]
fn test_overflow_and_flags() {
    const MEM_SIZE: usize = 1024;
    let program = assemble::<MEM_SIZE, I32>(
        "
        .code
        _start:
            mov R0, 2147483647
            add R0, 1
            cmp R0, -2147483648
        ",
    );
    let program = match program {
        Ok(program) => program,
        Err(err) => panic!("{}", FmtSlice(&err)),
    };

    let mut processor = Processor::builder().with_program(&program).build();

    let _ = processor.run_program();
    assert_eq!(processor.registers.get_reg(Register::R0), i32::MIN.into());
    assert_eq!(processor.registers.get_flag(procem::register::Flag::Z), true);
}

#[test]
fn factorial_program() {
    const MEM_SIZE: usize = 1024;
    let program = assemble::<MEM_SIZE, I32>(
        "
        .code
        _start:
            mov R0, 5
            mov R1, 1
        loop:
            mul R1, R0
            subs R0, 1
            jnz loop
        ",
    );
    let program = match program {
        Ok(program) => program,
        Err(err) => panic!("{}", FmtSlice(&err)),
    };

    let mut processor = Processor::builder().with_program(&program).build();

    let _ = processor.run_program();
    assert_eq!(processor.registers.get_reg(Register::R1), 120.into());
}

#[test]
fn swap_static_mem() {
    const MEM_SIZE: usize = 32;
    let program = assemble::<MEM_SIZE, I32>(
        "
        .data
        foo:
            .word 42, 43
        .code
        _start:
            adr r0, foo
            ldr r1, foo
            ldr r2, [r0, 1]
            str r2, foo
            str r1, [r0, 1]
        ",
    )
    .map_err(|err| panic!("{}", FmtSlice(&err)))
    .unwrap();

    let mut processor = Processor::builder().with_program(&program).build();

    assert_eq!(processor.registers.get_reg(Register::R0), 0.into());
    assert_eq!(processor.registers.get_reg(Register::R1), 0.into());
    assert_eq!(processor.mem.read(program.data().base_addr()), 42.into());
    assert_eq!(processor.mem.read(program.data().base_addr() + 1.into()), 43.into());
    assert_eq!(program.code().len(), 5);

    let _ = processor.run_program();

    assert_eq!(processor.registers.get_reg(Register::R0), 0.into());
    assert_eq!(processor.registers.get_reg(Register::R1), 42.into());
    assert_eq!(processor.registers.get_reg(Register::R2), 43.into());
    assert_eq!(processor.mem.read(program.data().base_addr()), 43.into());
    assert_eq!(processor.mem.read(program.data().base_addr() + 1.into()), 42.into());
}
