- Add Comment syntax //,# ??
- Move counted_enum into ars
- Add more tests

**Parser Findings**
1. 🔴 [tokenizer.rs:374](https://github.com/A1cey/procem/blob/23f51ee9b95b72128ccb45827386b8e0dc5cb558/procasm/src/tokenizer.rs#L374): valid literals panic when `0` is followed by punctuation. `.byte 0, 1` reaches `todo!()`. Treat standalone zero as decimal regardless of delimiter.

2. 🔴 [tokenizer.rs:285](https://github.com/A1cey/procem/blob/23f51ee9b95b72128ccb45827386b8e0dc5cb558/procasm/src/tokenizer.rs#L285): unterminated strings read past the input and panic. Bound the scan and return `TokenizerError`.

3. 🟡 [parser.rs:101](https://github.com/A1cey/procem/blob/23f51ee9b95b72128ccb45827386b8e0dc5cb558/procasm/src/parser.rs#L101): alternative parsing discards useful errors. `wat r0` reports “expected Newline” instead of “unknown mnemonic.” Preserve the furthest or most committed error.

4. 🟡 [components.rs:23](https://github.com/A1cey/procem/blob/23f51ee9b95b72128ccb45827386b8e0dc5cb558/procasm/src/parser/components.rs#L23): diagnostics frequently record `state.idx` after consuming the offending token. Capture the token index before parsing.

5. 🟡 Parser combinators restore only `idx`, not the complete `ParserState`. This is fragile because parsers also mutate labels, data, BSS, and unresolved instructions. Either make alternatives side-effect-free or checkpoint/restore all state.

The combinator structure itself is compact and readable, and ownership/lifetime use is sensible. The largest parser weakness is that malformed input is not yet reliably panic-free.

**Project Findings**
1. 🔴 [linker.rs:53](https://github.com/A1cey/procem/blob/23f51ee9b95b72128ccb45827386b8e0dc5cb558/procasm/src/linker.rs#L53): symbols have no section/type. Consequently `jmp data_label` is accepted, `_start` can be non-code, and `adr` cannot distinguish code addresses from data addresses.

2. 🔴 [linker.rs:142](https://github.com/A1cey/procem/blob/23f51ee9b95b72128ccb45827386b8e0dc5cb558/procasm/src/linker.rs#L142): BSS symbols remain section-relative. With one byte of data, the first BSS symbol still resolves to address `0`, not `1`.

3. 🔴 Static data+BSS size is not checked against `MEM_SIZE`. Assembly succeeds, then loading the resulting program panics in [processor.rs:75](https://github.com/A1cey/procem/blob/23f51ee9b95b72128ccb45827386b8e0dc5cb558/procem/src/processor.rs#L75).

4. 🔴 [jump_condition.rs:48](https://github.com/A1cey/procem/blob/23f51ee9b95b72128ccb45827386b8e0dc5cb558/procasm/src/instruction/jump_condition.rs#L48): signed comparisons ignore overflow. Signed less-than must use $N \ne V$; signed greater-than must use $Z=0 \land N=V$. A probe showed `i64::MIN > 1` branching as true.

5. 🟡 `Instruction` has no assembly formatter, while the parser and linker are private. An LLVM backend must either build text manually or duplicate linking. Expose a structured assembly module with typed symbols and relocations, plus a canonical emitter.

6. 🟡 There is no normal termination instruction. `run_program()` only ends with an error such as `PCOutOfBounds`. Add `HALT`/`EXIT` semantics before compiling complete programs.

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
