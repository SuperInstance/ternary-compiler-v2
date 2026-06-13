# Ternary Compiler v2 — Advanced Balanced-Ternary Compilation Pipeline

**Ternary Compiler v2** is an advanced compilation pipeline for balanced-ternary (base-3) computation, featuring a three-address intermediate representation (IR), ternary register allocation, and balanced-ternary code generation. It operates on the alphabet {-1, 0, +1} where each trit encodes negative, neutral, or positive — a representation that maps naturally to three-valued logic, ternary neural networks, and Z₃ cyclic systems.

## Why It Matters

Binary computation has been the dominant paradigm for decades, but balanced ternary is mathematically the most efficient integer base (radix economy ≈ 1.004, lower than binary's 1.061). Ternary hardware research has seen a resurgence because ternary neural networks achieve 16× memory density over FP32 using just 2 bits per weight. This compiler provides the tooling to target such architectures: it lowers ternary logic expressions into an IR, allocates ternary registers (trybbles of 3 trits, range -13 to +13), and emits executable ternary instruction sequences. Without a proper compiler, programming ternary hardware would require hand-writing every {-1, 0, +1} operation.

## How It Works

The pipeline has three stages: **front-end parsing** (produces ternary expressions), **IR generation** (three-address code over ternary operands), and **code generation** (register allocation + emission).

### Trits and Trybbles

The fundamental unit is a **trit** with values {-1, 0, +1}. A **trybble** is a group of 3 trits representing integers from -13 to +13 using balanced ternary positional notation:

```
value = t₀ × 3⁰ + t₁ × 3¹ + t₂ × 3²   where tᵢ ∈ {-1, 0, +1}
```

The conversion from integer to balanced ternary uses balanced division: `v = ((v + 1) mod 3) - 1`, then `v = (v - remainder) / 3`, repeated for each digit. This is O(k) for k trits.

### Intermediate Representation

The IR uses three-address form with ternary operands. Each instruction is a `TernaryOp` (Add, Mul, Cmp, Load, Store, Branch, Halt) operating on `Operand` values (trit literals, trybble literals, or register references). The IR preserves ternary semantics throughout — for example, multiplication of two trybbles produces a trybble result modulo the balanced range.

### Register Allocation

Registers are allocated as trybble slots. The allocator tracks live ranges and performs linear-scan assignment in O(n) time for n instructions, mapping virtual registers to physical trybble slots. Spill code uses zero-page trybble addresses.

## Quick Start

```rust
use ternary_compiler_v2::{Trit, Trybble, Operand, TernaryOp};

// Create a trybble representing the value +5
let trybble = Trybble::from_value(5).unwrap();
assert_eq!(trybble.to_value(), 5);

// Build IR: r0 = +1 * r1
let instr = TernaryOp::Mul(
    Operand::Reg(0),
    Operand::TritLit(Trit::Pos),
    Operand::Reg(1),
);
```

```bash
cargo add ternary-compiler-v2
cargo build
```

## API

| Type / Function | Description |
|---|---|
| `Trit` | Enum with `Neg`, `Zero`, `Pos` variants |
| `Trybble` | 3-trit word (range -13 to +13) with `to_value()` / `from_value()` |
| `Operand` | IR operand: trit literal, trybble literal, or register |
| `TernaryOp` | IR opcode: Add, Mul, Cmp, Load, Store, Branch, Halt |
| `Trybble::from_value(i16)` | Convert integer to balanced ternary (O(k)) |

## Architecture Notes

This crate is part of the **SuperInstance** ecosystem, where ternary computation underpins the conservation law **γ + η = C** (growth plus entropy equals a constant). The compiler enables executable ternary programs that satisfy this invariant by construction — every operation stays within Z₃ algebra. See the [Architecture Document](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for how the compilation pipeline feeds into fleet-wide ternary execution.

## References

- Knuth, Donald E. *The Art of Computer Programming, Vol. 2: Seminumerical Algorithms*, §4.1 — balanced ternary number system.
- Frieder, G. & Luk, C. "Fibonacci and Ternary Computers," *AFIPS Conference Proceedings*, 1975.
- Li, Feng et al. "Ternary Weight Networks" (TWN), *arXiv:1605.04711*, 2016 — 2-bit ternary weights for neural networks.

## License

MIT
