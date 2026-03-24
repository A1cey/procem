# Timing

## Results

| exe                       | time (sec)  |
|---------------------------+-------------|
| base                      | 18          |
| u8_slice_and_range        | 8           |
| no_uppercasing            | 6           |
| preallocate_token_vec     | 4           |

## Command

Get the exe: cargo bench --bench tokenizer_perf --no-run
Time it: timeit { procasm/benches/bin/<name>.exe }
Flamegraph: cargo flamegraph --bench tokenizer_perf