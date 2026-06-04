# ternary-compiler-v2

A compilation pipeline for balanced ternary programs: IR generation, register allocation, instruction selection, trit-encoded bytecode, optimization passes, and a stackless VM interpreter.

## Why This Exists

Binary compilers assume two states. Balanced ternary (where digits are −1, 0, +1 instead of 0 and 1) needs its own compilation pipeline: different arithmetic ops, different overflow behavior, and different encoding. This crate provides a complete (if minimal) frontend-to-bytecode path for ternary programs. You construct IR, optimize it, select target instructions, allocate registers, generate trit-encoded bytecode, and execute on a ternary VM. It's a teaching and experimentation platform for ternary computing, not a production compiler.

## Core Concepts

- **Trit** — A single balanced ternary digit: Neg (−1), Zero (0), or Pos (+1).
- **Trybble** — Three trits representing a value from −13 to +13. The balanced ternary encoding uses place values 1, 3, 9 (powers of 3).
- **TernaryIR** — Intermediate representation: a sequence of instructions operating on virtual registers. Each instruction has an opcode, operands (registers or literals), and a destination register.
- **TernaryOp** — Operations: Add, Sub, Mul, Not (negate), Min, Max, Consensus (majority vote of 3 trits).
- **BytecodeOp** — Target-level instructions: LoadTrit, LoadTrybble, arithmetic on physical registers, Store, Halt.
- **RegisterAllocator** — Maps virtual registers to a fixed pool of physical registers. Returns `None` (spill required) when the pool is exhausted.
- **CodeGenerator** — Encodes bytecode as a flat sequence of trits. Each opcode maps to a fixed trit pattern.
- **TernaryVM** — A simple interpreter that executes bytecode instructions on a register file. Operations use an implicit accumulator (register 0) as the working register.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-compiler-v2 = "0.1"
```

```rust
use ternary_compiler_v2::*;

fn main() {
    // Build IR for a simple add program: r2 = r0 + r1
    let mut ir = TernaryIR::new();
    ir.build_add_program();

    // Allocate registers
    let mut ra = RegisterAllocator::new(8);
    assert!(ra.allocate_ir(&ir));

    // Select target instructions
    let selector = InstructionSelector::new();
    let bytecode = selector.select(&ir);

    // Run on the VM
    let mut vm = TernaryVM::new(4);
    vm.set_reg(0, 7);
    vm.set_reg(1, 4);
    vm.run(&bytecode);
    println!("r2 = {}", vm.regs[2]); // r2 = 11
}
```

## API Overview

| Type | Description |
|------|-------------|
| `Trit` | Single balanced ternary digit (−1, 0, +1) |
| `Trybble` | 3-trit value (−13 to +13) |
| `TernaryIR` | IR program: instructions + virtual register pool |
| `IRInstruction` | Single IR instruction (opcode, operands, destination) |
| `TernaryOp` | IR opcodes: Add, Sub, Mul, Not, Min, Max, Consensus |
| `Operand` | IR operand: trit literal, trybble literal, or register |
| `InstructionSelector` | Maps IR to target bytecode |
| `BytecodeOp` | Target instruction: Load, arithmetic, Store, Halt |
| `RegisterAllocator` | Virtual-to-physical register mapping with spill detection |
| `CodeGenerator` | Bytecode → flat trit sequence |
| `TernaryVM` | Interpreter executing bytecode on a register file |

## How It Works

**IR construction:** Create a `TernaryIR`, allocate virtual registers with `alloc_reg()`, and emit instructions. Each instruction specifies an opcode, source operands (registers or literals), and a destination register.

**Optimization:** Two passes are available. `constant_folding()` replaces operations on all-literal operands with the computed result (tracked in a const_map for chaining). `dead_code_elimination()` removes instructions whose destination register is never read as a source by any other instruction (except the last instruction, whose output is preserved).

**Instruction selection:** The `InstructionSelector` walks IR instructions and emits `BytecodeOp` variants. Each binary IR op (Add, Sub, Mul, Min, Max) maps to a register-pair bytecode op followed by a Store. Unary ops (Not) follow the same pattern. Consensus takes three register operands.

**Register allocation:** Linear-scan approach: walk all instructions, allocate a physical register for each virtual register on first encounter. When physical registers are exhausted, `allocate()` returns `None` (spill needed, but no spilling logic is implemented — compilation fails).

**Code generation:** Each `BytecodeOp` maps to a fixed trit pattern. For example, `AddReg` encodes as `[Zero, Pos]`, `Halt` encodes as `[Zero, Zero, Zero]`. Literal values encode inline.

**VM execution:** The `TernaryVM` maintains a register file and program counter. All arithmetic ops write results to register 0 (implicit accumulator); `Store r` copies register 0 to register r. Execution stops at `Halt`.

## Known Limitations

- **No spill support.** When physical registers are exhausted, compilation fails. There's no spill/reload mechanism.
- **No control flow.** The IR and bytecode have no branch, jump, or conditional instructions. Programs are straight-line code only.
- **Accumulator-based VM.** All results go to register 0 first, then Store copies to the target. This is correct but generates extra instructions compared to a direct-register model.
- **Trit merge optimization is lossy.** `trit_merge()` removes consecutive identical non-zero trits, which changes the encoded program. Use with caution.
- **No function calls or stack.** The VM has no call stack, no return address register, and no calling convention.

## Use Cases

- **Ternary architecture simulation** — Compile and run small programs on a simulated ternary processor for research or education.
- **Hardware design reference** — The trit encoding scheme and instruction set can serve as a starting point for physical ternary processor design.
- **Compiler education** — The pipeline (IR → optimization → instruction selection → register allocation → codegen → execution) is a textbook compiler backend in miniature.

## Ecosystem Context

Part of the SuperInstance ternary crate family. `ternary-compiler-v2` sits at the infrastructure layer: it can compile programs that drive `ternary-cell` behavior, process data from `ternary-database`, or encode logic for `ternary-robotics` control systems. Its `Trit` and `Trybble` types are the fundamental ternary primitives used throughout the ecosystem.

## License

MIT
