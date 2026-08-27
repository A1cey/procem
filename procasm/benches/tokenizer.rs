use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use dev_utilities::generator::generate_asm;
use procasm::tokenizer::Tokenizer;

fn small_program(c: &mut Criterion) {
    // 100 instructions
    const LABEL_COUNT: usize = 10;
    const INST_PER_LABEL: usize = 10;
    const REGISTER_COUNT: usize = 16;
    let mut input = generate_asm(LABEL_COUNT, INST_PER_LABEL, REGISTER_COUNT).bytes().collect::<Vec<_>>();
    c.bench_function("tokenize_small", |b| b.iter(|| Tokenizer::tokenize(black_box(&mut input))));
}

fn medium_program(c: &mut Criterion) {
    // 1000 instructions
    const LABEL_COUNT: usize = 100;
    const INST_PER_LABEL: usize = 10;
    const REGISTER_COUNT: usize = 16;
    let mut input = generate_asm(LABEL_COUNT, INST_PER_LABEL, REGISTER_COUNT).bytes().collect::<Vec<_>>();

    c.bench_function("tokenize_medium", |b| b.iter(|| Tokenizer::tokenize(black_box(&mut input))));
}

fn large_program(c: &mut Criterion) {
    // 10_000 instructions
    const LABEL_COUNT: usize = 1000;
    const INST_PER_LABEL: usize = 10;
    const REGISTER_COUNT: usize = 16;
    let mut input = generate_asm(LABEL_COUNT, INST_PER_LABEL, REGISTER_COUNT).bytes().collect::<Vec<_>>();

    c.bench_function("tokenize_large", |b| b.iter(|| Tokenizer::tokenize(black_box(&mut input))));
}

fn very_large_program(c: &mut Criterion) {
    // 100_000 instructions
    const LABEL_COUNT: usize = 10_000;
    const INST_PER_LABEL: usize = 10;
    const REGISTER_COUNT: usize = 16;
    let mut input = generate_asm(LABEL_COUNT, INST_PER_LABEL, REGISTER_COUNT).bytes().collect::<Vec<_>>();

    c.bench_function("tokenize_very_large", |b| b.iter(|| Tokenizer::tokenize(black_box(&mut input))));
}

criterion_group!(benches, small_program, medium_program, large_program, very_large_program);
criterion_main!(benches);
