use std::hint::black_box;

use dev_utilities::generator::generate_asm;
use procem_default::tokenizer::Tokenizer;

fn main() {
    // 1_000_000 instructions
    const LABEL_COUNT: usize = 100_000;
    const INST_PER_LABEL: usize = 10;
    const REGISTER_COUNT: usize = 16;
    let input = generate_asm(LABEL_COUNT, INST_PER_LABEL, REGISTER_COUNT)
        .bytes()
        .collect::<Vec<_>>();

    for _ in 0..100 {
        let mut input = input.clone();
        let _a = Tokenizer::tokenize(black_box(&mut input));
    }
}
