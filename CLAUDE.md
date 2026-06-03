# CLAUDE.md

Toy EVM interpreter in Rust. Full exercise spec is in EXERCISE.md — read it only if you need exercise details.

- Opcode impls + bytecode: src/lib.rs. Tests: tests/exercises.rs.
- `self.pop()` returns Result<U256, Option<ExecutionResult>>; `?` propagates StackUnderflow.

## When I paste a test failure
- The failure is given. Do NOT re-run `cargo test` to confirm it.
- Read ONLY the function under test, not all of lib.rs.
- For bytecode bugs, trace the stack opcode-by-opcode in your reply. Don't run the binary.

## Common loop/bytecode bugs here
- Inverted JUMPI condition (body never runs → storage stays 0)
- Counter not DUP'd before the LT comparison
- SWAP/DUP index off by one (n = opcode - 0x80 + 1, etc.)
- pc not advanced past PUSH data, or pc += 1 not skipped after a taken JUMP

Be concise. Skip preamble.