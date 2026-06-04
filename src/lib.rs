#![forbid(unsafe_code)]

//! Advanced ternary compilation pipeline with IR, register allocation, and code generation.

/// A ternary trit value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trit {
    Neg = -1,
    Zero = 0,
    Pos = 1,
}

impl Trit {
    pub fn to_i8(self) -> i8 {
        match self {
            Trit::Neg => -1,
            Trit::Zero => 0,
            Trit::Pos => 1,
        }
    }
}

/// A trybble (3 trits) ranging from -13 to +13.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trybble {
    pub trits: [Trit; 3],
}

impl Trybble {
    pub fn new(t0: Trit, t1: Trit, t2: Trit) -> Self {
        Self { trits: [t0, t1, t2] }
    }

    pub fn zero() -> Self {
        Self { trits: [Trit::Zero; 3] }
    }

    pub fn to_value(&self) -> i16 {
        let mut v: i16 = 0;
        let mut power: i16 = 1;
        for i in 0..3 {
            v += self.trits[i].to_i8() as i16 * power;
            power *= 3;
        }
        v
    }

    pub fn from_value(val: i16) -> Option<Self> {
        if val < -13 || val > 13 {
            return None;
        }
        let mut v = val;
        let mut trits = [Trit::Zero; 3];
        for i in 0..3 {
            let rem = ((v + 1).rem_euclid(3) - 1) as i8;
            trits[i] = match rem {
                -1 => Trit::Neg,
                0 => Trit::Zero,
                1 => Trit::Pos,
                _ => return None,
            };
            v = (v - rem as i16) / 3;
        }
        if v == 0 { Some(Self { trits }) } else { None }
    }
}

// === Intermediate Representation ===

/// Operand in ternary IR.
#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    TritLit(Trit),
    TrybbleLit(Trybble),
    Reg(usize),
}

/// Ternary IR instruction opcodes.
#[derive(Clone, Debug, PartialEq)]
pub enum TernaryOp {
    Add,
    Sub,
    Mul,
    Not,
    Min,
    Max,
    Consensus, // majority vote of 3 trits
}

/// A single IR instruction.
#[derive(Clone, Debug, PartialEq)]
pub struct IRInstruction {
    pub op: TernaryOp,
    pub operands: Vec<Operand>,
    pub dest: usize, // destination register
}

impl IRInstruction {
    pub fn new(op: TernaryOp, operands: Vec<Operand>, dest: usize) -> Self {
        Self { op, operands, dest }
    }
}

/// Ternary IR — a sequence of instructions operating on virtual registers.
#[derive(Clone, Debug)]
pub struct TernaryIR {
    pub instructions: Vec<IRInstruction>,
    pub num_regs: usize,
}

impl TernaryIR {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            num_regs: 0,
        }
    }

    pub fn alloc_reg(&mut self) -> usize {
        let r = self.num_regs;
        self.num_regs += 1;
        r
    }

    pub fn emit(&mut self, instr: IRInstruction) {
        self.instructions.push(instr);
    }

    /// Build a simple add program: r2 = r0 + r1
    pub fn build_add_program(&mut self) {
        let r0 = self.alloc_reg();
        let r1 = self.alloc_reg();
        let r2 = self.alloc_reg();
        self.emit(IRInstruction::new(
            TernaryOp::Add,
            vec![Operand::Reg(r0), Operand::Reg(r1)],
            r2,
        ));
    }

    /// Optimize: remove dead code (registers written but never read).
    pub fn dead_code_elimination(&mut self) -> usize {
        let mut used: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for instr in &self.instructions {
            for op in &instr.operands {
                if let Operand::Reg(r) = op {
                    used.insert(*r);
                }
            }
        }
        // Check if dest register is used as a source in any other instruction
        // Keep the last instruction (its dest is the output)
        let orig_len = self.instructions.len();
        self.instructions.retain(|instr| {
            used.contains(&instr.dest) || instr.dest == self.num_regs - 1
        });
        orig_len - self.instructions.len()
    }

    /// Constant folding: replace operations on literals with computed result.
    pub fn constant_folding(&mut self) -> usize {
        let mut replacements = 0;
        let mut const_map: std::collections::HashMap<usize, Trybble> = std::collections::HashMap::new();
        
        for instr in &mut self.instructions {
            let mut all_const = true;
            let mut vals = Vec::new();
            for op in &instr.operands {
                match op {
                    Operand::TritLit(t) => vals.push(t.to_i8() as i16),
                    Operand::TrybbleLit(t) => vals.push(t.to_value()),
                    Operand::Reg(r) => {
                        if let Some(v) = const_map.get(r) {
                            vals.push(v.to_value());
                        } else {
                            all_const = false;
                        }
                    }
                }
            }
            if all_const && !vals.is_empty() {
                let result = match instr.op {
                    TernaryOp::Add => vals.iter().sum(),
                    TernaryOp::Sub => vals[0] - vals.get(1).copied().unwrap_or(0),
                    TernaryOp::Mul => vals.iter().product(),
                    TernaryOp::Not => -vals[0],
                    TernaryOp::Min => *vals.iter().min().unwrap(),
                    TernaryOp::Max => *vals.iter().max().unwrap(),
                    TernaryOp::Consensus => {
                        let sum: i16 = vals.iter().sum();
                        sum.signum()
                    }
                };
                if let Some(t) = Trybble::from_value(result) {
                    instr.op = TernaryOp::Add; // will be replaced
                    instr.operands = vec![Operand::TrybbleLit(t)];
                    replacements += 1;
                    const_map.insert(instr.dest, t);
                }
            }
        }
        replacements
    }
}

// === Instruction Selection ===

/// Target-level ternary bytecode instructions.
#[derive(Clone, Debug, PartialEq)]
pub enum BytecodeOp {
    LoadTrit(i8),       // load trit literal
    LoadTrybble(i16),   // load trybble literal
    AddReg(usize, usize),
    SubReg(usize, usize),
    MulReg(usize, usize),
    NotReg(usize),
    MinReg(usize, usize),
    MaxReg(usize, usize),
    ConsensusReg(usize, usize, usize),
    Store(usize),
    Halt,
}

/// Instruction selector maps IR to bytecode.
pub struct InstructionSelector;

impl InstructionSelector {
    pub fn new() -> Self {
        Self
    }

    pub fn select(&self, ir: &TernaryIR) -> Vec<BytecodeOp> {
        let mut code = Vec::new();
        for instr in &ir.instructions {
            match &instr.op {
                TernaryOp::Add => {
                    if let (Some(Operand::Reg(a)), Some(Operand::Reg(b))) = (&instr.operands.get(0), &instr.operands.get(1)) {
                        code.push(BytecodeOp::AddReg(*a, *b));
                        code.push(BytecodeOp::Store(instr.dest));
                    }
                }
                TernaryOp::Sub => {
                    if let (Some(Operand::Reg(a)), Some(Operand::Reg(b))) = (&instr.operands.get(0), &instr.operands.get(1)) {
                        code.push(BytecodeOp::SubReg(*a, *b));
                        code.push(BytecodeOp::Store(instr.dest));
                    }
                }
                TernaryOp::Mul => {
                    if let (Some(Operand::Reg(a)), Some(Operand::Reg(b))) = (&instr.operands.get(0), &instr.operands.get(1)) {
                        code.push(BytecodeOp::MulReg(*a, *b));
                        code.push(BytecodeOp::Store(instr.dest));
                    }
                }
                TernaryOp::Not => {
                    if let Some(Operand::Reg(a)) = &instr.operands.get(0) {
                        code.push(BytecodeOp::NotReg(*a));
                        code.push(BytecodeOp::Store(instr.dest));
                    }
                }
                TernaryOp::Min => {
                    if let (Some(Operand::Reg(a)), Some(Operand::Reg(b))) = (&instr.operands.get(0), &instr.operands.get(1)) {
                        code.push(BytecodeOp::MinReg(*a, *b));
                        code.push(BytecodeOp::Store(instr.dest));
                    }
                }
                TernaryOp::Max => {
                    if let (Some(Operand::Reg(a)), Some(Operand::Reg(b))) = (&instr.operands.get(0), &instr.operands.get(1)) {
                        code.push(BytecodeOp::MaxReg(*a, *b));
                        code.push(BytecodeOp::Store(instr.dest));
                    }
                }
                TernaryOp::Consensus => {
                    let regs: Vec<usize> = instr.operands.iter().filter_map(|o| {
                        if let Operand::Reg(r) = o { Some(*r) } else { None }
                    }).collect();
                    if regs.len() >= 3 {
                        code.push(BytecodeOp::ConsensusReg(regs[0], regs[1], regs[2]));
                        code.push(BytecodeOp::Store(instr.dest));
                    }
                }
            }
        }
        code.push(BytecodeOp::Halt);
        code
    }
}

// === Register Allocator ===

/// Maps virtual registers to physical ternary registers.
pub struct RegisterAllocator {
    pub num_physical: usize,
    pub allocation: std::collections::HashMap<usize, usize>,
    next_physical: usize,
}

impl RegisterAllocator {
    pub fn new(num_physical: usize) -> Self {
        Self {
            num_physical,
            allocation: std::collections::HashMap::new(),
            next_physical: 0,
        }
    }

    /// Allocate a physical register for a virtual register.
    pub fn allocate(&mut self, vreg: usize) -> Option<usize> {
        if let Some(&preg) = self.allocation.get(&vreg) {
            return Some(preg);
        }
        if self.next_physical < self.num_physical {
            let preg = self.next_physical;
            self.next_physical += 1;
            self.allocation.insert(vreg, preg);
            Some(preg)
        } else {
            None // spill needed
        }
    }

    /// Allocate all registers in an IR program.
    pub fn allocate_ir(&mut self, ir: &TernaryIR) -> bool {
        for instr in &ir.instructions {
            for op in &instr.operands {
                if let Operand::Reg(r) = op {
                    if self.allocate(*r).is_none() {
                        return false;
                    }
                }
            }
            if self.allocate(instr.dest).is_none() {
                return false;
            }
        }
        true
    }

    pub fn is_spilled(&self) -> bool {
        self.allocation.len() > self.num_physical
    }

    /// Remap a virtual register to physical.
    pub fn remap(&self, vreg: usize) -> Option<usize> {
        self.allocation.get(&vreg).copied()
    }
}

// === Code Generator ===

/// Generates balanced ternary bytecode from selected instructions.
pub struct CodeGenerator;

impl CodeGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a vector of trit-encoded instructions.
    pub fn generate(&self, bytecode: &[BytecodeOp]) -> Vec<Trit> {
        let mut output = Vec::new();
        for op in bytecode {
            match op {
                BytecodeOp::LoadTrit(v) => {
                    output.push(Trit::Pos); // opcode
                    output.push(match v {
                        -1 => Trit::Neg,
                        0 => Trit::Zero,
                        1 => Trit::Pos,
                        _ => Trit::Zero,
                    });
                }
                BytecodeOp::LoadTrybble(v) => {
                    output.push(Trit::Neg); // opcode
                    if let Some(t) = Trybble::from_value(*v) {
                        output.extend_from_slice(&t.trits);
                    } else {
                        output.extend_from_slice(&[Trit::Zero; 3]);
                    }
                }
                BytecodeOp::AddReg(_, _) => {
                    output.extend_from_slice(&[Trit::Zero, Trit::Pos]); // 00+1
                }
                BytecodeOp::SubReg(_, _) => {
                    output.extend_from_slice(&[Trit::Zero, Trit::Neg]); // 00-1
                }
                BytecodeOp::MulReg(_, _) => {
                    output.extend_from_slice(&[Trit::Pos, Trit::Pos]); // +1+1
                }
                BytecodeOp::NotReg(_) => {
                    output.extend_from_slice(&[Trit::Pos, Trit::Neg]); // +1-1
                }
                BytecodeOp::MinReg(_, _) => {
                    output.extend_from_slice(&[Trit::Neg, Trit::Pos]); // -1+1
                }
                BytecodeOp::MaxReg(_, _) => {
                    output.extend_from_slice(&[Trit::Neg, Trit::Neg]); // -1-1
                }
                BytecodeOp::ConsensusReg(_, _, _) => {
                    output.extend_from_slice(&[Trit::Zero, Trit::Zero]); // 00
                }
                BytecodeOp::Store(_) => {
                    output.push(Trit::Zero);
                }
                BytecodeOp::Halt => {
                    output.extend_from_slice(&[Trit::Zero, Trit::Zero, Trit::Zero]);
                }
            }
        }
        output
    }
}

// === Optimization: Trit Merging ===

/// Merge consecutive identical trit operations.
pub fn trit_merge(code: &mut Vec<Trit>) -> usize {
    let mut removed = 0;
    let mut i = 0;
    while i + 1 < code.len() {
        if code[i] == code[i + 1] && code[i] != Trit::Zero {
            // Two identical non-zero trits can be merged
            code.remove(i + 1);
            removed += 1;
        } else {
            i += 1;
        }
    }
    removed
}

// === Simple Interpreter ===

/// Interpret bytecode on a simple ternary VM.
pub struct TernaryVM {
    pub regs: Vec<i16>,
    pub pc: usize,
    pub halted: bool,
}

impl TernaryVM {
    pub fn new(num_regs: usize) -> Self {
        Self {
            regs: vec![0; num_regs],
            pc: 0,
            halted: false,
        }
    }

    pub fn set_reg(&mut self, r: usize, val: i16) {
        if r < self.regs.len() {
            self.regs[r] = val;
        }
    }

    /// Execute one instruction from bytecode.
    pub fn step(&mut self, bytecode: &[BytecodeOp]) -> bool {
        if self.halted || self.pc >= bytecode.len() {
            self.halted = true;
            return false;
        }
        match &bytecode[self.pc] {
            BytecodeOp::LoadTrit(v) => {
                // Load into implicit accumulator (reg 0)
                self.regs[0] = *v as i16;
                self.pc += 1;
            }
            BytecodeOp::LoadTrybble(v) => {
                self.regs[0] = *v;
                self.pc += 1;
            }
            BytecodeOp::AddReg(a, b) => {
                let result = self.regs.get(*a).copied().unwrap_or(0)
                    + self.regs.get(*b).copied().unwrap_or(0);
                self.regs[0] = result;
                self.pc += 1;
            }
            BytecodeOp::SubReg(a, b) => {
                let result = self.regs.get(*a).copied().unwrap_or(0)
                    - self.regs.get(*b).copied().unwrap_or(0);
                self.regs[0] = result;
                self.pc += 1;
            }
            BytecodeOp::MulReg(a, b) => {
                let result = self.regs.get(*a).copied().unwrap_or(0)
                    * self.regs.get(*b).copied().unwrap_or(0);
                self.regs[0] = result;
                self.pc += 1;
            }
            BytecodeOp::NotReg(a) => {
                self.regs[0] = -self.regs.get(*a).copied().unwrap_or(0);
                self.pc += 1;
            }
            BytecodeOp::MinReg(a, b) => {
                let va = self.regs.get(*a).copied().unwrap_or(0);
                let vb = self.regs.get(*b).copied().unwrap_or(0);
                self.regs[0] = va.min(vb);
                self.pc += 1;
            }
            BytecodeOp::MaxReg(a, b) => {
                let va = self.regs.get(*a).copied().unwrap_or(0);
                let vb = self.regs.get(*b).copied().unwrap_or(0);
                self.regs[0] = va.max(vb);
                self.pc += 1;
            }
            BytecodeOp::ConsensusReg(a, b, c) => {
                let sum = self.regs.get(*a).copied().unwrap_or(0)
                    + self.regs.get(*b).copied().unwrap_or(0)
                    + self.regs.get(*c).copied().unwrap_or(0);
                self.regs[0] = sum.signum();
                self.pc += 1;
            }
            BytecodeOp::Store(r) => {
                if *r < self.regs.len() {
                    self.regs[*r] = self.regs[0];
                }
                self.pc += 1;
            }
            BytecodeOp::Halt => {
                self.halted = true;
            }
        }
        true
    }

    /// Run until halt or end.
    pub fn run(&mut self, bytecode: &[BytecodeOp]) {
        while self.step(bytecode) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trybble_roundtrip() {
        for v in -13i16..=13 {
            let t = Trybble::from_value(v).unwrap();
            assert_eq!(t.to_value(), v, "Failed for value {}", v);
        }
    }

    #[test]
    fn test_trybble_out_of_range() {
        assert!(Trybble::from_value(14).is_none());
        assert!(Trybble::from_value(-14).is_none());
    }

    #[test]
    fn test_ir_alloc_reg() {
        let mut ir = TernaryIR::new();
        let r0 = ir.alloc_reg();
        let r1 = ir.alloc_reg();
        assert_eq!(r0, 0);
        assert_eq!(r1, 1);
        assert_eq!(ir.num_regs, 2);
    }

    #[test]
    fn test_ir_build_add() {
        let mut ir = TernaryIR::new();
        ir.build_add_program();
        assert_eq!(ir.instructions.len(), 1);
        assert_eq!(ir.instructions[0].op, TernaryOp::Add);
    }

    #[test]
    fn test_instruction_selector() {
        let mut ir = TernaryIR::new();
        ir.build_add_program();
        let selector = InstructionSelector::new();
        let bytecode = selector.select(&ir);
        assert!(!bytecode.is_empty());
        assert_eq!(bytecode.last(), Some(&BytecodeOp::Halt));
    }

    #[test]
    fn test_register_allocator_basic() {
        let mut ra = RegisterAllocator::new(8);
        let p0 = ra.allocate(0);
        let p1 = ra.allocate(1);
        assert_eq!(p0, Some(0));
        assert_eq!(p1, Some(1));
    }

    #[test]
    fn test_register_allocator_spill() {
        let mut ra = RegisterAllocator::new(2);
        ra.allocate(0);
        ra.allocate(1);
        let spilled = ra.allocate(2);
        assert_eq!(spilled, None);
    }

    #[test]
    fn test_register_allocator_reuse() {
        let mut ra = RegisterAllocator::new(4);
        ra.allocate(0);
        let p = ra.allocate(0); // re-allocate same vreg
        assert_eq!(p, Some(0));
    }

    #[test]
    fn test_register_allocator_ir() {
        let mut ir = TernaryIR::new();
        ir.build_add_program();
        let mut ra = RegisterAllocator::new(8);
        assert!(ra.allocate_ir(&ir));
    }

    #[test]
    fn test_code_generator() {
        let bytecode = vec![BytecodeOp::LoadTrit(1), BytecodeOp::Halt];
        let gen = CodeGenerator::new();
        let code = gen.generate(&bytecode);
        assert!(!code.is_empty());
    }

    #[test]
    fn test_trit_merge() {
        let mut code = vec![Trit::Pos, Trit::Pos, Trit::Neg, Trit::Neg, Trit::Zero];
        let removed = trit_merge(&mut code);
        assert!(removed > 0);
    }

    #[test]
    fn test_vm_add() {
        let bytecode = vec![
            BytecodeOp::AddReg(0, 1),
            BytecodeOp::Store(2),
            BytecodeOp::Halt,
        ];
        let mut vm = TernaryVM::new(4);
        vm.set_reg(0, 5);
        vm.set_reg(1, 3);
        vm.run(&bytecode);
        assert_eq!(vm.regs[2], 8);
    }

    #[test]
    fn test_vm_sub() {
        let bytecode = vec![
            BytecodeOp::SubReg(0, 1),
            BytecodeOp::Store(2),
            BytecodeOp::Halt,
        ];
        let mut vm = TernaryVM::new(4);
        vm.set_reg(0, 5);
        vm.set_reg(1, 3);
        vm.run(&bytecode);
        assert_eq!(vm.regs[2], 2);
    }

    #[test]
    fn test_vm_mul() {
        let bytecode = vec![
            BytecodeOp::MulReg(0, 1),
            BytecodeOp::Store(2),
            BytecodeOp::Halt,
        ];
        let mut vm = TernaryVM::new(4);
        vm.set_reg(0, 4);
        vm.set_reg(1, 3);
        vm.run(&bytecode);
        assert_eq!(vm.regs[2], 12);
    }

    #[test]
    fn test_vm_not() {
        let bytecode = vec![
            BytecodeOp::NotReg(0),
            BytecodeOp::Store(1),
            BytecodeOp::Halt,
        ];
        let mut vm = TernaryVM::new(4);
        vm.set_reg(0, 5);
        vm.run(&bytecode);
        assert_eq!(vm.regs[1], -5);
    }

    #[test]
    fn test_vm_min_max() {
        let mut vm = TernaryVM::new(4);
        vm.set_reg(0, 3);
        vm.set_reg(1, 7);
        let bc = vec![BytecodeOp::MinReg(0, 1), BytecodeOp::Store(2), BytecodeOp::Halt];
        vm.run(&bc);
        assert_eq!(vm.regs[2], 3);

        let mut vm2 = TernaryVM::new(4);
        vm2.set_reg(0, 3);
        vm2.set_reg(1, 7);
        let bc2 = vec![BytecodeOp::MaxReg(0, 1), BytecodeOp::Store(2), BytecodeOp::Halt];
        vm2.run(&bc2);
        assert_eq!(vm2.regs[2], 7);
    }

    #[test]
    fn test_vm_consensus() {
        let bytecode = vec![
            BytecodeOp::ConsensusReg(0, 1, 2),
            BytecodeOp::Store(3),
            BytecodeOp::Halt,
        ];
        let mut vm = TernaryVM::new(4);
        vm.set_reg(0, 1);
        vm.set_reg(1, 1);
        vm.set_reg(2, -1);
        vm.run(&bytecode);
        assert_eq!(vm.regs[3], 1); // majority positive
    }

    #[test]
    fn test_constant_folding() {
        let mut ir = TernaryIR::new();
        let r0 = ir.alloc_reg();
        let r1 = ir.alloc_reg();
        let r2 = ir.alloc_reg();
        ir.emit(IRInstruction::new(
            TernaryOp::Add,
            vec![Operand::TritLit(Trit::Pos), Operand::TritLit(Trit::Pos)],
            r2,
        ));
        let folded = ir.constant_folding();
        assert!(folded > 0);
    }

    #[test]
    fn test_full_pipeline() {
        let mut ir = TernaryIR::new();
        ir.build_add_program();
        let selector = InstructionSelector::new();
        let bytecode = selector.select(&ir);
        let gen = CodeGenerator::new();
        let code = gen.generate(&bytecode);
        assert!(!code.is_empty());
    }

    #[test]
    fn test_vm_load_and_store() {
        let bytecode = vec![
            BytecodeOp::LoadTrit(1),
            BytecodeOp::Store(2),
            BytecodeOp::LoadTrybble(5),
            BytecodeOp::Store(3),
            BytecodeOp::Halt,
        ];
        let mut vm = TernaryVM::new(4);
        vm.run(&bytecode);
        assert_eq!(vm.regs[2], 1);
        assert_eq!(vm.regs[3], 5);
        assert!(vm.halted);
    }
}
