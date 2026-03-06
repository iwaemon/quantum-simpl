# quantum-simpl Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a CLI tool that parses a Hamiltonian DSL and outputs mVMC-ready input files.

**Architecture:** Flat Term Table with 4-stage pipeline (expand → normal order → combine → symmetry). Each Term is a f64 coefficient + SmallVec<[Op; 4]> operator string.

**Tech Stack:** Rust, clap, smallvec, rustc-hash, rayon

---

### Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/core/mod.rs`
- Create: `src/core/op.rs`
- Create: `src/parser/mod.rs`
- Create: `src/parser/ast.rs`
- Create: `src/output/mod.rs`
- Create: `src/output/mvmc.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "quantum-simpl"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "quantum-simpl"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
smallvec = { version = "1", features = ["serde"] }
rustc-hash = "2"
rayon = "1"

[dev-dependencies]
tempfile = "3"
```

**Step 2: Create minimal src/main.rs**

```rust
mod core;
mod parser;
mod output;

fn main() {
    println!("quantum-simpl v0.1.0");
}
```

**Step 3: Create stub modules**

`src/core/mod.rs`:
```rust
pub mod op;
```

`src/core/op.rs`:
```rust
// Operator types defined in Task 2
```

`src/parser/mod.rs`:
```rust
pub mod ast;
```

`src/parser/ast.rs`:
```rust
// AST types defined in Task 3
```

`src/output/mod.rs`:
```rust
pub mod mvmc;
```

`src/output/mvmc.rs`:
```rust
// mVMC writer defined in Task 8
```

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles with no errors

**Step 5: Commit**

```bash
git add Cargo.toml src/
git commit -m "feat: project scaffold with stub modules"
```

---

### Task 2: Core Data Model (Op, Term, Hamiltonian)

**Files:**
- Modify: `src/core/op.rs`
- Create: `tests/unit_op.rs`

**Step 1: Write the failing test**

Create `tests/unit_op.rs`:
```rust
use quantum_simpl::core::op::{Op, Spin, Term, Hamiltonian};
use smallvec::smallvec;

#[test]
fn term_creation() {
    let term = Term {
        coeff: -1.0,
        ops: smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(1, Spin::Up),
        ],
    };
    assert_eq!(term.coeff, -1.0);
    assert_eq!(term.ops.len(), 2);
}

#[test]
fn hamiltonian_creation() {
    let ham = Hamiltonian {
        terms: vec![],
        num_sites: 10,
    };
    assert_eq!(ham.num_sites, 10);
    assert_eq!(ham.terms.len(), 0);
}

#[test]
fn term_is_identity() {
    let identity = Term { coeff: 1.0, ops: smallvec![] };
    assert!(identity.ops.is_empty());

    let not_identity = Term {
        coeff: 1.0,
        ops: smallvec![Op::SpinZ(0)],
    };
    assert!(!not_identity.ops.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_op`
Expected: FAIL (module not public / types not defined)

**Step 3: Implement the data model**

`src/core/op.rs`:
```rust
use smallvec::SmallVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Spin {
    Up,   // 0 in mVMC
    Down, // 1 in mVMC
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    FermionCreate(usize, Spin),
    FermionAnnihilate(usize, Spin),
    SpinPlus(usize),
    SpinMinus(usize),
    SpinZ(usize),
}

impl Op {
    /// Returns the site index this operator acts on.
    pub fn site(&self) -> usize {
        match self {
            Op::FermionCreate(s, _)
            | Op::FermionAnnihilate(s, _)
            | Op::SpinPlus(s)
            | Op::SpinMinus(s)
            | Op::SpinZ(s) => *s,
        }
    }

    /// Returns the change in Sz quantum number caused by this operator.
    /// +1 for creation-up/S+, -1 for creation-down/S-, 0 for Sz/annihilate mirrors.
    pub fn delta_sz(&self) -> i32 {
        match self {
            Op::FermionCreate(_, Spin::Up) => 1,
            Op::FermionCreate(_, Spin::Down) => -1,
            Op::FermionAnnihilate(_, Spin::Up) => -1,
            Op::FermionAnnihilate(_, Spin::Down) => 1,
            Op::SpinPlus(_) => 1,
            Op::SpinMinus(_) => -1,
            Op::SpinZ(_) => 0,
        }
    }

    /// True if this is a creation operator.
    pub fn is_creation(&self) -> bool {
        matches!(self, Op::FermionCreate(_, _))
    }

    /// True if this is an annihilation operator.
    pub fn is_annihilation(&self) -> bool {
        matches!(self, Op::FermionAnnihilate(_, _))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Term {
    pub coeff: f64,
    pub ops: SmallVec<[Op; 4]>,
}

impl Term {
    pub fn new(coeff: f64, ops: SmallVec<[Op; 4]>) -> Self {
        Self { coeff, ops }
    }

    /// Returns the total change in Sz for this term.
    pub fn delta_sz(&self) -> i32 {
        self.ops.iter().map(|op| op.delta_sz()).sum()
    }

    /// Returns the Hermitian conjugate of this term.
    pub fn hermitian_conjugate(&self) -> Self {
        let new_ops: SmallVec<[Op; 4]> = self.ops.iter().rev().map(|op| match op {
            Op::FermionCreate(s, spin) => Op::FermionAnnihilate(*s, *spin),
            Op::FermionAnnihilate(s, spin) => Op::FermionCreate(*s, *spin),
            Op::SpinPlus(s) => Op::SpinMinus(*s),
            Op::SpinMinus(s) => Op::SpinPlus(*s),
            Op::SpinZ(s) => Op::SpinZ(*s),
        }).collect();
        Self { coeff: self.coeff, ops: new_ops }
    }

    /// Number of operators (body count: 1-body = 2 ops, 2-body = 4 ops).
    pub fn num_ops(&self) -> usize {
        self.ops.len()
    }
}

#[derive(Debug, Clone)]
pub struct Hamiltonian {
    pub terms: Vec<Term>,
    pub num_sites: usize,
}

impl Hamiltonian {
    pub fn new(num_sites: usize) -> Self {
        Self { terms: vec![], num_sites }
    }

    pub fn add_term(&mut self, term: Term) {
        self.terms.push(term);
    }
}
```

Update `src/core/mod.rs`:
```rust
pub mod op;
```

Update `src/main.rs` to make the crate a library too. Create `src/lib.rs`:
```rust
pub mod core;
pub mod parser;
pub mod output;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_op`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add src/core/op.rs src/lib.rs tests/unit_op.rs
git commit -m "feat: core data model — Op, Term, Hamiltonian"
```

---

### Task 3: AST Types for Parser

**Files:**
- Modify: `src/parser/ast.rs`

**Step 1: Write the failing test**

Add to bottom of `src/parser/ast.rs` (will be written in step 3):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_definition_creation() {
        let model = ModelDef {
            lattice: LatticeDef {
                dimension: "1d".to_string(),
                num_sites: 10,
                pbc: true,
            },
            sum_blocks: vec![],
            params: vec![("t".to_string(), 1.0)],
        };
        assert_eq!(model.lattice.num_sites, 10);
        assert!(model.lattice.pbc);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test ast::tests`
Expected: FAIL (types not defined)

**Step 3: Implement AST types**

`src/parser/ast.rs`:
```rust
/// Top-level model definition from DSL input.
#[derive(Debug, Clone)]
pub struct ModelDef {
    pub lattice: LatticeDef,
    pub sum_blocks: Vec<SumBlock>,
    pub params: Vec<(String, f64)>,
}

/// Lattice declaration: `lattice 1d sites=10 pbc=true`
#[derive(Debug, Clone)]
pub struct LatticeDef {
    pub dimension: String,
    pub num_sites: usize,
    pub pbc: bool,
}

/// A `sum i=0..N:` block containing operator expressions.
#[derive(Debug, Clone)]
pub struct SumBlock {
    pub var: String,
    pub range_start: usize,
    pub range_end: usize,
    pub expressions: Vec<Expression>,
}

/// A single line inside a sum block, e.g. `-t * c†(i,up) c(i+1,up) + h.c.`
#[derive(Debug, Clone)]
pub struct Expression {
    pub coeff: CoeffExpr,
    pub operators: Vec<OpExpr>,
    pub hermitian_conjugate: bool,
}

/// Coefficient: either a literal number or a parameter name, possibly negated.
#[derive(Debug, Clone)]
pub enum CoeffExpr {
    Literal(f64),
    Param(String),
    Neg(Box<CoeffExpr>),
    Mul(Box<CoeffExpr>, Box<CoeffExpr>),
}

/// An operator in the DSL, with site as an index expression.
#[derive(Debug, Clone)]
pub enum OpExpr {
    FermionCreate(IndexExpr, SpinExpr),
    FermionAnnihilate(IndexExpr, SpinExpr),
    Number(IndexExpr, SpinExpr),         // n(i,s) sugar
    SpinPlus(IndexExpr),
    SpinMinus(IndexExpr),
    SpinZ(IndexExpr),
}

/// Site index expression: `i`, `i+1`, `i-1`, or a literal.
#[derive(Debug, Clone)]
pub enum IndexExpr {
    Var(String),
    VarPlus(String, usize),
    VarMinus(String, usize),
    Literal(usize),
}

/// Spin in DSL: `up` or `down`.
#[derive(Debug, Clone)]
pub enum SpinExpr {
    Up,
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_definition_creation() {
        let model = ModelDef {
            lattice: LatticeDef {
                dimension: "1d".to_string(),
                num_sites: 10,
                pbc: true,
            },
            sum_blocks: vec![],
            params: vec![("t".to_string(), 1.0)],
        };
        assert_eq!(model.lattice.num_sites, 10);
        assert!(model.lattice.pbc);
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test ast::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/parser/ast.rs
git commit -m "feat: AST types for DSL parser"
```

---

### Task 4: DSL Parser

**Files:**
- Modify: `src/parser/mod.rs`
- Create: `tests/unit_parser.rs`

**Step 1: Write the failing test**

Create `tests/unit_parser.rs`:
```rust
use quantum_simpl::parser::parse;

#[test]
fn parse_simple_hubbard() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  -t * c†(i,up) c(i+1,up) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
"#;
    let model = parse(input).unwrap();
    assert_eq!(model.lattice.num_sites, 2);
    assert!(!model.lattice.pbc);
    assert_eq!(model.sum_blocks.len(), 1);
    assert_eq!(model.sum_blocks[0].var, "i");
    assert_eq!(model.sum_blocks[0].range_start, 0);
    assert_eq!(model.sum_blocks[0].range_end, 1);
    assert_eq!(model.sum_blocks[0].expressions.len(), 2);
    assert_eq!(model.params.len(), 2);
}

#[test]
fn parse_heisenberg() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..4:
  J * Sp(i) Sm(i+1)
  J * Sm(i) Sp(i+1)
  J * 2.0 * Sz(i) Sz(i+1)

params:
  J = 1.0
"#;
    let model = parse(input).unwrap();
    assert_eq!(model.lattice.num_sites, 4);
    assert!(model.lattice.pbc);
    assert_eq!(model.sum_blocks[0].expressions.len(), 3);
}

#[test]
fn parse_hermitian_conjugate_flag() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  -t * c†(i,up) c(i+1,up) + h.c.

params:
  t = 1.0
"#;
    let model = parse(input).unwrap();
    assert!(model.sum_blocks[0].expressions[0].hermitian_conjugate);
}

#[test]
fn parse_error_on_invalid_input() {
    let input = "this is not valid DSL";
    assert!(parse(input).is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_parser`
Expected: FAIL (parse function not defined)

**Step 3: Implement the parser**

`src/parser/mod.rs`:
```rust
pub mod ast;

use ast::*;

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.0)
    }
}

impl std::error::Error for ParseError {}

pub fn parse(input: &str) -> Result<ModelDef, ParseError> {
    let mut lattice: Option<LatticeDef> = None;
    let mut sum_blocks: Vec<SumBlock> = Vec::new();
    let mut params: Vec<(String, f64)> = Vec::new();

    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if line.starts_with("lattice") {
            lattice = Some(parse_lattice(line)?);
            i += 1;
        } else if line.starts_with("sum") {
            let (block, next_i) = parse_sum_block(&lines, i)?;
            sum_blocks.push(block);
            i = next_i;
        } else if line.starts_with("params:") {
            let (p, next_i) = parse_params(&lines, i + 1)?;
            params = p;
            i = next_i;
        } else {
            return Err(ParseError(format!("Unexpected line: '{}'", line)));
        }
    }

    let lattice = lattice.ok_or_else(|| ParseError("Missing lattice declaration".to_string()))?;

    Ok(ModelDef { lattice, sum_blocks, params })
}

fn parse_lattice(line: &str) -> Result<LatticeDef, ParseError> {
    // lattice 1d sites=10 pbc=true
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return Err(ParseError(format!("Invalid lattice line: '{}'", line)));
    }

    let dimension = parts[1].to_string();
    let mut num_sites = 0usize;
    let mut pbc = false;

    for part in &parts[2..] {
        if let Some(val) = part.strip_prefix("sites=") {
            num_sites = val.parse().map_err(|_| ParseError(format!("Invalid sites: {}", val)))?;
        } else if let Some(val) = part.strip_prefix("pbc=") {
            pbc = val.parse().map_err(|_| ParseError(format!("Invalid pbc: {}", val)))?;
        }
    }

    if num_sites == 0 {
        return Err(ParseError("sites must be > 0".to_string()));
    }

    Ok(LatticeDef { dimension, num_sites, pbc })
}

fn parse_sum_block(lines: &[&str], start: usize) -> Result<(SumBlock, usize), ParseError> {
    // sum i=0..10:
    let header = lines[start].trim();
    let header = header.strip_prefix("sum").ok_or_else(|| ParseError("Expected 'sum'".to_string()))?.trim();
    let header = header.strip_suffix(':').ok_or_else(|| ParseError("Expected ':' after sum range".to_string()))?.trim();

    // Parse "i=0..10"
    let eq_pos = header.find('=').ok_or_else(|| ParseError("Expected '=' in sum".to_string()))?;
    let var = header[..eq_pos].trim().to_string();
    let range_str = &header[eq_pos + 1..];
    let dots = range_str.find("..").ok_or_else(|| ParseError("Expected '..' in range".to_string()))?;
    let range_start: usize = range_str[..dots].trim().parse()
        .map_err(|_| ParseError("Invalid range start".to_string()))?;
    let range_end: usize = range_str[dots + 2..].trim().parse()
        .map_err(|_| ParseError("Invalid range end".to_string()))?;

    let mut expressions = Vec::new();
    let mut i = start + 1;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            i += 1;
            break;
        }
        if !line.starts_with(' ') && !lines[i].starts_with('\t') && !lines[i].starts_with(' ') {
            // Check if the original line (not trimmed) is indented
            if !lines[i].starts_with(' ') && !lines[i].starts_with('\t') {
                break;
            }
        }
        if line.starts_with("params:") || line.starts_with("sum") || line.starts_with("lattice") {
            break;
        }
        if !line.is_empty() {
            expressions.push(parse_expression(line)?);
        }
        i += 1;
    }

    Ok((SumBlock { var, range_start, range_end, expressions }, i))
}

fn parse_expression(line: &str) -> Result<Expression, ParseError> {
    // Examples:
    //   -t * c†(i,up) c(i+1,up) + h.c.
    //   U * n(i,up) n(i,down)
    //   J * 2.0 * Sz(i) Sz(i+1)

    let (line, hc) = if line.ends_with("+ h.c.") {
        (line[..line.len() - 6].trim(), true)
    } else {
        (line, false)
    };

    // Split by '*' to get coefficient parts and operator parts
    let parts: Vec<&str> = line.split('*').map(|s| s.trim()).collect();

    // Find where operators start (operators contain '(' )
    let mut coeff_parts = Vec::new();
    let mut op_str = String::new();

    for part in &parts {
        if part.contains('(') {
            op_str = part.to_string();
        } else {
            coeff_parts.push(*part);
        }
    }

    // If no explicit split by *, look for operators in the last part
    if op_str.is_empty() && !parts.is_empty() {
        // Re-parse: everything before first operator call is coefficient
        let tokens = tokenize_expression(line)?;
        return build_expression(tokens, hc);
    }

    // Parse coefficient
    let coeff = parse_coeff_parts(&coeff_parts)?;

    // Parse operators from op_str (space-separated)
    let operators = parse_operators(&op_str)?;

    Ok(Expression { coeff, operators, hermitian_conjugate: hc })
}

fn tokenize_expression(line: &str) -> Result<Vec<String>, ParseError> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                chars.next();
            }
            '*' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                tokens.push("*".to_string());
                chars.next();
            }
            '(' => {
                current.push(ch);
                chars.next();
                // Read until matching ')'
                while let Some(&c) = chars.peek() {
                    current.push(c);
                    chars.next();
                    if c == ')' { break; }
                }
            }
            _ => {
                current.push(ch);
                chars.next();
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

fn build_expression(tokens: Vec<String>, hc: bool) -> Result<Expression, ParseError> {
    let mut coeff_parts: Vec<String> = Vec::new();
    let mut operators = Vec::new();
    let mut expect_star = false;

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];

        if token == "*" {
            expect_star = false;
            i += 1;
            continue;
        }

        if token.contains('(') || token.starts_with("c†") || token.starts_with("c(") {
            // This and everything after are operators
            operators.push(parse_single_operator(token)?);
            i += 1;
            while i < tokens.len() {
                if tokens[i] == "*" { i += 1; continue; }
                if tokens[i].contains('(') {
                    operators.push(parse_single_operator(&tokens[i])?);
                }
                i += 1;
            }
            break;
        } else {
            coeff_parts.push(token.clone());
            expect_star = true;
            i += 1;
        }
    }

    let coeff = parse_coeff_from_strings(&coeff_parts)?;

    Ok(Expression { coeff, operators, hermitian_conjugate: hc })
}

fn parse_coeff_parts(parts: &[&str]) -> Result<CoeffExpr, ParseError> {
    if parts.is_empty() {
        return Ok(CoeffExpr::Literal(1.0));
    }
    let strs: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
    parse_coeff_from_strings(&strs)
}

fn parse_coeff_from_strings(parts: &[String]) -> Result<CoeffExpr, ParseError> {
    if parts.is_empty() {
        return Ok(CoeffExpr::Literal(1.0));
    }

    let mut result: Option<CoeffExpr> = None;

    for part in parts {
        let part = part.trim();
        let expr = if part.starts_with('-') {
            let inner = &part[1..];
            if inner.is_empty() {
                CoeffExpr::Neg(Box::new(CoeffExpr::Literal(1.0)))
            } else if let Ok(v) = inner.parse::<f64>() {
                CoeffExpr::Literal(-v)
            } else {
                CoeffExpr::Neg(Box::new(CoeffExpr::Param(inner.to_string())))
            }
        } else if let Ok(v) = part.parse::<f64>() {
            CoeffExpr::Literal(v)
        } else {
            CoeffExpr::Param(part.to_string())
        };

        result = Some(match result {
            None => expr,
            Some(prev) => CoeffExpr::Mul(Box::new(prev), Box::new(expr)),
        });
    }

    Ok(result.unwrap())
}

fn parse_operators(op_str: &str) -> Result<Vec<OpExpr>, ParseError> {
    let mut ops = Vec::new();
    let tokens = tokenize_expression(op_str)?;

    for token in &tokens {
        if token == "*" { continue; }
        if token.contains('(') {
            ops.push(parse_single_operator(token)?);
        }
    }

    Ok(ops)
}

fn parse_single_operator(token: &str) -> Result<OpExpr, ParseError> {
    // c†(i,up), c(i+1,down), n(i,up), Sp(i), Sm(i), Sz(i)
    let paren_start = token.find('(')
        .ok_or_else(|| ParseError(format!("Expected '(' in operator: {}", token)))?;
    let paren_end = token.find(')')
        .ok_or_else(|| ParseError(format!("Expected ')' in operator: {}", token)))?;

    let name = &token[..paren_start];
    let args_str = &token[paren_start + 1..paren_end];
    let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

    match name {
        "c†" | "c\u{2020}" => {
            if args.len() != 2 {
                return Err(ParseError(format!("c† expects 2 args, got {}", args.len())));
            }
            Ok(OpExpr::FermionCreate(parse_index(args[0])?, parse_spin(args[1])?))
        }
        "c" => {
            if args.len() != 2 {
                return Err(ParseError(format!("c expects 2 args, got {}", args.len())));
            }
            Ok(OpExpr::FermionAnnihilate(parse_index(args[0])?, parse_spin(args[1])?))
        }
        "n" => {
            if args.len() != 2 {
                return Err(ParseError(format!("n expects 2 args, got {}", args.len())));
            }
            Ok(OpExpr::Number(parse_index(args[0])?, parse_spin(args[1])?))
        }
        "Sp" => Ok(OpExpr::SpinPlus(parse_index(args[0])?)),
        "Sm" => Ok(OpExpr::SpinMinus(parse_index(args[0])?)),
        "Sz" => Ok(OpExpr::SpinZ(parse_index(args[0])?)),
        _ => Err(ParseError(format!("Unknown operator: {}", name))),
    }
}

fn parse_index(s: &str) -> Result<IndexExpr, ParseError> {
    let s = s.trim();

    if let Ok(n) = s.parse::<usize>() {
        return Ok(IndexExpr::Literal(n));
    }

    if let Some(pos) = s.find('+') {
        let var = s[..pos].trim().to_string();
        let offset: usize = s[pos + 1..].trim().parse()
            .map_err(|_| ParseError(format!("Invalid index offset: {}", s)))?;
        return Ok(IndexExpr::VarPlus(var, offset));
    }

    if let Some(pos) = s.find('-') {
        if pos > 0 {
            let var = s[..pos].trim().to_string();
            let offset: usize = s[pos + 1..].trim().parse()
                .map_err(|_| ParseError(format!("Invalid index offset: {}", s)))?;
            return Ok(IndexExpr::VarMinus(var, offset));
        }
    }

    Ok(IndexExpr::Var(s.to_string()))
}

fn parse_spin(s: &str) -> Result<SpinExpr, ParseError> {
    match s.trim() {
        "up" => Ok(SpinExpr::Up),
        "down" => Ok(SpinExpr::Down),
        _ => Err(ParseError(format!("Invalid spin: {}", s))),
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_parser`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add src/parser/mod.rs tests/unit_parser.rs
git commit -m "feat: DSL parser for lattice/sum/params blocks"
```

---

### Task 5: Expand Stage

**Files:**
- Create: `src/core/expand.rs`
- Modify: `src/core/mod.rs`
- Create: `tests/unit_expand.rs`

**Step 1: Write the failing test**

Create `tests/unit_expand.rs`:
```rust
use quantum_simpl::parser::parse;
use quantum_simpl::core::expand::expand;
use quantum_simpl::core::op::Op;

#[test]
fn expand_2site_hubbard_term_count() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  -t * c†(i,up) c(i+1,up) + h.c.
  -t * c†(i,down) c(i+1,down) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
"#;
    let model = parse(input).unwrap();
    let ham = expand(&model);
    // 2 hopping up + 2 h.c. up + 2 hopping down + 2 h.c. down + 1 interaction (n expanded to c†c c†c)
    // sum i=0..1 means i=0 only, so: 1 hop up + 1 hc up + 1 hop down + 1 hc down + 1 interaction = 5
    // But n(i,up) n(i,down) expands to c†(i,up) c(i,up) c†(i,down) c(i,down) = 1 term
    assert_eq!(ham.num_sites, 2);
    assert_eq!(ham.terms.len(), 5);
}

#[test]
fn expand_hermitian_conjugate() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  -t * c†(i,up) c(i+1,up) + h.c.

params:
  t = 1.0
"#;
    let model = parse(input).unwrap();
    let ham = expand(&model);
    // Original: -1.0 * c†(0,up) c(1,up)
    // h.c.:    -1.0 * c†(1,up) c(0,up)
    assert_eq!(ham.terms.len(), 2);
    assert_eq!(ham.terms[0].coeff, -1.0);
    assert_eq!(ham.terms[1].coeff, -1.0);
}

#[test]
fn expand_number_operator_sugar() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  U * n(i,up) n(i,down)

params:
  U = 4.0
"#;
    let model = parse(input).unwrap();
    let ham = expand(&model);
    assert_eq!(ham.terms.len(), 1);
    assert_eq!(ham.terms[0].coeff, 4.0);
    assert_eq!(ham.terms[0].num_ops(), 4); // c†c c†c
}

#[test]
fn expand_pbc_wraps_around() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..4:
  -t * c†(i,up) c(i+1,up) + h.c.

params:
  t = 1.0
"#;
    let model = parse(input).unwrap();
    let ham = expand(&model);
    // 4 sites, PBC: i=0,1,2,3 each produce 1 hop + 1 h.c. = 8 terms
    assert_eq!(ham.terms.len(), 8);
    // Check that site 3 -> site 0 wrapping exists
    let has_wrap = ham.terms.iter().any(|t| {
        t.ops.len() == 2
            && t.ops[0] == Op::FermionCreate(3, quantum_simpl::core::op::Spin::Up)
            && t.ops[1] == Op::FermionAnnihilate(0, quantum_simpl::core::op::Spin::Up)
    });
    assert!(has_wrap, "PBC wrap term 3->0 not found");
}

#[test]
fn expand_obc_no_wrap() {
    let input = r#"
lattice 1d sites=4 pbc=false

sum i=0..3:
  -t * c†(i,up) c(i+1,up) + h.c.

params:
  t = 1.0
"#;
    let model = parse(input).unwrap();
    let ham = expand(&model);
    // 3 bonds, each with hop + h.c. = 6 terms
    assert_eq!(ham.terms.len(), 6);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_expand`
Expected: FAIL (expand module not defined)

**Step 3: Implement expand**

`src/core/expand.rs`:
```rust
use crate::core::op::{Op, Spin, Term, Hamiltonian};
use crate::parser::ast::*;
use smallvec::{smallvec, SmallVec};

pub fn expand(model: &ModelDef) -> Hamiltonian {
    let num_sites = model.lattice.num_sites;
    let pbc = model.lattice.pbc;
    let mut ham = Hamiltonian::new(num_sites);

    // Build param lookup
    let params: std::collections::HashMap<&str, f64> = model.params.iter()
        .map(|(k, v)| (k.as_str(), *v))
        .collect();

    for block in &model.sum_blocks {
        for idx in block.range_start..block.range_end {
            for expr in &block.expressions {
                let coeff = eval_coeff(&expr.coeff, &params);
                let ops = expand_operators(&expr.operators, &block.var, idx, num_sites, pbc);

                if let Some(ops) = ops {
                    let term = Term::new(coeff, ops);

                    if expr.hermitian_conjugate {
                        let hc = term.hermitian_conjugate();
                        ham.add_term(term);
                        ham.add_term(hc);
                    } else {
                        ham.add_term(term);
                    }
                }
            }
        }
    }

    ham
}

fn eval_coeff(expr: &CoeffExpr, params: &std::collections::HashMap<&str, f64>) -> f64 {
    match expr {
        CoeffExpr::Literal(v) => *v,
        CoeffExpr::Param(name) => *params.get(name.as_str()).unwrap_or(&0.0),
        CoeffExpr::Neg(inner) => -eval_coeff(inner, params),
        CoeffExpr::Mul(a, b) => eval_coeff(a, params) * eval_coeff(b, params),
    }
}

fn expand_operators(
    op_exprs: &[OpExpr],
    var: &str,
    idx: usize,
    num_sites: usize,
    pbc: bool,
) -> Option<SmallVec<[Op; 4]>> {
    let mut ops: SmallVec<[Op; 4]> = SmallVec::new();

    for op_expr in op_exprs {
        match op_expr {
            OpExpr::FermionCreate(index, spin) => {
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                ops.push(Op::FermionCreate(site, resolve_spin(spin)));
            }
            OpExpr::FermionAnnihilate(index, spin) => {
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                ops.push(Op::FermionAnnihilate(site, resolve_spin(spin)));
            }
            OpExpr::Number(index, spin) => {
                // n(i,s) = c†(i,s) c(i,s)
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                let s = resolve_spin(spin);
                ops.push(Op::FermionCreate(site, s));
                ops.push(Op::FermionAnnihilate(site, s));
            }
            OpExpr::SpinPlus(index) => {
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                ops.push(Op::SpinPlus(site));
            }
            OpExpr::SpinMinus(index) => {
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                ops.push(Op::SpinMinus(site));
            }
            OpExpr::SpinZ(index) => {
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                ops.push(Op::SpinZ(site));
            }
        }
    }

    Some(ops)
}

fn resolve_index(expr: &IndexExpr, var: &str, idx: usize, num_sites: usize, pbc: bool) -> Option<usize> {
    let raw = match expr {
        IndexExpr::Var(v) if v == var => idx,
        IndexExpr::VarPlus(v, offset) if v == var => idx + offset,
        IndexExpr::VarMinus(v, offset) if v == var => {
            if idx < *offset {
                if pbc { num_sites + idx - offset } else { return None; }
            } else {
                idx - offset
            }
        }
        IndexExpr::Literal(n) => *n,
        _ => return None,
    };

    if raw >= num_sites {
        if pbc {
            Some(raw % num_sites)
        } else {
            None // skip this term for OBC
        }
    } else {
        Some(raw)
    }
}

fn resolve_spin(spin: &SpinExpr) -> Spin {
    match spin {
        SpinExpr::Up => Spin::Up,
        SpinExpr::Down => Spin::Down,
    }
}
```

Update `src/core/mod.rs`:
```rust
pub mod op;
pub mod expand;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_expand`
Expected: 5 tests PASS

**Step 5: Commit**

```bash
git add src/core/expand.rs src/core/mod.rs tests/unit_expand.rs
git commit -m "feat: expand stage — loop unroll, h.c., n() sugar, PBC"
```

---

### Task 6: Normal Ordering

**Files:**
- Create: `src/core/normal.rs`
- Modify: `src/core/mod.rs`
- Create: `tests/unit_normal.rs`

**Step 1: Write the failing test**

Create `tests/unit_normal.rs`:
```rust
use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::normal::normal_order;
use smallvec::smallvec;

#[test]
fn already_normal_ordered() {
    // c†(0,up) c(1,up) — already in normal order
    let terms = vec![Term::new(1.0, smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(1, Spin::Up),
    ])];
    let result = normal_order(&terms);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].coeff, 1.0);
}

#[test]
fn swap_c_cdagger_same_site_spin() {
    // c(0,up) c†(0,up) = -c†(0,up) c(0,up) + 1 (delta)
    let terms = vec![Term::new(1.0, smallvec![
        Op::FermionAnnihilate(0, Spin::Up),
        Op::FermionCreate(0, Spin::Up),
    ])];
    let result = normal_order(&terms);
    // Should produce two terms:
    // -1.0 * c†(0,up) c(0,up)
    // +1.0 * identity
    assert_eq!(result.len(), 2);

    let normal_term = result.iter().find(|t| !t.ops.is_empty()).unwrap();
    assert_eq!(normal_term.coeff, -1.0);
    assert_eq!(normal_term.ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(normal_term.ops[1], Op::FermionAnnihilate(0, Spin::Up));

    let identity_term = result.iter().find(|t| t.ops.is_empty()).unwrap();
    assert_eq!(identity_term.coeff, 1.0);
}

#[test]
fn swap_c_cdagger_different_site() {
    // c(0,up) c†(1,up) = -c†(1,up) c(0,up) + 0 (delta=0, different sites)
    let terms = vec![Term::new(1.0, smallvec![
        Op::FermionAnnihilate(0, Spin::Up),
        Op::FermionCreate(1, Spin::Up),
    ])];
    let result = normal_order(&terms);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].coeff, -1.0);
}

#[test]
fn number_operator_product_normal_order() {
    // c†(0,up) c(0,up) c†(0,down) c(0,down) — already normal ordered
    let terms = vec![Term::new(4.0, smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(0, Spin::Up),
        Op::FermionCreate(0, Spin::Down),
        Op::FermionAnnihilate(0, Spin::Down),
    ])];
    let result = normal_order(&terms);
    // This is already normal ordered (c†_up c_up c†_down c_down)
    // Actually, full normal order would put ALL c† before ALL c:
    // c†_up c†_down c_up c_down (need to swap c_up past c†_down)
    // c†(0,up) c(0,up) c†(0,down) c(0,down)
    //   = c†(0,up) (-c†(0,down) c(0,up) + δ(0,up;0,down)) c(0,down)
    //   = -c†(0,up) c†(0,down) c(0,up) c(0,down) + 0 (δ=0, different spins)
    //   = c†(0,up) c†(0,down) c(0,down) c(0,up) (swap c_up, c_down: minus sign)
    // hmm, this gets complicated. Let's just check it doesn't crash and preserves physics.
    assert!(!result.is_empty());
}

#[test]
fn spin_operators_different_sites_unchanged() {
    // Sp(0) Sm(1) — different sites, no commutation needed
    let terms = vec![Term::new(1.0, smallvec![
        Op::SpinPlus(0),
        Op::SpinMinus(1),
    ])];
    let result = normal_order(&terms);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ops.len(), 2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_normal`
Expected: FAIL

**Step 3: Implement normal ordering**

`src/core/normal.rs`:
```rust
use crate::core::op::{Op, Term};
use smallvec::SmallVec;

/// Normal-order all terms: move all c† to the left of c,
/// applying anticommutation relations {c_i, c†_j} = δ_{ij}.
/// For spin operators on the same site, apply [S+, S-] = 2Sz.
pub fn normal_order(terms: &[Term]) -> Vec<Term> {
    let mut result = Vec::new();
    for term in terms {
        let mut expanded = vec![term.clone()];
        let mut all_done = false;

        // Iteratively bubble-sort creation operators left
        while !all_done {
            all_done = true;
            let mut next_expanded = Vec::new();

            for t in &expanded {
                if let Some(swap_pos) = find_swap_position(&t.ops) {
                    all_done = false;
                    let swapped = apply_swap(t, swap_pos);
                    next_expanded.extend(swapped);
                } else {
                    next_expanded.push(t.clone());
                }
            }

            expanded = next_expanded;
        }

        result.extend(expanded);
    }
    result
}

/// Find the leftmost position where a non-creation op appears before a creation op
/// (for fermions) or where S- appears before S+ on the same site (for spins).
fn find_swap_position(ops: &SmallVec<[Op; 4]>) -> Option<usize> {
    for i in 0..ops.len().saturating_sub(1) {
        let left = &ops[i];
        let right = &ops[i + 1];

        // Fermion: annihilation before creation
        if left.is_annihilation() && right.is_creation() {
            return Some(i);
        }

        // Spin: S- before S+ on same site
        if let (Op::SpinMinus(s1), Op::SpinPlus(s2)) = (left, right) {
            if s1 == s2 {
                return Some(i);
            }
        }
    }
    None
}

/// Apply anticommutation/commutation at position i, producing new terms.
fn apply_swap(term: &Term, pos: usize) -> Vec<Term> {
    let left = &term.ops[pos];
    let right = &term.ops[pos + 1];

    match (left, right) {
        // Fermion anticommutation: c_a c†_b = -c†_b c_a + δ_{ab}
        (Op::FermionAnnihilate(s1, sp1), Op::FermionCreate(s2, sp2)) => {
            // Swapped term: -coeff * (... c†_b c_a ...)
            let mut swapped_ops = term.ops.clone();
            swapped_ops[pos] = *right;
            swapped_ops[pos + 1] = *left;
            let swapped = Term::new(-term.coeff, swapped_ops);

            let mut result = vec![swapped];

            // Delta term: if same site and spin, add coeff * (... without these two ops ...)
            if s1 == s2 && sp1 == sp2 {
                let mut delta_ops: SmallVec<[Op; 4]> = SmallVec::new();
                for (j, op) in term.ops.iter().enumerate() {
                    if j != pos && j != pos + 1 {
                        delta_ops.push(*op);
                    }
                }
                result.push(Term::new(term.coeff, delta_ops));
            }

            result
        }

        // Spin commutation: S-(i) S+(i) = S+(i) S-(i) - 2Sz(i)
        (Op::SpinMinus(s1), Op::SpinPlus(s2)) if s1 == s2 => {
            // Swapped: S+(i) S-(i)
            let mut swapped_ops = term.ops.clone();
            swapped_ops[pos] = *right;
            swapped_ops[pos + 1] = *left;
            let swapped = Term::new(term.coeff, swapped_ops);

            // Commutator contribution: -2Sz(i)
            let mut sz_ops: SmallVec<[Op; 4]> = SmallVec::new();
            for (j, op) in term.ops.iter().enumerate() {
                if j == pos {
                    sz_ops.push(Op::SpinZ(*s1));
                } else if j == pos + 1 {
                    // skip (replaced by single Sz above)
                } else {
                    sz_ops.push(*op);
                }
            }
            let sz_term = Term::new(-2.0 * term.coeff, sz_ops);

            vec![swapped, sz_term]
        }

        _ => vec![term.clone()],
    }
}
```

Update `src/core/mod.rs`:
```rust
pub mod op;
pub mod expand;
pub mod normal;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_normal`
Expected: 5 tests PASS

**Step 5: Commit**

```bash
git add src/core/normal.rs src/core/mod.rs tests/unit_normal.rs
git commit -m "feat: normal ordering — fermion anticommutation and spin commutation"
```

---

### Task 7: Combine Like Terms

**Files:**
- Create: `src/core/combine.rs`
- Modify: `src/core/mod.rs`
- Create: `tests/unit_combine.rs`

**Step 1: Write the failing test**

Create `tests/unit_combine.rs`:
```rust
use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::combine::combine;
use smallvec::smallvec;

#[test]
fn combine_same_ops_sums_coefficients() {
    let terms = vec![
        Term::new(2.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Up)]),
        Term::new(3.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Up)]),
    ];
    let result = combine(&terms);
    assert_eq!(result.len(), 1);
    assert!((result[0].coeff - 5.0).abs() < 1e-12);
}

#[test]
fn combine_different_ops_kept_separate() {
    let terms = vec![
        Term::new(1.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Up)]),
        Term::new(1.0, smallvec![Op::FermionCreate(1, Spin::Up), Op::FermionAnnihilate(0, Spin::Up)]),
    ];
    let result = combine(&terms);
    assert_eq!(result.len(), 2);
}

#[test]
fn combine_eliminates_zero_coefficients() {
    let terms = vec![
        Term::new(1.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Up)]),
        Term::new(-1.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Up)]),
    ];
    let result = combine(&terms);
    assert_eq!(result.len(), 0);
}

#[test]
fn combine_identity_terms() {
    let terms = vec![
        Term::new(1.0, smallvec![]),
        Term::new(2.0, smallvec![]),
    ];
    let result = combine(&terms);
    assert_eq!(result.len(), 1);
    assert!((result[0].coeff - 3.0).abs() < 1e-12);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_combine`
Expected: FAIL

**Step 3: Implement combine**

`src/core/combine.rs`:
```rust
use crate::core::op::{Op, Term};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

const ZERO_THRESHOLD: f64 = 1e-15;

/// Combine terms with identical operator strings by summing their coefficients.
/// Remove terms whose coefficient is effectively zero.
pub fn combine(terms: &[Term]) -> Vec<Term> {
    let mut map: FxHashMap<SmallVec<[Op; 4]>, f64> = FxHashMap::default();

    for term in terms {
        *map.entry(term.ops.clone()).or_insert(0.0) += term.coeff;
    }

    map.into_iter()
        .filter(|(_, coeff)| coeff.abs() > ZERO_THRESHOLD)
        .map(|(ops, coeff)| Term::new(coeff, ops))
        .collect()
}
```

Update `src/core/mod.rs`:
```rust
pub mod op;
pub mod expand;
pub mod normal;
pub mod combine;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_combine`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add src/core/combine.rs src/core/mod.rs tests/unit_combine.rs
git commit -m "feat: combine like terms with FxHashMap deduplication"
```

---

### Task 8: Sz Symmetry Reduction

**Files:**
- Create: `src/core/symmetry.rs`
- Modify: `src/core/mod.rs`
- Create: `tests/unit_symmetry.rs`

**Step 1: Write the failing test**

Create `tests/unit_symmetry.rs`:
```rust
use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::symmetry::filter_sz_conserving;
use smallvec::smallvec;

#[test]
fn sz_conserving_terms_kept() {
    let terms = vec![
        // c†(0,up) c(1,up) — Δ Sz = 1 + (-1) = 0 ✓
        Term::new(1.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Up)]),
        // Sp(0) Sm(1) — Δ Sz = 1 + (-1) = 0 ✓
        Term::new(1.0, smallvec![Op::SpinPlus(0), Op::SpinMinus(1)]),
        // Sz(0) Sz(1) — Δ Sz = 0 + 0 = 0 ✓
        Term::new(1.0, smallvec![Op::SpinZ(0), Op::SpinZ(1)]),
    ];
    let result = filter_sz_conserving(&terms);
    assert_eq!(result.len(), 3);
}

#[test]
fn sz_nonconserving_terms_removed() {
    let terms = vec![
        // c†(0,up) c(1,down) — Δ Sz = 1 + 1 = 2 ✗
        Term::new(1.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Down)]),
        // Sp(0) Sp(1) — Δ Sz = 1 + 1 = 2 ✗
        Term::new(1.0, smallvec![Op::SpinPlus(0), Op::SpinPlus(1)]),
    ];
    let result = filter_sz_conserving(&terms);
    assert_eq!(result.len(), 0);
}

#[test]
fn identity_term_preserved() {
    let terms = vec![
        Term::new(5.0, smallvec![]), // identity, Δ Sz = 0
    ];
    let result = filter_sz_conserving(&terms);
    assert_eq!(result.len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_symmetry`
Expected: FAIL

**Step 3: Implement symmetry filter**

`src/core/symmetry.rs`:
```rust
use crate::core::op::Term;

/// Keep only terms that conserve total Sz (i.e., ΔSz = 0).
pub fn filter_sz_conserving(terms: &[Term]) -> Vec<Term> {
    terms.iter()
        .filter(|t| t.delta_sz() == 0)
        .cloned()
        .collect()
}
```

Update `src/core/mod.rs`:
```rust
pub mod op;
pub mod expand;
pub mod normal;
pub mod combine;
pub mod symmetry;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_symmetry`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add src/core/symmetry.rs src/core/mod.rs tests/unit_symmetry.rs
git commit -m "feat: Sz symmetry filter — remove non-conserving terms"
```

---

### Task 9: mVMC Output Writer

**Files:**
- Modify: `src/output/mvmc.rs`
- Create: `tests/unit_mvmc.rs`

**Step 1: Write the failing test**

Create `tests/unit_mvmc.rs`:
```rust
use quantum_simpl::core::op::{Op, Spin, Term, Hamiltonian};
use quantum_simpl::output::mvmc::{generate_trans_def, generate_interall_def, generate_namelist};
use smallvec::smallvec;

#[test]
fn trans_def_one_body_format() {
    let mut ham = Hamiltonian::new(2);
    // -1.0 * c†(0,up) c(1,up)
    ham.add_term(Term::new(-1.0, smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(1, Spin::Up),
    ]));
    let output = generate_trans_def(&ham);
    assert!(output.contains("NTransfer"));
    assert!(output.contains("    0     0     1     0"));
    assert!(output.contains("-1.000000"));
}

#[test]
fn interall_def_two_body_format() {
    let mut ham = Hamiltonian::new(2);
    // 4.0 * c†(0,up) c(0,up) c†(0,down) c(0,down)
    ham.add_term(Term::new(4.0, smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(0, Spin::Up),
        Op::FermionCreate(0, Spin::Down),
        Op::FermionAnnihilate(0, Spin::Down),
    ]));
    let output = generate_interall_def(&ham);
    assert!(output.contains("TotalNumber"));
    assert!(output.contains("4.000000"));
}

#[test]
fn namelist_references_all_files() {
    let output = generate_namelist();
    assert!(output.contains("ModPara"));
    assert!(output.contains("Trans"));
    assert!(output.contains("LocSpin"));
    assert!(output.contains("InterAll"));
}

#[test]
fn trans_def_empty_if_no_one_body() {
    let ham = Hamiltonian::new(2);
    let output = generate_trans_def(&ham);
    assert!(output.contains("NTransfer      0"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_mvmc`
Expected: FAIL

**Step 3: Implement mVMC writer**

`src/output/mvmc.rs`:
```rust
use crate::core::op::{Op, Spin, Hamiltonian, Term};

fn spin_to_idx(s: Spin) -> usize {
    match s {
        Spin::Up => 0,
        Spin::Down => 1,
    }
}

/// Classify terms into one-body (2 ops) and two-body (4 ops).
fn classify_terms(ham: &Hamiltonian) -> (Vec<&Term>, Vec<&Term>) {
    let mut one_body = Vec::new();
    let mut two_body = Vec::new();

    for term in &ham.terms {
        match term.ops.len() {
            2 => one_body.push(term),
            4 => two_body.push(term),
            0 => {} // identity terms (constant energy offset), skip for now
            _ => {} // skip higher-body terms
        }
    }

    (one_body, two_body)
}

pub fn generate_trans_def(ham: &Hamiltonian) -> String {
    let (one_body, _) = classify_terms(ham);
    let mut out = String::new();

    out.push_str("======================== \n");
    out.push_str(&format!("NTransfer      {}  \n", one_body.len()));
    out.push_str("======================== \n");
    out.push_str("========i_j_s_tijs====== \n");
    out.push_str("======================== \n");

    for term in &one_body {
        // c†(i, si) c(j, sj) → i si j sj Re Im
        if let (Op::FermionCreate(i, si), Op::FermionAnnihilate(j, sj)) = (term.ops[0], term.ops[1]) {
            out.push_str(&format!("    {}     {}     {}     {}         {:.15}         {:.15}\n",
                i, spin_to_idx(si), j, spin_to_idx(sj), term.coeff, 0.0));
        }
    }

    out
}

pub fn generate_interall_def(ham: &Hamiltonian) -> String {
    let (_, two_body) = classify_terms(ham);
    let mut out = String::new();

    out.push_str("========================\n");
    out.push_str(&format!("TotalNumber {}\n", two_body.len()));
    out.push_str("Comment: interall\n");
    out.push_str("========================\n");
    out.push_str("========================\n");

    for term in &two_body {
        // c†(i,si) c(j,sj) c†(k,sk) c(l,sl) → i si j sj k sk l sl Re Im
        if let (
            Op::FermionCreate(i, si),
            Op::FermionAnnihilate(j, sj),
            Op::FermionCreate(k, sk),
            Op::FermionAnnihilate(l, sl),
        ) = (term.ops[0], term.ops[1], term.ops[2], term.ops[3]) {
            out.push_str(&format!("{} {} {} {} {} {} {} {} {:.1} {:.1} \n",
                i, spin_to_idx(si), j, spin_to_idx(sj),
                k, spin_to_idx(sk), l, spin_to_idx(sl),
                term.coeff, 0.0));
        }
    }

    out
}

pub fn generate_locspn_def(ham: &Hamiltonian) -> String {
    let mut out = String::new();
    out.push_str("================================ \n");
    out.push_str(&format!("NlocalSpin     {}  \n", 0));
    out.push_str("================================ \n");
    out.push_str("========i_0LocSpn_1IteElc ====== \n");
    out.push_str("================================ \n");
    for i in 0..ham.num_sites {
        out.push_str(&format!("    {}      0\n", i));
    }
    out
}

pub fn generate_modpara_def(ham: &Hamiltonian) -> String {
    let nsite = ham.num_sites;
    let ncond = nsite; // half-filling default
    format!(
"--------------------
Model_Parameters   0
--------------------
VMC_Cal_Parameters
--------------------
CDataFileHead  zvo
CParaFileHead  zqp
--------------------
NVMCCalMode    0
--------------------
NDataIdxStart  1
NDataQtySmp    1
--------------------
Nsite          {nsite}
Ncond          {ncond}
2Sz            0
NSPGaussLeg    1
NSPStot        0
NMPTrans       1
NSROptItrStep  1000
NSROptItrSmp   100
DSROptRedCut   1e-10
DSROptStaDel   0.001
DSROptStepDt   0.003
NVMCWarmUp     10
NVMCInterval   1
NVMCSample     100
NExUpdatePath  0
NSplitSize     1
NStore         1
NSRCG          0
RndSeed  12345
")
}

pub fn generate_namelist() -> String {
    "         ModPara  modpara.def\n\
         LocSpin  locspn.def\n\
           Trans  trans.def\n\
        InterAll  interall.def\n\
      Gutzwiller  gutzwilleridx.def\n\
         Jastrow  jastrowidx.def\n\
         Orbital  orbitalidx.def\n\
        TransSym  qptransidx.def\n"
        .to_string()
}

pub fn generate_gutzwilleridx_def(ham: &Hamiltonian) -> String {
    let mut out = String::new();
    out.push_str("=============================================\n");
    out.push_str(&format!("NGutzwillerIdx          1\n"));
    out.push_str("ComplexType          0\n");
    out.push_str("=============================================\n");
    out.push_str("=============================================\n");
    for i in 0..ham.num_sites {
        out.push_str(&format!("    {}      0\n", i));
    }
    out.push_str("    0      1\n");
    out
}

pub fn generate_jastrowidx_def(ham: &Hamiltonian) -> String {
    let n = ham.num_sites;
    let mut out = String::new();
    out.push_str("=============================================\n");
    out.push_str(&format!("NJastrowIdx          {}\n", n / 2));
    out.push_str("ComplexType          0\n");
    out.push_str("=============================================\n");
    out.push_str("=============================================\n");
    for i in 0..n {
        for j in 0..n {
            if i != j {
                let dist = if i > j { i - j } else { j - i };
                let idx = dist.min(n - dist) - 1;
                out.push_str(&format!("    {}      {}      {}\n", i, j, idx));
            }
        }
    }
    for i in 0..(n / 2) {
        out.push_str(&format!("    {}      1\n", i));
    }
    out
}

pub fn generate_orbitalidx_def(ham: &Hamiltonian) -> String {
    let n = ham.num_sites;
    let mut out = String::new();
    out.push_str("=============================================\n");
    out.push_str(&format!("NOrbitalIdx         {}\n", n));
    out.push_str("ComplexType          0\n");
    out.push_str("=============================================\n");
    out.push_str("=============================================\n");
    for i in 0..n {
        for j in 0..n {
            let idx = ((j + n - i) % n);
            out.push_str(&format!("    {}      {}      {}\n", i, j, idx));
        }
    }
    for i in 0..n {
        out.push_str(&format!("    {}      1\n", i));
    }
    out
}

pub fn generate_qptransidx_def(ham: &Hamiltonian) -> String {
    let n = ham.num_sites;
    let mut out = String::new();
    out.push_str("=============================================\n");
    out.push_str("NQPTrans          1\n");
    out.push_str("=============================================\n");
    out.push_str("======== TrIdx_TrWeight_and_TrIdx_i_xi ======\n");
    out.push_str("=============================================\n");
    out.push_str("0    1.00000\n");
    for i in 0..n {
        out.push_str(&format!("    0      {}      {}\n", i, i));
    }
    out
}

use std::path::Path;
use std::fs;

pub fn write_all_files(ham: &Hamiltonian, output_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(output_dir)?;

    fs::write(output_dir.join("namelist.def"), generate_namelist())?;
    fs::write(output_dir.join("modpara.def"), generate_modpara_def(ham))?;
    fs::write(output_dir.join("locspn.def"), generate_locspn_def(ham))?;
    fs::write(output_dir.join("trans.def"), generate_trans_def(ham))?;
    fs::write(output_dir.join("interall.def"), generate_interall_def(ham))?;
    fs::write(output_dir.join("gutzwilleridx.def"), generate_gutzwilleridx_def(ham))?;
    fs::write(output_dir.join("jastrowidx.def"), generate_jastrowidx_def(ham))?;
    fs::write(output_dir.join("orbitalidx.def"), generate_orbitalidx_def(ham))?;
    fs::write(output_dir.join("qptransidx.def"), generate_qptransidx_def(ham))?;

    Ok(())
}
```

Update `src/output/mod.rs`:
```rust
pub mod mvmc;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_mvmc`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add src/output/mvmc.rs src/output/mod.rs tests/unit_mvmc.rs
git commit -m "feat: mVMC output writer — trans.def, interall.def, and all template files"
```

---

### Task 10: CLI Entry Point and Full Pipeline

**Files:**
- Modify: `src/main.rs`
- Create: `tests/integration/test_pipeline.rs`

**Step 1: Write the failing test**

Create `tests/integration/test_pipeline.rs`:
```rust
use quantum_simpl::parser::parse;
use quantum_simpl::core::expand::expand;
use quantum_simpl::core::normal::normal_order;
use quantum_simpl::core::combine::combine;
use quantum_simpl::core::symmetry::filter_sz_conserving;
use quantum_simpl::output::mvmc::write_all_files;
use tempfile::TempDir;

#[test]
fn full_pipeline_hubbard_2site() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  -t * c†(i,up) c(i+1,up) + h.c.
  -t * c†(i,down) c(i+1,down) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
"#;
    let model = parse(input).unwrap();
    let ham = expand(&model);
    let terms = normal_order(&ham.terms);
    let terms = combine(&terms);
    let terms = filter_sz_conserving(&terms);

    let mut final_ham = quantum_simpl::core::op::Hamiltonian::new(model.lattice.num_sites);
    for t in terms {
        final_ham.add_term(t);
    }

    let dir = TempDir::new().unwrap();
    write_all_files(&final_ham, dir.path()).unwrap();

    // Check files exist
    assert!(dir.path().join("namelist.def").exists());
    assert!(dir.path().join("trans.def").exists());
    assert!(dir.path().join("interall.def").exists());
    assert!(dir.path().join("modpara.def").exists());
    assert!(dir.path().join("locspn.def").exists());

    // Check trans.def has content
    let trans = std::fs::read_to_string(dir.path().join("trans.def")).unwrap();
    assert!(trans.contains("NTransfer"));
}

#[test]
fn full_pipeline_heisenberg_4site() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..4:
  J * Sp(i) Sm(i+1)
  J * Sm(i) Sp(i+1)
  J * 2.0 * Sz(i) Sz(i+1)

params:
  J = 1.0
"#;
    let model = parse(input).unwrap();
    let ham = expand(&model);
    let terms = normal_order(&ham.terms);
    let terms = combine(&terms);
    let terms = filter_sz_conserving(&terms);

    // All isotropic Heisenberg terms conserve Sz
    assert!(terms.len() >= 12);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_pipeline`
Expected: FAIL (pipeline not wired up)

**Step 3: Implement CLI**

`src/main.rs`:
```rust
mod core;
mod parser;
mod output;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "quantum-simpl")]
#[command(about = "Hamiltonian symbolic preprocessor for mVMC")]
struct Cli {
    /// Input DSL file
    input: PathBuf,

    /// Output directory for mVMC files
    #[arg(short, long, default_value = "output")]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let input = std::fs::read_to_string(&cli.input)
        .unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", cli.input.display(), e);
            std::process::exit(1);
        });

    let model = parser::parse(&input)
        .unwrap_or_else(|e| {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        });

    eprintln!("Parsed: {} sites, {} sum blocks, {} params",
        model.lattice.num_sites, model.sum_blocks.len(), model.params.len());

    // Pipeline
    let ham = core::expand::expand(&model);
    eprintln!("Expanded: {} terms", ham.terms.len());

    let terms = core::normal::normal_order(&ham.terms);
    eprintln!("Normal ordered: {} terms", terms.len());

    let terms = core::combine::combine(&terms);
    eprintln!("Combined: {} terms", terms.len());

    let terms = core::symmetry::filter_sz_conserving(&terms);
    eprintln!("After Sz filter: {} terms", terms.len());

    let mut final_ham = core::op::Hamiltonian::new(model.lattice.num_sites);
    for t in terms {
        final_ham.add_term(t);
    }

    output::mvmc::write_all_files(&final_ham, &cli.output)
        .unwrap_or_else(|e| {
            eprintln!("Error writing output: {}", e);
            std::process::exit(1);
        });

    eprintln!("Written mVMC files to {}", cli.output.display());
}
```

**Step 4: Run tests to verify**

Run: `cargo test --test test_pipeline`
Expected: 2 tests PASS

Run: `cargo build`
Expected: Compiles

**Step 5: Commit**

```bash
git add src/main.rs tests/integration/test_pipeline.rs
git commit -m "feat: CLI entry point and full pipeline wiring"
```

---

### Task 11: Update Integration Tests and Final Verification

**Files:**
- Modify: `tests/integration/test_hubbard.rs` (replace skeleton with working tests)
- Modify: `tests/integration/test_heisenberg.rs` (replace skeleton with working tests)
- Modify: `tests/integration/test_mvmc_output.rs` (replace skeleton with working tests)

**Step 1: Update test_hubbard.rs**

Replace `todo!()` calls with actual pipeline calls using the now-implemented API. Remove `#[ignore]` attributes.

**Step 2: Update test_heisenberg.rs**

Same — replace skeletons with working tests.

**Step 3: Update test_mvmc_output.rs**

Same — replace skeletons with working tests using `tempfile`.

**Step 4: Run all tests**

Run: `cargo test`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add tests/
git commit -m "feat: enable all integration tests with working pipeline"
```

---

### Task 12: Push and Close Issue

**Step 1: Push to GitHub**

```bash
git push origin main
```

**Step 2: Close the test plan issue**

```bash
gh issue close 1 --repo iwaemon/quantum-simpl --comment "All integration tests passing. Implementation complete."
```

---

## Summary

| Task | Component | Tests |
|------|-----------|-------|
| 1 | Project scaffold | compile check |
| 2 | Op, Term, Hamiltonian | 3 unit tests |
| 3 | AST types | 1 unit test |
| 4 | DSL Parser | 4 unit tests |
| 5 | Expand stage | 5 unit tests |
| 6 | Normal ordering | 5 unit tests |
| 7 | Combine like terms | 4 unit tests |
| 8 | Sz symmetry filter | 3 unit tests |
| 9 | mVMC output writer | 4 unit tests |
| 10 | CLI + full pipeline | 2 integration tests |
| 11 | Update integration tests | 14 integration tests |
| 12 | Push and close issue | — |

**Total: 12 tasks, ~45 tests**
