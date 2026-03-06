use std::fmt::Write as _;

#[must_use]
pub fn generate_asm(label_count: usize, inst_per_label: usize, register_count: usize) -> String {
    let mut asm = String::new();

    for l in 1..=label_count {
        let _ = writeln!(asm, ".L{l}");

        for i in 0..inst_per_label - 1 {
            let _ = writeln!(asm, "mov R{}, #{}", i % register_count, (i + 1) * l);
        }

        let _ = writeln!(asm, "jmp .L{l}");
    }

    asm
}
