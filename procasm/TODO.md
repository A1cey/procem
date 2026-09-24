- Add Comment syntax //,# ??
- Move counted_enum into ars
- Add more tests

**Parser Findings**
3. [parser.rs:101]: alternative parsing discards useful errors. `wat r0` reports “expected Newline” instead of “unknown mnemonic.” Preserve the furthest or most committed error.

4. [components.rs:23]: diagnostics frequently record `state.idx` after consuming the offending token. Capture the token index before parsing.

The largest parser weakness is that malformed input is not yet reliably panic-free.

**Project Findings**
1. [linker.rs:53]: symbols have no section/type. Consequently `jmp data_label` is accepted, `_start` can be non-code, and `adr` cannot distinguish code addresses from data addresses.

2. [linker.rs:142]: BSS symbols remain section-relative. With one byte of data, the first BSS symbol still resolves to address `0`, not `1`.

3. `Instruction` has no assembly formatter, while the parser and linker are private. An LLVM backend must either build text manually or duplicate linking. Expose a structured assembly module with typed symbols and relocations, plus a canonical emitter.

4. There is no normal termination instruction. `run_program()` only ends with an error such as `PCOutOfBounds`. Add `HALT`/`EXIT` semantics before compiling complete programs.

**Required Before LLVM Lowering**
1. Define a target specification: 64-bit little-endian pointers/registers, stack layout and alignment, integer widths, memory map, entry point, and unsupported LLVM features.
2. Define an ABI: argument and return registers, caller/callee-saved registers, spill slots, frame layout, direct/indirect calls, and symbol visibility.
3. Implement typed symbols and relocations for code, data, BSS, function calls, and pointer-valued global initializers.
4. Fix signed branches, image bounds, stack alignment, parser panics, and BSS relocation.
5. Add a public assembly IR and renderer. The LLVM translator should target this IR, not concatenate assembly strings.
6. Add backend machinery: SSA destruction/PHI copies, instruction legalization, register allocation with spilling, stack frames, and block-label generation.
7. Initially support integers `i1/i8/i16/i32/i64`, arithmetic, casts, `icmp`, branches, load/store, `alloca`, GEP, calls, returns, and globals. Explicitly reject floats, vectors, atomics, exceptions, varargs, and integers over 64 bits.
8. Add differential tests against `lli`: LLVM IR → procasm → procem, comparing return values and memory.

Inkwell is suitable for loading and inspecting verified LLVM modules, but the missing work is primarily target definition and lowering infrastructure. I would not begin instruction selection until items 1–5 are settled.
