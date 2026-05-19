/// # Toy EVM — Week 1 Exercise
///
/// This is a minimal Ethereum Virtual Machine interpreter.
/// Your job: implement each opcode handler in the `step()` method.
///
/// The EVM is a stack-based machine. Every instruction either:
/// - pushes values onto the stack
/// - pops values off the stack and pushes a result
/// - reads/writes memory or storage
/// - changes the program counter (jumps)
///
/// Reference: https://evm.codes
use std::collections::HashMap;

// ============================================================
// We use u256 as [u8; 32] for simplicity. In production you'd
// use `ruint` or `primitive_types::U256`. For learning, rolling
// your own helps you understand why 256-bit math matters.
// ============================================================

/// A 256-bit unsigned integer, stored big-endian in 32 bytes.
/// The EVM's native word size — every stack slot is one of these.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct U256(pub [u8; 32]);

impl U256 {
    pub const ZERO: Self = Self([0u8; 32]);
    pub const ONE: Self = Self::from_u64(1);
    pub const MAX: Self = Self([0xFF; 32]);

    /// Create from a u64 (most test values fit in 8 bytes).
    pub const fn from_u64(v: u64) -> Self {
        let mut bytes = [0u8; 32];
        let vb = v.to_be_bytes();

        bytes[24] = vb[0];
        bytes[25] = vb[1];
        bytes[26] = vb[2];
        bytes[27] = vb[3];
        bytes[28] = vb[4];
        bytes[29] = vb[5];
        bytes[30] = vb[6];
        bytes[31] = vb[7];
        Self(bytes)
    }

    /// Convert to u64, panicking if the value doesn't fit.
    pub fn as_u64(&self) -> u64 {
        for i in 0..24 {
            assert!(self.0[i] == 0, "U256 value too large for u64");
        }
        u64::from_be_bytes(self.0[24..32].try_into().unwrap())
    }

    /// Convert to usize (for memory offsets).
    pub fn as_usize(&self) -> usize {
        self.as_u64() as usize
    }

    // --------------------------------------------------------
    // TODO (Exercise 1a): Implement wrapping_add, wrapping_mul,
    // wrapping_sub, wrapping_div, is_zero, bitwise_and,
    // bitwise_or, bitwise_not, less_than, greater_than, equal.
    //
    // Hint: for add/sub/mul, work byte-by-byte from the least
    // significant byte (index 31) to most significant (index 0),
    // carrying overflow. Or convert to u128 pairs.
    // --------------------------------------------------------

    /// a + b, wrapping on overflow (modulo 2^256)
    pub fn wrapping_add(self, other: Self) -> Self {
        let mut result = [0u8; 32];
        let mut carry = 0;

        for i in (0..32).rev() {
            let sum = self.0[i] as u16 + other.0[i] as u16 + carry;
            result[i] = sum as u8;
            carry = sum >> 8;
        }

        U256(result)
    }

    /// a - b, wrapping on underflow
    pub fn wrapping_sub(self, other: Self) -> Self {
        let mut result = [0u8; 32];
        let mut borrow = 0;

        for i in (0..32).rev() {
            let diff = self.0[i] as i16 - other.0[i] as i16 - borrow;
            if diff < 0 {
                result[i] = (diff + 256) as u8;
                borrow = 1;
            } else {
                result[i] = diff as u8;
                borrow = 0;
            }
        }

        U256(result)
    }

    /// a * b, wrapping on overflow
    pub fn wrapping_mul(self, other: Self) -> Self {
        todo!("Exercise 1a: implement 256-bit wrapping multiplication")
    }

    /// a / b (integer division), returns 0 if b is zero (EVM spec)
    pub fn wrapping_div(self, other: Self) -> Self {
        todo!("Exercise 1a: implement 256-bit division")
    }

    /// a % b (modulo), returns 0 if b is zero
    pub fn wrapping_mod(self, other: Self) -> Self {
        todo!("Exercise 1a: implement 256-bit modulo")
    }

    pub fn is_zero(&self) -> bool {
        todo!("Exercise 1a: check if all 32 bytes are zero")
    }

    pub fn bitwise_and(self, other: Self) -> Self {
        todo!("Exercise 1a: AND each byte pair")
    }

    pub fn bitwise_or(self, other: Self) -> Self {
        todo!("Exercise 1a: OR each byte pair")
    }

    pub fn bitwise_not(self) -> Self {
        todo!("Exercise 1a: NOT each byte")
    }

    /// self < other (unsigned comparison)
    pub fn less_than(&self, other: &Self) -> bool {
        todo!("Exercise 1a: compare big-endian byte arrays")
    }

    /// self > other (unsigned comparison)
    pub fn greater_than(&self, other: &Self) -> bool {
        todo!("Exercise 1a: compare big-endian byte arrays")
    }

    /// self == other
    pub fn equal(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Display for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print as hex, stripping leading zeros
        let hex: String = self.0.iter().map(|b| format!("{:02x}", b)).collect();
        let trimmed = hex.trim_start_matches('0');
        if trimmed.is_empty() {
            write!(f, "0x0")
        } else {
            write!(f, "0x{}", trimmed)
        }
    }
}

// ============================================================
// Opcodes — these are the instruction set of the EVM.
// Reference: https://evm.codes
// ============================================================

#[allow(dead_code)]
pub mod opcodes {
    // Stop and arithmetic
    pub const STOP: u8 = 0x00;
    pub const ADD: u8 = 0x01;
    pub const MUL: u8 = 0x02;
    pub const SUB: u8 = 0x03;
    pub const DIV: u8 = 0x04;
    pub const MOD: u8 = 0x06;
    pub const ADDMOD: u8 = 0x08;
    pub const MULMOD: u8 = 0x09;

    // Comparison and bitwise
    pub const LT: u8 = 0x10;
    pub const GT: u8 = 0x11;
    pub const EQ: u8 = 0x14;
    pub const ISZERO: u8 = 0x15;
    pub const AND: u8 = 0x16;
    pub const OR: u8 = 0x17;
    pub const NOT: u8 = 0x19;

    // Stack, memory, storage
    pub const POP: u8 = 0x50;
    pub const MLOAD: u8 = 0x51;
    pub const MSTORE: u8 = 0x52;
    pub const MSTORE8: u8 = 0x53;
    pub const SLOAD: u8 = 0x54;
    pub const SSTORE: u8 = 0x55;
    pub const JUMP: u8 = 0x56;
    pub const JUMPI: u8 = 0x57;
    pub const PC: u8 = 0x58;
    pub const MSIZE: u8 = 0x59;
    pub const JUMPDEST: u8 = 0x5b;

    // Push operations (PUSH1 through PUSH32)
    pub const PUSH1: u8 = 0x60;
    pub const PUSH2: u8 = 0x61;
    pub const PUSH32: u8 = 0x7f;

    // Dup operations
    pub const DUP1: u8 = 0x80;
    pub const DUP16: u8 = 0x8f;

    // Swap operations
    pub const SWAP1: u8 = 0x90;
    pub const SWAP16: u8 = 0x9f;

    // System
    pub const RETURN: u8 = 0xf3;
    pub const REVERT: u8 = 0xfd;
    pub const INVALID: u8 = 0xfe;
}

// ============================================================
// The EVM itself
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionResult {
    Stop,
    Return(Vec<u8>),
    Revert(Vec<u8>),
    InvalidOpcode(u8),
    StackUnderflow,
    StackOverflow,
    InvalidJump,
    OutOfGas,
}

pub struct Evm {
    /// The bytecode being executed
    pub bytecode: Vec<u8>,
    /// Program counter — index into bytecode
    pub pc: usize,
    /// The stack (max 1024 items per EVM spec)
    pub stack: Vec<U256>,
    /// Byte-addressable memory (dynamically sized)
    pub memory: Vec<u8>,
    /// Persistent storage (key → value, both 256-bit)
    pub storage: HashMap<U256, U256>,
    /// Gas remaining
    pub gas_remaining: u64,
    /// Whether execution has halted
    pub stopped: bool,
}

impl Evm {
    pub fn new(bytecode: Vec<u8>, gas: u64) -> Self {
        Self {
            bytecode,
            pc: 0,
            stack: Vec::with_capacity(1024),
            memory: Vec::new(),
            storage: HashMap::new(),
            gas_remaining: gas,
            stopped: false,
        }
    }

    /// Run until halted. Returns the execution result.
    pub fn run(&mut self) -> ExecutionResult {
        loop {
            if self.pc >= self.bytecode.len() {
                return ExecutionResult::Stop;
            }
            match self.step() {
                None => continue,
                Some(result) => return result,
            }
        }
    }

    /// Execute one instruction. Returns None to continue, Some to halt.
    fn step(&mut self) -> Option<ExecutionResult> {
        let opcode = self.bytecode[self.pc];

        // Gas accounting — simplified: 3 gas per instruction
        // (real EVM has per-opcode costs, SSTORE costs 20000, etc.)
        if self.gas_remaining < 3 {
            return Some(ExecutionResult::OutOfGas);
        }
        self.gas_remaining -= 3;

        match opcode {
            // ================================================
            // STOP (0x00) — halt execution
            // ================================================
            opcodes::STOP => {
                self.stopped = true;
                return Some(ExecutionResult::Stop);
            }

            // ================================================
            // TODO (Exercise 1b): Arithmetic opcodes
            //
            // ADD (0x01): pop a, pop b, push a + b (wrapping)
            // MUL (0x02): pop a, pop b, push a * b (wrapping)
            // SUB (0x03): pop a, pop b, push a - b (wrapping)
            // DIV (0x04): pop a, pop b, push a / b (0 if b==0)
            // MOD (0x06): pop a, pop b, push a % b (0 if b==0)
            //
            // Hint: use self.pop()? and self.push(value)?
            // ================================================
            opcodes::ADD => {
                todo!("Exercise 1b: pop two values, push their sum")
            }
            opcodes::MUL => {
                todo!("Exercise 1b: pop two values, push their product")
            }
            opcodes::SUB => {
                todo!("Exercise 1b: pop two values, push a - b")
            }
            opcodes::DIV => {
                todo!("Exercise 1b: pop two values, push a / b")
            }
            opcodes::MOD => {
                todo!("Exercise 1b: pop two values, push a % b")
            }

            // ================================================
            // TODO (Exercise 1c): Comparison and bitwise
            //
            // LT: push 1 if a < b, else 0
            // GT: push 1 if a > b, else 0
            // EQ: push 1 if a == b, else 0
            // ISZERO: push 1 if a == 0, else 0
            // AND, OR, NOT: bitwise operations
            // ================================================
            opcodes::LT => {
                todo!("Exercise 1c")
            }
            opcodes::GT => {
                todo!("Exercise 1c")
            }
            opcodes::EQ => {
                todo!("Exercise 1c")
            }
            opcodes::ISZERO => {
                todo!("Exercise 1c")
            }
            opcodes::AND => {
                todo!("Exercise 1c")
            }
            opcodes::OR => {
                todo!("Exercise 1c")
            }
            opcodes::NOT => {
                todo!("Exercise 1c")
            }

            // ================================================
            // TODO (Exercise 1d): PUSH1 through PUSH32
            //
            // PUSH1 reads 1 byte after the opcode and pushes it.
            // PUSH2 reads 2 bytes, etc, up to PUSH32.
            //
            // The bytes are read from bytecode[pc+1..pc+1+n]
            // and placed into a U256 (right-aligned, big-endian).
            // After execution, pc advances past the data bytes.
            //
            // Hint: the opcode value itself tells you n:
            //   n = opcode - PUSH1 + 1
            // ================================================
            op if op >= opcodes::PUSH1 && op <= opcodes::PUSH32 => {
                todo!("Exercise 1d: read n bytes from bytecode, push as U256")
            }

            // ================================================
            // TODO (Exercise 1e): Stack manipulation
            //
            // POP:   remove top of stack
            // DUP1:  duplicate top item (DUP2 = duplicate 2nd, etc.)
            // SWAP1: swap top with 2nd item (SWAP2 = swap with 3rd, etc.)
            //
            // Hint for DUP: n = opcode - DUP1 + 1, copy stack[len-n]
            // Hint for SWAP: n = opcode - SWAP1 + 1, swap stack[len-1] with stack[len-1-n]
            // ================================================
            opcodes::POP => {
                todo!("Exercise 1e")
            }
            op if op >= opcodes::DUP1 && op <= opcodes::DUP16 => {
                todo!("Exercise 1e")
            }
            op if op >= opcodes::SWAP1 && op <= opcodes::SWAP16 => {
                todo!("Exercise 1e")
            }

            // ================================================
            // TODO (Exercise 2a): Memory operations
            //
            // MLOAD:  pop offset, read 32 bytes from memory[offset..offset+32], push
            // MSTORE: pop offset, pop value, write 32 bytes to memory[offset..offset+32]
            // MSTORE8: pop offset, pop value, write lowest byte to memory[offset]
            //
            // Memory auto-extends with zeros when you access beyond its current size.
            // Use self.expand_memory(offset + 32) before reading/writing.
            // ================================================
            opcodes::MLOAD => {
                todo!("Exercise 2a")
            }
            opcodes::MSTORE => {
                todo!("Exercise 2a")
            }
            opcodes::MSTORE8 => {
                todo!("Exercise 2a")
            }
            opcodes::MSIZE => {
                let size = U256::from_u64(self.memory.len() as u64);
                let res = self.push(size);

                match res {
                    Ok(()) => {}
                    Err(Some(result)) => return Some(result),
                    Err(None) => unreachable!(),
                }
            }

            // ================================================
            // TODO (Exercise 2b): Storage operations
            //
            // SLOAD:  pop key, push storage[key] (default 0)
            // SSTORE: pop key, pop value, write storage[key] = value
            //
            // This is the persistent state that lives on-chain.
            // In a real EVM, SSTORE costs 20000 gas for a fresh write.
            // ================================================
            opcodes::SLOAD => {
                todo!("Exercise 2b")
            }
            opcodes::SSTORE => {
                todo!("Exercise 2b")
            }

            // ================================================
            // TODO (Exercise 2c): Control flow
            //
            // JUMP:  pop dest, set pc = dest (must land on JUMPDEST)
            // JUMPI: pop dest, pop condition. If condition != 0, jump.
            // JUMPDEST: no-op, just marks a valid jump target
            // PC: push current program counter
            //
            // IMPORTANT: after JUMP/JUMPI (when taken), do NOT
            // increment pc at the end — you've already set it.
            // ================================================
            opcodes::JUMP => {
                todo!("Exercise 2c")
            }
            opcodes::JUMPI => {
                todo!("Exercise 2c")
            }
            opcodes::JUMPDEST => {
                // Valid jump target — just advance pc
            }
            opcodes::PC => {
                let pc_val = U256::from_u64(self.pc as u64);
                let res = self.push(pc_val);

                match res {
                    Ok(()) => {}
                    Err(Some(result)) => return Some(result),
                    Err(None) => unreachable!(),
                }
            }

            // ================================================
            // RETURN / REVERT — read data from memory and halt
            // ================================================
            opcodes::RETURN => {
                let offset = self.pop().unwrap().as_usize();

                let size = self.pop().unwrap().as_usize();
                self.expand_memory(offset + size);
                let data = self.memory[offset..offset + size].to_vec();
                return Some(ExecutionResult::Return(data));
            }
            opcodes::REVERT => {
                let offset = self.pop().unwrap().as_usize();
                let size = self.pop().unwrap().as_usize();
                self.expand_memory(offset + size);
                let data = self.memory[offset..offset + size].to_vec();
                return Some(ExecutionResult::Revert(data));
            }

            opcodes::INVALID => {
                return Some(ExecutionResult::InvalidOpcode(opcode));
            }

            _ => {
                return Some(ExecutionResult::InvalidOpcode(opcode));
            }
        }

        // Advance program counter (most opcodes advance by 1)
        self.pc += 1;
        None
    }

    // ========================================================
    // Helper methods — these are given to you.
    // ========================================================

    fn pop(&mut self) -> Result<U256, Option<ExecutionResult>> {
        self.stack
            .pop()
            .ok_or(Some(ExecutionResult::StackUnderflow))
    }

    fn push(&mut self, value: U256) -> Result<(), Option<ExecutionResult>> {
        if self.stack.len() >= 1024 {
            return Err(Some(ExecutionResult::StackOverflow));
        }
        self.stack.push(value);
        Ok(())
    }

    fn expand_memory(&mut self, new_size: usize) {
        if new_size > self.memory.len() {
            // Round up to nearest 32-byte word (EVM spec)
            let padded = (new_size + 31) & !31;
            self.memory.resize(padded, 0);
        }
    }

    /// Peek at the top of the stack without removing it.
    pub fn peek(&self) -> Option<&U256> {
        self.stack.last()
    }

    /// Get a storage value (for testing).
    pub fn get_storage(&self, key: &U256) -> U256 {
        self.storage.get(key).copied().unwrap_or(U256::ZERO)
    }
}
