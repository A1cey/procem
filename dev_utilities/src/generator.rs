pub fn generate_asm(label_count: usize, inst_per_label: usize, register_count: usize) -> String {
    let mut asm = String::new();

    for l in 1..label_count + 1 {
        asm.push_str(&format!(".L{l}\n"));

        for i in 0..inst_per_label - 1 {
            asm.push_str(&format!("mov R{}, #{}\n", i % register_count, (i + 1) * l));
        }

        asm.push_str(&format!("jmp .L{l}"));
    }

    asm
}
