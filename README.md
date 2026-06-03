# Toy EVM Interpreter — Week 1 Exercise

## Goal

Build a minimal Ethereum Virtual Machine interpreter in Rust from scratch. By the end of this exercise you should be able to hand-trace what any EVM bytecode does at the opcode level, understand how the stack, memory, and storage interact, and have a working interpreter that can execute real contract bytecode snippets.

## Reference

Keep https://evm.codes open while you work. Every opcode is documented there with stack inputs/outputs, gas costs, and examples.

## Project structure

```
toy-evm/
├── Cargo.toml          # No external dependencies
├── src/
│   └── lib.rs          # U256 type + Evm struct (starter code provided)
├── tests/
│   └── exercises.rs    # Test file — validates each exercise phase
└── CLAUDE.md           # This file
```

## How to work through this

The starter `lib.rs` has the full EVM struct scaffolded with `todo!()` placeholders. Work through the exercises in order — each one unlocks the next. Run `cargo test` after each phase to validate your work. Tests are organized by exercise number so you can run a specific phase:

```bash
cargo test exercise_1a    # U256 arithmetic
cargo test exercise_1b    # Arithmetic opcodes
cargo test exercise_1c    # Comparison and bitwise opcodes
cargo test exercise_1d    # PUSH opcodes
cargo test exercise_1e    # POP, DUP, SWAP
cargo test exercise_2a    # Memory (MLOAD, MSTORE, MSTORE8)
cargo test exercise_2b    # Storage (SLOAD, SSTORE)
cargo test exercise_2c    # Control flow (JUMP, JUMPI, JUMPDEST)
cargo test exercise_3     # Integration: real-world bytecode patterns
```

---

## Exercise 1a — U256 arithmetic

**What you're learning:** The EVM operates on 256-bit integers. Ethereum uses 256 bits because addresses are 160-bit, hashes are 256-bit, and token balances can be astronomically large (think 18 decimal places). Every stack slot, every storage value, every memory word is 256 bits.

**What to implement in `lib.rs`:**

- `wrapping_add(self, other) -> Self` — Addition modulo 2^256. Work from byte index 31 (least significant) to 0, carrying overflow.
- `wrapping_sub(self, other) -> Self` — Subtraction with wrapping underflow. Hint: a - b = a + (NOT b) + 1 (two's complement).
- `wrapping_mul(self, other) -> Self` — Multiplication modulo 2^256. Easiest approach: split each number into u64 limbs and do schoolbook multiplication with carries.
- `wrapping_div(self, other) -> Self` — Integer division. Returns 0 if divisor is zero (EVM spec, not a panic). You can implement long division or convert to a simpler representation.
- `wrapping_mod(self, other) -> Self` — Modulo. Returns 0 if divisor is zero.
- `is_zero(&self) -> bool` — Check all 32 bytes are zero.
- `bitwise_and`, `bitwise_or`, `bitwise_not` — Byte-by-byte bitwise ops.
- `less_than`, `greater_than` — Unsigned comparison of big-endian byte arrays. Compare from the most significant byte (index 0) downward.

**Hints:**
- For add/sub, a u16 accumulator per byte handles the carry cleanly.
- For mul, think of the U256 as four u64 limbs: `[u64; 4]` where limb[0] is most significant. Schoolbook multiply: for each pair of limbs, multiply to get u128, accumulate, carry.
- For div/mod, the simplest correct approach for learning purposes is shift-and-subtract long division. It's not fast but it's correct and teaches you how EVM big-int math works.
- Comparison: iterate from byte 0 to 31. First byte that differs determines the result.

**Validation:** `cargo test exercise_1a`

---

## Exercise 1b — Arithmetic opcodes (ADD, MUL, SUB, DIV, MOD)

**What you're learning:** How the EVM stack machine works. Every arithmetic opcode pops its operands from the stack and pushes the result back. There are no registers — everything goes through the stack.

**What to implement in `lib.rs` `step()` method:**

- `ADD` (0x01): pop `a`, pop `b`, push `a.wrapping_add(b)`
- `MUL` (0x02): pop `a`, pop `b`, push `a.wrapping_mul(b)`
- `SUB` (0x03): pop `a`, pop `b`, push `a.wrapping_sub(b)`
- `DIV` (0x04): pop `a`, pop `b`, push `a.wrapping_div(b)`
- `MOD` (0x06): pop `a`, pop `b`, push `a.wrapping_mod(b)`

**Pattern for each:**
```rust
opcodes::ADD => {
    let a = self.pop()?;
    let b = self.pop()?;
    self.push(a.wrapping_add(b))?;
}
```

Note: `self.pop()` returns `Result<U256, Option<ExecutionResult>>`. The `?` propagates a `StackUnderflow` error if the stack is empty. Same for `self.push()` — it errors on `StackOverflow` (>1024 items).

**Validation:** `cargo test exercise_1b`

---

## Exercise 1c — Comparison and bitwise opcodes

**What you're learning:** Conditional logic in the EVM. Smart contracts use these for `if` statements, `require()` checks, and access control. The EVM has no booleans — it uses U256 where 0 = false and anything else = true.

**What to implement:**

- `LT` (0x10): pop `a`, pop `b`, push 1 if `a < b`, else 0
- `GT` (0x11): pop `a`, pop `b`, push 1 if `a > b`, else 0
- `EQ` (0x14): pop `a`, pop `b`, push 1 if `a == b`, else 0
- `ISZERO` (0x15): pop `a`, push 1 if `a == 0`, else 0
- `AND` (0x16): pop `a`, pop `b`, push `a & b`
- `OR` (0x17): pop `a`, pop `b`, push `a | b`
- `NOT` (0x19): pop `a`, push `~a` (bitwise complement)

**Pattern:**
```rust
opcodes::LT => {
    let a = self.pop()?;
    let b = self.pop()?;
    let result = if a.less_than(&b) { U256::ONE } else { U256::ZERO };
    self.push(result)?;
}
```

**Validation:** `cargo test exercise_1c`

---

## Exercise 1d — PUSH opcodes (PUSH1 through PUSH32)

**What you're learning:** How immediate values get onto the stack. In x86 you'd write `mov eax, 42`. In the EVM, you write `PUSH1 0x2a` — the value 42 is encoded directly in the bytecode after the opcode. PUSH1 reads 1 byte, PUSH2 reads 2 bytes, all the way to PUSH32 which reads 32 bytes.

**What to implement:**

For any opcode in the range 0x60..=0x7f:
1. Calculate `n = opcode - 0x60 + 1` (number of bytes to read)
2. Read bytes from `bytecode[pc+1..pc+1+n]`
3. Place them right-aligned into a U256 (big-endian)
4. Push the U256 onto the stack
5. Advance `pc` past the data bytes: `self.pc += n` (the main loop adds +1 for the opcode itself)

**Example:** Bytecode `[0x61, 0x01, 0x00]` is `PUSH2 0x0100` which pushes 256 onto the stack.

**Important:** After handling a PUSH, you need to advance `pc` by `n` *extra* bytes (beyond the normal +1 that happens at the end of `step()`). Set `self.pc += n` inside the PUSH handler, and the `self.pc += 1` at the bottom of `step()` handles the opcode byte itself.

**Validation:** `cargo test exercise_1d`

---

## Exercise 1e — Stack manipulation (POP, DUP, SWAP)

**What you're learning:** The EVM only has a stack — no named variables. Solidity's compiler uses DUP and SWAP constantly to juggle values. Understanding these helps you read compiled bytecode.

**What to implement:**

- `POP` (0x50): remove and discard the top stack item
- `DUP1..DUP16` (0x80..0x8f): duplicate the Nth item from the top. DUP1 copies the top item, DUP2 copies the second, etc.
  - `n = opcode - 0x80 + 1`
  - Copy `stack[stack.len() - n]` and push it on top
  - Return StackUnderflow if the stack has fewer than `n` items
- `SWAP1..SWAP16` (0x90..0x9f): swap the top item with the (N+1)th item. SWAP1 swaps positions 0 and 1, SWAP2 swaps 0 and 2, etc.
  - `n = opcode - 0x90 + 1`
  - Swap `stack[len-1]` with `stack[len-1-n]`
  - Return StackUnderflow if the stack has fewer than `n+1` items

**Validation:** `cargo test exercise_1e`

---

## Exercise 2a — Memory (MLOAD, MSTORE, MSTORE8)

**What you're learning:** EVM memory is a byte array that starts empty and grows as needed. It's volatile — cleared after each transaction. Smart contracts use memory for function arguments, return values, and intermediate computation. Memory is cheap but not free — it costs gas proportional to the square of the highest accessed offset.

**What to implement:**

- `MSTORE` (0x52): pop `offset`, pop `value`, write `value` as 32 big-endian bytes to `memory[offset..offset+32]`
- `MLOAD` (0x51): pop `offset`, read 32 bytes from `memory[offset..offset+32]`, push as U256
- `MSTORE8` (0x53): pop `offset`, pop `value`, write the least significant byte of `value` to `memory[offset]`

**Key detail:** Memory auto-extends with zeros. Before any read or write, call `self.expand_memory(offset + size)` where size is 32 for MLOAD/MSTORE and 1 for MSTORE8. The `expand_memory` helper is already implemented — it rounds up to the nearest 32-byte word.

**Pattern:**
```rust
opcodes::MSTORE => {
    let offset = self.pop()?.as_usize();
    let value = self.pop()?;
    self.expand_memory(offset + 32);
    self.memory[offset..offset + 32].copy_from_slice(&value.0);
}
```

For MLOAD, do the reverse — read 32 bytes into a `[u8; 32]` array, wrap in `U256()`, and push.

**Validation:** `cargo test exercise_2a`

---

## Exercise 2b — Storage (SLOAD, SSTORE)

**What you're learning:** Storage is the permanent state on the blockchain. When a Uniswap pool stores its reserves, when an ERC-20 records balances, when a governance contract tracks votes — that's all storage. It's a mapping from U256 keys to U256 values. In a real EVM, SSTORE is the most expensive opcode (20,000 gas for a fresh write) because it changes consensus state that every node must persist.

**What to implement:**

- `SLOAD` (0x54): pop `key`, push `storage[key]` (default to U256::ZERO if the key doesn't exist)
- `SSTORE` (0x55): pop `key`, pop `value`, set `storage[key] = value`

**Pattern:**
```rust
opcodes::SLOAD => {
    let key = self.pop()?;
    let value = self.storage.get(&key).copied().unwrap_or(U256::ZERO);
    self.push(value)?;
}
```

**Why this matters for MEV:** When you simulate a Uniswap swap, you're predicting what SSTORE will do to the pool's reserve slots. Slot 8 in a Uniswap V2 pair holds `reserve0` and `reserve1` packed together. An MEV bot that can read and predict storage changes can calculate exact profit.

**Validation:** `cargo test exercise_2b`

---

## Exercise 2c — Control flow (JUMP, JUMPI, JUMPDEST)

**What you're learning:** Loops and conditionals in the EVM. Solidity's `if`, `for`, `while`, and `require()` all compile to JUMPI instructions. JUMP is unconditional — go to address. JUMPI is conditional — go if the condition is nonzero. Every jump target must be a JUMPDEST opcode, which prevents jumping into the middle of a PUSH instruction's data (a security measure).

**What to implement:**

- `JUMP` (0x56): pop `dest`, verify `bytecode[dest] == JUMPDEST`, set `pc = dest`, and return `None` (do NOT let the `pc += 1` at the end of `step()` execute — you need to return early after setting pc)
- `JUMPI` (0x57): pop `dest`, pop `condition`. If `condition != 0`, do the same as JUMP. If `condition == 0`, just advance normally (fall through).
- `JUMPDEST` (0x5b): no-op. Just marks a valid jump target.

**Critical detail about `pc` management:** The normal `self.pc += 1` at the bottom of `step()` runs after every opcode. For JUMP/JUMPI (when taken), you set `pc` directly to the destination. You must NOT let the `+1` run. The cleanest approach:

```rust
opcodes::JUMP => {
    let dest = self.pop()?.as_usize();
    if dest >= self.bytecode.len() || self.bytecode[dest] != opcodes::JUMPDEST {
        return Some(ExecutionResult::InvalidJump);
    }
    self.pc = dest;
    return None;  // Skip the pc += 1 at the bottom
}
```

For JUMPI when the condition is zero (not taken), just let execution fall through to the normal `pc += 1`.

**Validation:** `cargo test exercise_2c`

---

## Exercise 3 — Integration tests: real-world patterns

Once all opcodes are implemented, these tests verify your EVM can handle patterns from actual smart contracts.

### 3a — Simple counter

Bytecode that implements `counter = counter + 1` using storage:
```
PUSH1 0x00    // storage slot 0
SLOAD         // load current value
PUSH1 0x01    // constant 1
ADD           // current + 1
PUSH1 0x00    // storage slot 0
SSTORE        // store new value
STOP
```

### 3b — Conditional branch

Bytecode that checks `if (x > 10) { store 1 } else { store 0 }`:
```
PUSH1 0x05    // x = 5
PUSH1 0x0a    // threshold = 10
GT            // 5 > 10? => 0 (false)
PUSH1 0x??    // jump target (JUMPDEST for the "true" branch)
JUMPI         // conditional jump
PUSH1 0x00    // false: store 0
PUSH1 0x00
SSTORE
STOP
JUMPDEST      // true branch
PUSH1 0x01    // true: store 1
PUSH1 0x00
SSTORE
STOP
```

### 3c — Simple loop

Bytecode that computes `sum = 0 + 1 + 2 + ... + 5`:
```
PUSH1 0x00    // sum = 0
PUSH1 0x01    // i = 1
JUMPDEST      // loop_start:
DUP1          // copy i
PUSH1 0x06    // limit = 6
LT            // i < 6?
ISZERO        // invert: if i >= 6, done
PUSH1 ??      // jump to loop_end
JUMPI
DUP1          // copy i
SWAP2         // stack: i, sum, i -> sum, i, i ... rearrange for add
ADD           // sum += i
SWAP1         // put i back on top
PUSH1 0x01
ADD           // i += 1
PUSH1 ??      // jump to loop_start
JUMP
JUMPDEST      // loop_end:
POP           // remove i, leaving sum on stack
PUSH1 0x00
SSTORE        // store sum in slot 0
STOP
```

### 3d — Memory round-trip

Store a value in memory, load it back, verify it matches:
```
PUSH1 0x42    // value = 0x42
PUSH1 0x00    // offset = 0
MSTORE        // memory[0..32] = 0x42
PUSH1 0x00    // offset = 0
MLOAD         // push memory[0..32]
```

### 3e — Return data

Store bytes in memory and return them:
```
PUSH1 0xFF    // value
PUSH1 0x00    // offset
MSTORE        // write to memory
PUSH1 0x20    // size = 32 bytes
PUSH1 0x00    // offset = 0
RETURN        // return memory[0..32]
```

**Validation:** `cargo test exercise_3`

---

## Debugging tips

- Add `println!` in your `step()` method to trace execution:
  ```rust
  println!("pc={} opcode=0x{:02x} stack={:?}", self.pc, opcode, self.stack);
  ```
- If a test hangs, you probably have an infinite loop — your JUMP target math is wrong or you forgot to advance `pc` past PUSH data bytes.
- If you get `StackUnderflow`, trace the bytecode by hand and count what should be on the stack at each step.
- Use `cargo test -- --nocapture` to see println output during tests.

## What's next (Week 2)

Once you've completed all exercises, you'll have a working EVM interpreter that handles arithmetic, logic, memory, storage, and control flow. In week 2, you'll extend this with:

- CALL, STATICCALL, DELEGATECALL (contract-to-contract calls)
- CALLVALUE, CALLDATALOAD, CALLDATASIZE (transaction context)
- SHA3 (keccak256 — how storage slots are computed for mappings)
- Gas accounting per opcode (real costs, not the simplified flat 3 gas)
- LOG0..LOG4 (events — how contracts emit data that off-chain systems read)

This builds toward understanding how Uniswap's `swap()` function executes at the bytecode level — which is exactly what `revm` does under the hood in week 5.
