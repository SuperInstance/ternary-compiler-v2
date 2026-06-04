# Future Integration: ternary-compiler-v2

## Current State
Provides an advanced ternary compilation pipeline with intermediate representation (IR), trybble operations, register allocation, ternary opcodes, and code generation targeting ternary hardware.

## Integration Opportunities

### With ternary-hardware (Code Generation Target)
The compiler needs a target. `ternary-hardware` defines TernaryALU, TernaryMemory, and TernaryRegister. `ternary-compiler-v2` generates IR that maps to these hardware primitives. Together: source code → ternary IR → register allocation → machine code for TernaryALU. This is the GCC of ternary computing.

### With ternary-compiler-python (Cross-Language Pipeline)
Python compiler compiles strategies into lookup tables. Rust compiler v2 compiles into optimized ternary IR. Together: Python for rapid prototyping of strategies, Rust v2 for production optimization. The Python output can be an input to the Rust pipeline — start with a lookup table, optimize it through IR passes, emit efficient ternary machine code.

### With compiled-policy-c (Embedded Deployment)
Compile ternary policies for microcontrollers: policy specification → ternary IR → dead code elimination → register allocation → C output via compiled-policy-c. The v2 optimizer handles the hard part (optimization); compiled-policy-c handles the easy part (C emission for ESP32).

## Potential in Mature Systems
In room-as-codespace, the compiler is the toolchain for room specialization. Each room has specific computational needs; the compiler takes a generic agent description and compiles it for the room's target hardware: Codespace (full optimization), Jetson (GPU-aware), ESP32 (minimal, no heap). Multi-target compilation from a single source.

## Cross-Pollination Ideas
- IR optimization passes as room specialization phases
- Register allocation for minimizing agent memory footprint on constrained devices
- Dead code elimination as "skill pruning" — remove capabilities the room doesn't need

## Dependencies for Next Steps
- ternary-hardware as the code generation target
- ternary-compiler-python bridge for cross-language pipeline
- compiled-policy-c backend for C emission from IR
