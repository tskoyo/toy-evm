use toy_evm::*;

// ============================================================
// Helper: build bytecode from a slice and run it
// ============================================================
fn run_bytecode(bytecode: &[u8]) -> (Evm, ExecutionResult) {
    let mut evm = Evm::new(bytecode.to_vec(), 100_000);
    let result = evm.run();
    (evm, result)
}

// ============================================================
// Exercise 1a — U256 arithmetic
// ============================================================

#[test]
fn exercise_1a_add_small() {
    let a = U256::from_u64(665537);
    let b = U256::from_u64(20);
    assert_eq!(a.wrapping_add(b), U256::from_u64(665557));
}

#[test]
fn exercise_1a_add_overflow() {
    // MAX + 1 should wrap to 0
    let result = U256::MAX.wrapping_add(U256::ONE);
    assert_eq!(result, U256::ZERO);
}

#[test]
fn exercise_1a_sub_small() {
    let a = U256::from_u64(50);
    let b = U256::from_u64(30);
    assert_eq!(a.wrapping_sub(b), U256::from_u64(20));
}

#[test]
fn exercise_1a_sub_underflow() {
    // 0 - 1 should wrap to MAX
    let result = U256::ZERO.wrapping_sub(U256::ONE);
    assert_eq!(result, U256::MAX);
}

#[test]
fn exercise_1a_mul_small() {
    let a = U256::from_u64(7);
    let b = U256::from_u64(8);
    assert_eq!(a.wrapping_mul(b), U256::from_u64(56));
}

#[test]
fn exercise_1a_mul_large() {
    let a = U256::from_u64(1_000_000);
    let b = U256::from_u64(1_000_000);
    assert_eq!(a.wrapping_mul(b), U256::from_u64(1_000_000_000_000));
}

#[test]
fn exercise_1a_div() {
    let a = U256::from_u64(100);
    let b = U256::from_u64(7);
    assert_eq!(a.wrapping_div(b), U256::from_u64(14)); // 100 / 7 = 14
}

#[test]
fn exercise_1a_div_by_zero() {
    let a = U256::from_u64(100);
    assert_eq!(a.wrapping_div(U256::ZERO), U256::ZERO);
}

#[test]
fn exercise_1a_mod() {
    let a = U256::from_u64(100);
    let b = U256::from_u64(7);
    assert_eq!(a.wrapping_mod(b), U256::from_u64(2)); // 100 % 7 = 2
}

#[test]
fn exercise_1a_mod_by_zero() {
    let a = U256::from_u64(100);
    assert_eq!(a.wrapping_mod(U256::ZERO), U256::ZERO);
}

#[test]
fn exercise_1a_is_zero() {
    assert!(U256::ZERO.is_zero());
    assert!(!U256::ONE.is_zero());
    assert!(!U256::from_u64(255).is_zero());
}

#[test]
fn exercise_1a_bitwise_and() {
    let a = U256::from_u64(0xFF00);
    let b = U256::from_u64(0x0FF0);
    assert_eq!(a.bitwise_and(b), U256::from_u64(0x0F00));
}

#[test]
fn exercise_1a_bitwise_or() {
    let a = U256::from_u64(0xFF00);
    let b = U256::from_u64(0x00FF);
    assert_eq!(a.bitwise_or(b), U256::from_u64(0xFFFF));
}

#[test]
fn exercise_1a_bitwise_not() {
    let result = U256::ZERO.bitwise_not();
    assert_eq!(result, U256::MAX);
}

#[test]
fn exercise_1a_less_than() {
    assert!(U256::from_u64(5).less_than(&U256::from_u64(10)));
    assert!(!U256::from_u64(10).less_than(&U256::from_u64(5)));
    assert!(!U256::from_u64(5).less_than(&U256::from_u64(5)));
}

#[test]
fn exercise_1a_greater_than() {
    assert!(U256::from_u64(10).greater_than(&U256::from_u64(5)));
    assert!(!U256::from_u64(5).greater_than(&U256::from_u64(10)));
    assert!(!U256::from_u64(5).greater_than(&U256::from_u64(5)));
}

// ============================================================
// Exercise 1b — Arithmetic opcodes
// ============================================================

#[test]
fn exercise_1b_add() {
    // PUSH1 10, PUSH1 20, ADD, STOP
    let (evm, result) = run_bytecode(&[0x60, 10, 0x60, 20, 0x01, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(30));
}

#[test]
fn exercise_1b_mul() {
    // PUSH1 7, PUSH1 8, MUL, STOP
    let (evm, result) = run_bytecode(&[0x60, 7, 0x60, 8, 0x02, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(56));
}

#[test]
fn exercise_1b_sub() {
    // PUSH1 10, PUSH1 30, SUB → 30 - 10 = 20
    // Note: SUB pops a then b, computes a - b
    let (evm, result) = run_bytecode(&[0x60, 10, 0x60, 30, 0x03, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(20));
}

#[test]
fn exercise_1b_div() {
    // PUSH1 7, PUSH1 100, DIV → 100 / 7 = 14
    let (evm, result) = run_bytecode(&[0x60, 7, 0x60, 100, 0x04, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(14));
}

#[test]
fn exercise_1b_mod() {
    // PUSH1 7, PUSH1 100, MOD → 100 % 7 = 2
    let (evm, result) = run_bytecode(&[0x60, 7, 0x60, 100, 0x06, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(2));
}

// ============================================================
// Exercise 1c — Comparison and bitwise opcodes
// ============================================================

#[test]
fn exercise_1c_lt_true() {
    // PUSH1 10, PUSH1 5, LT → 5 < 10 = 1
    let (evm, result) = run_bytecode(&[0x60, 10, 0x60, 5, 0x10, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::ONE);
}

#[test]
fn exercise_1c_lt_false() {
    // PUSH1 5, PUSH1 10, LT → 10 < 5 = 0
    let (evm, result) = run_bytecode(&[0x60, 5, 0x60, 10, 0x10, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::ZERO);
}

#[test]
fn exercise_1c_gt_true() {
    // PUSH1 5, PUSH1 10, GT → 10 > 5 = 1
    let (evm, result) = run_bytecode(&[0x60, 5, 0x60, 10, 0x11, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::ONE);
}

#[test]
fn exercise_1c_eq_true() {
    // PUSH1 42, PUSH1 42, EQ → 1
    let (evm, result) = run_bytecode(&[0x60, 42, 0x60, 42, 0x14, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::ONE);
}

#[test]
fn exercise_1c_eq_false() {
    // PUSH1 42, PUSH1 43, EQ → 0
    let (evm, result) = run_bytecode(&[0x60, 42, 0x60, 43, 0x14, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::ZERO);
}

#[test]
fn exercise_1c_iszero_true() {
    // PUSH1 0, ISZERO → 1
    let (evm, result) = run_bytecode(&[0x60, 0, 0x15, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::ONE);
}

#[test]
fn exercise_1c_iszero_false() {
    // PUSH1 5, ISZERO → 0
    let (evm, result) = run_bytecode(&[0x60, 5, 0x15, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::ZERO);
}

#[test]
fn exercise_1c_and() {
    // PUSH1 0x0F, PUSH1 0xFF, AND → 0x0F
    let (evm, result) = run_bytecode(&[0x60, 0x0F, 0x60, 0xFF, 0x16, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(0x0F));
}

#[test]
fn exercise_1c_or() {
    // PUSH1 0xF0, PUSH1 0x0F, OR → 0xFF
    let (evm, result) = run_bytecode(&[0x60, 0xF0, 0x60, 0x0F, 0x17, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(0xFF));
}

#[test]
fn exercise_1c_not() {
    // PUSH1 0x00, NOT → all 1s (U256::MAX)
    let (evm, result) = run_bytecode(&[0x60, 0x00, 0x19, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::MAX);
}

// ============================================================
// Exercise 1d — PUSH opcodes
// ============================================================

#[test]
fn exercise_1d_push1() {
    // PUSH1 0x42, STOP
    let (evm, result) = run_bytecode(&[0x60, 0x42, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(0x42));
}

#[test]
fn exercise_1d_push2() {
    // PUSH2 0x01 0x00 (= 256), STOP
    let (evm, result) = run_bytecode(&[0x61, 0x01, 0x00, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(256));
}

#[test]
fn exercise_1d_push4() {
    // PUSH4 0x00 0x01 0x00 0x00 (= 65536), STOP
    let (evm, result) = run_bytecode(&[0x63, 0x00, 0x01, 0x00, 0x00, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(65536));
}

#[test]
fn exercise_1d_push32() {
    // PUSH32 with all zeros except last byte = 0xFF
    let mut bytecode = vec![0x7f]; // PUSH32
    bytecode.extend_from_slice(&[0u8; 31]);
    bytecode.push(0xFF);
    bytecode.push(0x00); // STOP
    let (evm, result) = run_bytecode(&bytecode);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(0xFF));
}

#[test]
fn exercise_1d_multiple_pushes() {
    // PUSH1 1, PUSH1 2, PUSH1 3 → stack should be [1, 2, 3] (3 on top)
    let (evm, result) = run_bytecode(&[0x60, 1, 0x60, 2, 0x60, 3, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack.len(), 3);
    assert_eq!(evm.stack[0], U256::from_u64(1));
    assert_eq!(evm.stack[1], U256::from_u64(2));
    assert_eq!(evm.stack[2], U256::from_u64(3));
}

// ============================================================
// Exercise 1e — POP, DUP, SWAP
// ============================================================

#[test]
fn exercise_1e_pop() {
    // PUSH1 1, PUSH1 2, POP → stack: [1]
    let (evm, result) = run_bytecode(&[0x60, 1, 0x60, 2, 0x50, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack.len(), 1);
    assert_eq!(evm.stack[0], U256::from_u64(1));
}

#[test]
fn exercise_1e_dup1() {
    // PUSH1 42, DUP1 → stack: [42, 42]
    let (evm, result) = run_bytecode(&[0x60, 42, 0x80, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack.len(), 2);
    assert_eq!(evm.stack[0], U256::from_u64(42));
    assert_eq!(evm.stack[1], U256::from_u64(42));
}

#[test]
fn exercise_1e_dup2() {
    // PUSH1 1, PUSH1 2, DUP2 → stack: [1, 2, 1]
    let (evm, result) = run_bytecode(&[0x60, 1, 0x60, 2, 0x81, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack.len(), 3);
    assert_eq!(evm.stack[0], U256::from_u64(1));
    assert_eq!(evm.stack[1], U256::from_u64(2));
    assert_eq!(evm.stack[2], U256::from_u64(1)); // copy of position 2 from top
}

#[test]
fn exercise_1e_swap1() {
    // PUSH1 1, PUSH1 2, SWAP1 → stack: [2, 1]
    let (evm, result) = run_bytecode(&[0x60, 1, 0x60, 2, 0x90, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(2));
    assert_eq!(evm.stack[1], U256::from_u64(1));
}

#[test]
fn exercise_1e_swap2() {
    // PUSH1 1, PUSH1 2, PUSH1 3, SWAP2 → stack: [3, 2, 1]
    let (evm, result) = run_bytecode(&[0x60, 1, 0x60, 2, 0x60, 3, 0x91, 0x00]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(3));
    assert_eq!(evm.stack[1], U256::from_u64(2));
    assert_eq!(evm.stack[2], U256::from_u64(1));
}

// ============================================================
// Exercise 2a — Memory operations
// ============================================================

#[test]
fn exercise_2a_mstore_mload() {
    // PUSH1 0x42, PUSH1 0x00, MSTORE, PUSH1 0x00, MLOAD → 0x42 on stack
    let (evm, result) = run_bytecode(&[
        0x60, 0x42, // PUSH1 0x42
        0x60, 0x00, // PUSH1 0x00 (offset)
        0x52, // MSTORE
        0x60, 0x00, // PUSH1 0x00 (offset)
        0x51, // MLOAD
        0x00, // STOP
    ]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(0x42));
}

#[test]
fn exercise_2a_mstore8() {
    // PUSH1 0xAB, PUSH1 0x00, MSTORE8 → memory[0] = 0xAB
    let (evm, result) = run_bytecode(&[
        0x60, 0xAB, // PUSH1 0xAB
        0x60, 0x00, // PUSH1 0x00 (offset)
        0x53, // MSTORE8
        0x00, // STOP
    ]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.memory[0], 0xAB);
}

#[test]
fn exercise_2a_memory_expansion() {
    // PUSH1 0xFF, PUSH1 0x40, MSTORE8 → memory expands to at least 0x41 bytes
    let (evm, result) = run_bytecode(&[
        0x60, 0xFF, 0x60, 0x40, // offset 64
        0x53, // MSTORE8
        0x00,
    ]);
    assert_eq!(result, ExecutionResult::Stop);
    assert!(evm.memory.len() >= 65);
    assert_eq!(evm.memory[64], 0xFF);
}

#[test]
fn exercise_2a_msize() {
    // MSIZE initially 0, after MSTORE at offset 0 → 32
    let (evm, result) = run_bytecode(&[
        0x60, 0x01, // PUSH1 1
        0x60, 0x00, // PUSH1 0
        0x52, // MSTORE
        0x59, // MSIZE
        0x00, // STOP
    ]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(32));
}

// ============================================================
// Exercise 2b — Storage operations
// ============================================================

#[test]
fn exercise_2b_sstore_sload() {
    // PUSH1 0xFF (value), PUSH1 0x00 (key), SSTORE
    // PUSH1 0x00 (key), SLOAD → 0xFF on stack
    let (evm, result) = run_bytecode(&[
        0x60, 0xFF, // value
        0x60, 0x00, // key = slot 0
        0x55, // SSTORE
        0x60, 0x00, // key = slot 0
        0x54, // SLOAD
        0x00, // STOP
    ]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(0xFF));
}

#[test]
fn exercise_2b_sload_empty_slot() {
    // SLOAD from an unused slot returns 0
    let (evm, result) = run_bytecode(&[
        0x60, 0x05, // key = slot 5
        0x54, // SLOAD
        0x00, // STOP
    ]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::ZERO);
}

#[test]
fn exercise_2b_sstore_overwrite() {
    // Write 10 to slot 0, then write 20 to slot 0, then read
    let (evm, result) = run_bytecode(&[
        0x60, 10, // value = 10
        0x60, 0x00, // slot 0
        0x55, // SSTORE
        0x60, 20, // value = 20
        0x60, 0x00, // slot 0
        0x55, // SSTORE
        0x60, 0x00, // slot 0
        0x54, // SLOAD
        0x00, // STOP
    ]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(20));
}

// ============================================================
// Exercise 2c — Control flow
// ============================================================

#[test]
fn exercise_2c_jump() {
    // PUSH1 0x04, JUMP, INVALID, JUMPDEST, PUSH1 0x42, STOP
    // Bytecode: [0x60, 0x04, 0x56, 0xFE, 0x5B, 0x60, 0x42, 0x00]
    //             0     1     2     3     4     5     6     7
    // Jumps to offset 4 (JUMPDEST), skipping the INVALID at offset 3
    let (evm, result) = run_bytecode(&[
        0x60, 0x04, // PUSH1 4
        0x56, // JUMP
        0xFE, // INVALID (should be skipped)
        0x5B, // JUMPDEST
        0x60, 0x42, // PUSH1 0x42
        0x00, // STOP
    ]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(0x42));
}

#[test]
fn exercise_2c_jumpi_taken() {
    // PUSH1 1 (condition=true), PUSH1 0x06, JUMPI, INVALID, INVALID, INVALID, JUMPDEST, PUSH1 0x42, STOP
    let (evm, result) = run_bytecode(&[
        0x60, 0x01, // PUSH1 1 (truthy)
        0x60, 0x06, // PUSH1 6 (dest)
        0x57, // JUMPI
        0xFE, // INVALID (skipped)
        0x5B, // JUMPDEST at offset 6
        0x60, 0x42, // PUSH1 0x42
        0x00, // STOP
    ]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(0x42));
}

#[test]
fn exercise_2c_jumpi_not_taken() {
    // PUSH1 0 (condition=false), PUSH1 0x06, JUMPI, PUSH1 0x99, STOP
    let (evm, result) = run_bytecode(&[
        0x60, 0x00, // PUSH1 0 (falsy)
        0x60, 0x06, // PUSH1 6 (dest)
        0x57, // JUMPI — not taken
        0x60, 0x99, // PUSH1 0x99
        0x00, // STOP
    ]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(0x99));
}

#[test]
fn exercise_2c_invalid_jump() {
    // PUSH1 0x03, JUMP — offset 3 is not a JUMPDEST
    let (_, result) = run_bytecode(&[
        0x60, 0x03, // PUSH1 3
        0x56, // JUMP
        0x60, 0x42, // offset 3 is PUSH1, not JUMPDEST
    ]);
    assert_eq!(result, ExecutionResult::InvalidJump);
}

// ============================================================
// Exercise 3 — Integration tests
// ============================================================

#[test]
fn exercise_3a_counter_increment() {
    // counter = counter + 1, twice
    let bytecode = vec![
        // First increment: load slot 0, add 1, store slot 0
        0x60, 0x00, // PUSH1 0 (slot)
        0x54, // SLOAD (= 0)
        0x60, 0x01, // PUSH1 1
        0x01, // ADD
        0x60, 0x00, // PUSH1 0 (slot)
        0x55, // SSTORE (slot 0 = 1)
        // Second increment
        0x60, 0x00, // PUSH1 0 (slot)
        0x54, // SLOAD (= 1)
        0x60, 0x01, // PUSH1 1
        0x01, // ADD
        0x60, 0x00, // PUSH1 0 (slot)
        0x55, // SSTORE (slot 0 = 2)
        0x00, // STOP
    ];
    let (evm, result) = run_bytecode(&bytecode);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.get_storage(&U256::ZERO), U256::from_u64(2));
}

#[test]
fn exercise_3b_conditional_false() {
    // if (5 > 10) { store 1 } else { store 0 }
    // Expected: 5 > 10 is false → store 0
    let bytecode = vec![
        0x60, 0x0A, // PUSH1 10
        0x60, 0x05, // PUSH1 5
        0x11, // GT: 5 > 10 = 0
        0x60, 0x0F, // PUSH1 15 (jump target: true branch)
        0x57, // JUMPI — not taken (condition is 0)
        // false branch:
        0x60, 0x00, // PUSH1 0 (value)
        0x60, 0x00, // PUSH1 0 (slot)
        0x55, // SSTORE
        0x00, // STOP
        // Padding byte
        0x00, // true branch at offset 15:
        0x5B, // JUMPDEST
        0x60, 0x01, // PUSH1 1 (value)
        0x60, 0x00, // PUSH1 0 (slot)
        0x55, // SSTORE
        0x00, // STOP
    ];
    let (evm, result) = run_bytecode(&bytecode);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.get_storage(&U256::ZERO), U256::ZERO);
}

#[test]
fn exercise_3b_conditional_true() {
    // if (20 > 10) { store 1 } else { store 0 }
    // Expected: 20 > 10 is true → store 1
    let bytecode = vec![
        0x60, 0x0A, // PUSH1 10
        0x60, 0x14, // PUSH1 20
        0x11, // GT: 20 > 10 = 1
        0x60, 0x0F, // PUSH1 15 (jump target: true branch)
        0x57, // JUMPI — taken
        // false branch:
        0x60, 0x00, // PUSH1 0
        0x60, 0x00, // PUSH1 0
        0x55, // SSTORE
        0x00, // STOP
        // Padding
        0x00, // true branch at offset 15:
        0x5B, // JUMPDEST
        0x60, 0x01, // PUSH1 1
        0x60, 0x00, // PUSH1 0
        0x55, // SSTORE
        0x00, // STOP
    ];
    let (evm, result) = run_bytecode(&bytecode);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.get_storage(&U256::ZERO), U256::ONE);
}

#[test]
fn exercise_3c_loop_sum_1_to_5() {
    // Computes sum = 1 + 2 + 3 + 4 + 5 = 15
    //
    // Layout:
    //   0: PUSH1 0      (sum = 0)
    //   2: PUSH1 1      (i = 1)
    //   4: JUMPDEST      (loop_start)
    //   5: DUP1          (copy i)
    //   6: PUSH1 6       (limit = 6)
    //   8: LT            (i < 6?)
    //   9: ISZERO         (if NOT less_than, we're done)
    //  10: PUSH1 24      (loop_end address)
    //  12: JUMPI          (jump to end if done)
    //  13: DUP1           (copy i)
    //  14: SWAP2          (rearrange: sum is now on top with i below)
    //  15: ADD            (sum += i)
    //  16: SWAP1          (put i back on top)
    //  17: PUSH1 1        (constant 1)
    //  19: ADD            (i += 1)
    //  20: PUSH1 4        (loop_start address)
    //  22: JUMP           (go back)
    //  23: INVALID        (should never reach here)
    //  24: JUMPDEST       (loop_end)
    //  25: POP            (remove i)
    //  26: PUSH1 0        (slot 0)
    //  28: SSTORE         (store sum)
    //  29: STOP

    let bytecode = vec![
        0x60, 0x00, // PUSH1 0 (sum)
        0x60, 0x01, // PUSH1 1 (i)
        0x5B, // JUMPDEST (loop_start, offset 4)
        0x80, // DUP1 (copy i)
        0x60, 0x06, // PUSH1 6 (limit)
        0x10, // LT (i < 6?)
        0x15, // ISZERO (negate)
        0x60, 0x18, // PUSH1 24 (loop_end)
        0x57, // JUMPI
        0x80, // DUP1 (copy i)
        0x91, // SWAP2
        0x01, // ADD (sum += i)
        0x90, // SWAP1
        0x60, 0x01, // PUSH1 1
        0x01, // ADD (i += 1)
        0x60, 0x04, // PUSH1 4 (loop_start)
        0x56, // JUMP
        0xFE, // INVALID
        0x5B, // JUMPDEST (loop_end, offset 24)
        0x50, // POP (remove i)
        0x60, 0x00, // PUSH1 0 (slot)
        0x55, // SSTORE
        0x00, // STOP
    ];

    let (evm, result) = run_bytecode(&bytecode);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.get_storage(&U256::ZERO), U256::from_u64(15));
}

#[test]
fn exercise_3d_memory_round_trip() {
    // Store a value in memory, load it back
    let (evm, result) = run_bytecode(&[
        0x60, 0x42, // PUSH1 0x42
        0x60, 0x00, // PUSH1 0 (offset)
        0x52, // MSTORE
        0x60, 0x00, // PUSH1 0 (offset)
        0x51, // MLOAD
        0x00, // STOP
    ]);
    assert_eq!(result, ExecutionResult::Stop);
    assert_eq!(evm.stack[0], U256::from_u64(0x42));
}

#[test]
fn exercise_3e_return_data() {
    // Store 0xFF in memory, return 32 bytes
    let (_, result) = run_bytecode(&[
        0x60, 0xFF, // PUSH1 0xFF
        0x60, 0x00, // PUSH1 0 (offset)
        0x52, // MSTORE
        0x60, 0x20, // PUSH1 32 (size)
        0x60, 0x00, // PUSH1 0 (offset)
        0xF3, // RETURN
    ]);
    match result {
        ExecutionResult::Return(data) => {
            assert_eq!(data.len(), 32);
            // 0xFF is stored as a U256, so it's at the last byte
            assert_eq!(data[31], 0xFF);
        }
        other => panic!("Expected Return, got {:?}", other),
    }
}

#[test]
fn exercise_3_stack_underflow() {
    // ADD with nothing on the stack → StackUnderflow
    let (_, result) = run_bytecode(&[0x01]);
    assert_eq!(result, ExecutionResult::StackUnderflow);
}

#[test]
fn exercise_3_out_of_gas() {
    // Infinite loop with low gas
    let mut evm = Evm::new(
        vec![
            0x5B, // JUMPDEST at 0
            0x60, 0x00, // PUSH1 0
            0x56, // JUMP to 0
        ],
        20, // only 20 gas — will run out
    );
    let result = evm.run();
    assert_eq!(result, ExecutionResult::OutOfGas);
}
