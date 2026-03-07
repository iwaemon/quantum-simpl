# Correlation Function Auto-Transformation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `--correlation` mode that reads a DSL file with spin/density correlation functions, automatically transforms them to fermionic form, and outputs mVMC measurement files (`cisajs.def`, `cisajscktaltdc.def`) plus a human-readable summary.

**Architecture:** Extend the existing parser with `S(i).S(j)` syntax sugar, add a `spin_to_fermion` transform that converts spin operators to `c†c` products (with Term splitting for `Sz`), reuse normal_order → combine, then apply green_reorder to get mVMC's `c†cc†c` format. Output via new formatters in `mvmc.rs`.

**Tech Stack:** Rust, existing quantum-simpl pipeline (parser, expand, normal, combine, green), SmallVec, clap

---

### Task 1: Add `S(i).S(j)` syntax sugar to parser

**Files:**
- Modify: `src/parser/ast.rs` (add `SpinDot` variant to `OpExpr`)
- Modify: `src/parser/mod.rs:269-301` (add `S` operator parsing)
- Create: `tests/unit_parser_spindot.rs`

**Step 1: Write the failing test**

Create `tests/unit_parser_spindot.rs`:

```rust
use quantum_simpl::parser::parse;
use quantum_simpl::parser::ast::*;

#[test]
fn parse_spindot_expression() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..3:
  S(i) . S(i+1)
"#;
    let model = parse(input).unwrap();
    assert_eq!(model.sum_blocks.len(), 1);

    let block = &model.sum_blocks[0];
    // S(i).S(j) should expand into 3 expressions:
    // 0.5 * Sp(i) Sm(i+1)
    // 0.5 * Sm(i) Sp(i+1)
    // 1.0 * Sz(i) Sz(i+1)
    assert_eq!(block.expressions.len(), 3);

    // First: 0.5 * Sp(i) Sm(i+1)
    let e0 = &block.expressions[0];
    assert!(matches!(e0.coeff, CoeffExpr::Literal(c) if (c - 0.5).abs() < 1e-12));
    assert_eq!(e0.operators.len(), 2);
    assert!(matches!(&e0.operators[0], OpExpr::SpinPlus(IndexExpr::Var(v)) if v == "i"));
    assert!(matches!(&e0.operators[1], OpExpr::SpinMinus(IndexExpr::VarPlus(v, 1)) if v == "i"));

    // Second: 0.5 * Sm(i) Sp(i+1)
    let e1 = &block.expressions[1];
    assert!(matches!(e1.coeff, CoeffExpr::Literal(c) if (c - 0.5).abs() < 1e-12));
    assert!(matches!(&e1.operators[0], OpExpr::SpinMinus(IndexExpr::Var(v)) if v == "i"));
    assert!(matches!(&e1.operators[1], OpExpr::SpinPlus(IndexExpr::VarPlus(v, 1)) if v == "i"));

    // Third: 1.0 * Sz(i) Sz(i+1)
    let e2 = &block.expressions[2];
    assert!(matches!(e2.coeff, CoeffExpr::Literal(c) if (c - 1.0).abs() < 1e-12));
    assert!(matches!(&e2.operators[0], OpExpr::SpinZ(IndexExpr::Var(v)) if v == "i"));
    assert!(matches!(&e2.operators[1], OpExpr::SpinZ(IndexExpr::VarPlus(v, 1)) if v == "i"));
}

#[test]
fn parse_spindot_with_coefficient() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..3:
  J * S(i) . S(i+1)

params:
  J = 1.5
"#;
    let model = parse(input).unwrap();
    let block = &model.sum_blocks[0];
    assert_eq!(block.expressions.len(), 3);

    // Coefficients should be J*0.5, J*0.5, J*1.0
    // The coeff should include the J parameter
    let e0 = &block.expressions[0];
    // coeff is Mul(Param("J"), Literal(0.5))
    assert!(matches!(&e0.coeff, CoeffExpr::Mul(_, _)));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_parser_spindot`
Expected: FAIL — cannot find `unit_parser_spindot` or parse error on `S(i) . S(i+1)`.

**Step 3: Implement S(i).S(j) parsing**

In `src/parser/mod.rs`, modify `parse_expression` (around line 146) to detect `S(i) . S(j)` pattern and return multiple expressions. The tricky part: `parse_expression` currently returns one `Expression`, but `S(i).S(j)` produces three.

**Approach:** Change `parse_expression` to return `Vec<Expression>` and update `parse_sum_block` accordingly.

Modify `src/parser/mod.rs`:

1. Change `parse_expression` signature and add `S(i).S(j)` detection:

```rust
fn parse_expression(line: &str) -> Result<Vec<Expression>, ParseError> {
    // Check for S(i) . S(j) pattern
    if let Some(exprs) = try_parse_spindot(line)? {
        return Ok(exprs);
    }

    // Original logic, wrapped in Vec
    let (line, hc) = if line.ends_with("+ h.c.") {
        (line[..line.len() - 6].trim(), true)
    } else {
        (line, false)
    };

    let tokens = tokenize(line)?;
    let expr = build_expression(&tokens, hc)?;
    Ok(vec![expr])
}
```

2. Add `try_parse_spindot`:

```rust
fn try_parse_spindot(line: &str) -> Result<Option<Vec<Expression>>, ParseError> {
    // Look for pattern: [coeff *] S(idx1) . S(idx2)
    // The '.' with spaces is the key marker
    let dot_pos = line.find(" . ");
    if dot_pos.is_none() {
        return Ok(None);
    }
    let dot_pos = dot_pos.unwrap();

    let left_part = line[..dot_pos].trim();
    let right_part = line[dot_pos + 3..].trim();

    // Right side must be S(...)
    if !right_part.starts_with("S(") {
        return Ok(None);
    }

    // Parse left side: optional coeff + S(idx1)
    // Find S( in left part
    let s_pos = left_part.rfind("S(");
    if s_pos.is_none() {
        return Ok(None);
    }
    let s_pos = s_pos.unwrap();

    let coeff_str = left_part[..s_pos].trim().trim_end_matches('*').trim();
    let s_left_token = &left_part[s_pos..];

    // Parse the two S operator arguments to get IndexExpr
    let idx1 = parse_s_index(s_left_token)?;
    let idx2 = parse_s_index(right_part)?;

    // Build the coefficient
    let base_coeff = if coeff_str.is_empty() {
        CoeffExpr::Literal(1.0)
    } else {
        let coeff_parts: Vec<String> = coeff_str.split_whitespace()
            .filter(|s| *s != "*")
            .map(|s| s.to_string())
            .collect();
        parse_coeff_from_strings(&coeff_parts)?
    };

    // Generate 3 expressions:
    // 0.5 * Sp(i) Sm(j)
    let coeff_half = CoeffExpr::Mul(
        Box::new(base_coeff.clone()),
        Box::new(CoeffExpr::Literal(0.5)),
    );
    let e_sp_sm = Expression {
        coeff: coeff_half.clone(),
        operators: vec![OpExpr::SpinPlus(idx1.clone()), OpExpr::SpinMinus(idx2.clone())],
        hermitian_conjugate: false,
    };

    // 0.5 * Sm(i) Sp(j)
    let e_sm_sp = Expression {
        coeff: coeff_half,
        operators: vec![OpExpr::SpinMinus(idx1.clone()), OpExpr::SpinPlus(idx2.clone())],
        hermitian_conjugate: false,
    };

    // 1.0 * Sz(i) Sz(j)
    let coeff_one = base_coeff;
    let e_sz_sz = Expression {
        coeff: coeff_one,
        operators: vec![OpExpr::SpinZ(idx1), OpExpr::SpinZ(idx2)],
        hermitian_conjugate: false,
    };

    Ok(Some(vec![e_sp_sm, e_sm_sp, e_sz_sz]))
}

fn parse_s_index(token: &str) -> Result<IndexExpr, ParseError> {
    let start = token.find('(').ok_or_else(|| ParseError("Expected '(' in S operator".to_string()))?;
    let end = token.find(')').ok_or_else(|| ParseError("Expected ')' in S operator".to_string()))?;
    let arg = &token[start + 1..end];
    parse_index(arg)
}
```

3. Update `parse_sum_block` (line 113) to use `Vec<Expression>`:

Change:
```rust
expressions.push(parse_expression(trimmed)?);
```
To:
```rust
expressions.extend(parse_expression(trimmed)?);
```

4. Add `Clone` derive to `IndexExpr` and `CoeffExpr` in `src/parser/ast.rs` (already present — both derive Clone).

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_parser_spindot`
Expected: All 2 tests PASS.

**Step 5: Run all existing tests to check for regressions**

Run: `cargo test`
Expected: All existing tests still PASS.

**Step 6: Commit**

```bash
git add src/parser/mod.rs tests/unit_parser_spindot.rs
git commit -m "feat: add S(i).S(j) syntax sugar to parser"
```

---

### Task 2: Add `spin_to_fermion` transform

**Files:**
- Modify: `src/core/transform.rs`
- Create: `tests/unit_spin_to_fermion.rs`

**Step 1: Write the failing test**

Create `tests/unit_spin_to_fermion.rs`:

```rust
use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::transform::spin_to_fermion;
use smallvec::smallvec;

#[test]
fn spin_plus_to_fermion() {
    // Sp(0) → c†(0,↑) c(0,↓)
    let terms = vec![Term::new(1.0, smallvec![Op::SpinPlus(0)])];
    let result = spin_to_fermion(&terms);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ops.len(), 2);
    assert_eq!(result[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(result[0].ops[1], Op::FermionAnnihilate(0, Spin::Down));
    assert!((result[0].coeff - 1.0).abs() < 1e-12);
}

#[test]
fn spin_minus_to_fermion() {
    // Sm(0) → c†(0,↓) c(0,↑)
    let terms = vec![Term::new(1.0, smallvec![Op::SpinMinus(0)])];
    let result = spin_to_fermion(&terms);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ops[0], Op::FermionCreate(0, Spin::Down));
    assert_eq!(result[0].ops[1], Op::FermionAnnihilate(0, Spin::Up));
}

#[test]
fn spin_z_to_fermion() {
    // Sz(0) → 0.5 * c†(0,↑)c(0,↑) - 0.5 * c†(0,↓)c(0,↓)
    let terms = vec![Term::new(1.0, smallvec![Op::SpinZ(0)])];
    let result = spin_to_fermion(&terms);
    assert_eq!(result.len(), 2);

    // First term: +0.5 * c†(0,↑) c(0,↑)
    assert!((result[0].coeff - 0.5).abs() < 1e-12);
    assert_eq!(result[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(result[0].ops[1], Op::FermionAnnihilate(0, Spin::Up));

    // Second term: -0.5 * c†(0,↓) c(0,↓)
    assert!((result[1].coeff - (-0.5)).abs() < 1e-12);
    assert_eq!(result[1].ops[0], Op::FermionCreate(0, Spin::Down));
    assert_eq!(result[1].ops[1], Op::FermionAnnihilate(0, Spin::Down));
}

#[test]
fn spin_z_with_coefficient() {
    // 2.0 * Sz(0) → 1.0 * c†c↑ - 1.0 * c†c↓
    let terms = vec![Term::new(2.0, smallvec![Op::SpinZ(0)])];
    let result = spin_to_fermion(&terms);
    assert_eq!(result.len(), 2);
    assert!((result[0].coeff - 1.0).abs() < 1e-12);
    assert!((result[1].coeff - (-1.0)).abs() < 1e-12);
}

#[test]
fn sp_sm_product() {
    // Sp(0) Sm(1) → c†(0,↑)c(0,↓) c†(1,↓)c(1,↑)
    let terms = vec![Term::new(0.5, smallvec![Op::SpinPlus(0), Op::SpinMinus(1)])];
    let result = spin_to_fermion(&terms);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ops.len(), 4);
    assert_eq!(result[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(result[0].ops[1], Op::FermionAnnihilate(0, Spin::Down));
    assert_eq!(result[0].ops[2], Op::FermionCreate(1, Spin::Down));
    assert_eq!(result[0].ops[3], Op::FermionAnnihilate(1, Spin::Up));
}

#[test]
fn sz_sz_product() {
    // Sz(0) Sz(1) → 4 terms from (0.5*n↑ - 0.5*n↓)(0.5*n↑ - 0.5*n↓)
    let terms = vec![Term::new(1.0, smallvec![Op::SpinZ(0), Op::SpinZ(1)])];
    let result = spin_to_fermion(&terms);
    assert_eq!(result.len(), 4);

    // +0.25 * n(0,↑) n(1,↑)
    assert!((result[0].coeff - 0.25).abs() < 1e-12);
    assert_eq!(result[0].ops.len(), 4);

    // -0.25 * n(0,↑) n(1,↓)
    assert!((result[1].coeff - (-0.25)).abs() < 1e-12);

    // -0.25 * n(0,↓) n(1,↑)
    assert!((result[2].coeff - (-0.25)).abs() < 1e-12);

    // +0.25 * n(0,↓) n(1,↓)
    assert!((result[3].coeff - 0.25).abs() < 1e-12);
}

#[test]
fn fermion_ops_pass_through() {
    // c†(0,↑) c(1,↑) should be unchanged
    let terms = vec![Term::new(-1.0, smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(1, Spin::Up),
    ])];
    let result = spin_to_fermion(&terms);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(result[0].ops[1], Op::FermionAnnihilate(1, Spin::Up));
    assert!((result[0].coeff - (-1.0)).abs() < 1e-12);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_spin_to_fermion`
Expected: FAIL — `spin_to_fermion` not found.

**Step 3: Implement spin_to_fermion**

Add to `src/core/transform.rs`:

```rust
/// Convert all spin operators in terms to fermionic form.
/// - Sp(i) → c†(i,↑) c(i,↓)
/// - Sm(i) → c†(i,↓) c(i,↑)
/// - Sz(i) → 0.5*c†(i,↑)c(i,↑) - 0.5*c†(i,↓)c(i,↓)
///
/// Sz causes Term splitting: one Term with Sz becomes two Terms.
/// For products like Sz(i)*Sz(j), this produces 2*2=4 Terms.
pub fn spin_to_fermion(terms: &[Term]) -> Vec<Term> {
    let mut result = Vec::new();

    for term in terms {
        // Process ops left to right. For each op, we may produce
        // 1 replacement (Sp, Sm, fermion) or 2 replacements (Sz).
        // We build up partial terms as a Vec and expand as needed.
        let mut partials: Vec<(f64, SmallVec<[Op; 4]>)> = vec![(term.coeff, SmallVec::new())];

        for op in &term.ops {
            match op {
                Op::SpinPlus(site) => {
                    for (_, ops) in &mut partials {
                        ops.push(Op::FermionCreate(*site, Spin::Up));
                        ops.push(Op::FermionAnnihilate(*site, Spin::Down));
                    }
                }
                Op::SpinMinus(site) => {
                    for (_, ops) in &mut partials {
                        ops.push(Op::FermionCreate(*site, Spin::Down));
                        ops.push(Op::FermionAnnihilate(*site, Spin::Up));
                    }
                }
                Op::SpinZ(site) => {
                    // Each partial splits into two:
                    // +0.5 * c†(site,↑)c(site,↑)
                    // -0.5 * c†(site,↓)c(site,↓)
                    let mut new_partials = Vec::with_capacity(partials.len() * 2);
                    for (coeff, ops) in &partials {
                        let mut ops_up = ops.clone();
                        ops_up.push(Op::FermionCreate(*site, Spin::Up));
                        ops_up.push(Op::FermionAnnihilate(*site, Spin::Up));
                        new_partials.push((*coeff * 0.5, ops_up));

                        let mut ops_down = ops.clone();
                        ops_down.push(Op::FermionCreate(*site, Spin::Down));
                        ops_down.push(Op::FermionAnnihilate(*site, Spin::Down));
                        new_partials.push((*coeff * -0.5, ops_down));
                    }
                    partials = new_partials;
                }
                // Fermion ops pass through unchanged
                other => {
                    for (_, ops) in &mut partials {
                        ops.push(*other);
                    }
                }
            }
        }

        for (coeff, ops) in partials {
            result.push(Term::new(coeff, ops));
        }
    }

    result
}
```

Also add at the top of `transform.rs`:

```rust
use smallvec::SmallVec;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_spin_to_fermion`
Expected: All 7 tests PASS.

**Step 5: Commit**

```bash
git add src/core/transform.rs tests/unit_spin_to_fermion.rs
git commit -m "feat: add spin_to_fermion transform (Sp/Sm/Sz → c†c)"
```

---

### Task 3: Add `cisajs.def` and `cisajscktaltdc.def` output formatters

**Files:**
- Modify: `src/output/mvmc.rs`
- Create: `tests/unit_mvmc_green.rs`

**Step 1: Write the failing test**

Create `tests/unit_mvmc_green.rs`:

```rust
use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::output::mvmc::{generate_cisajs_def, generate_cisajscktaltdc_def};
use smallvec::smallvec;

#[test]
fn cisajs_format() {
    let terms = vec![
        Term::new(1.0, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
        ]),
        Term::new(0.5, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(1, Spin::Up),
        ]),
    ];
    let output = generate_cisajs_def(&terms);
    assert!(output.contains("NCisAjs"));
    assert!(output.contains("2")); // 2 entries
    // Format: i si j sj
    assert!(output.contains("0     0     0     0"));
    assert!(output.contains("0     0     1     0"));
}

#[test]
fn cisajscktaltdc_format() {
    let terms = vec![
        Term::new(0.5, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Down),
            Op::FermionCreate(1, Spin::Down),
            Op::FermionAnnihilate(1, Spin::Up),
        ]),
    ];
    let output = generate_cisajscktaltdc_def(&terms);
    assert!(output.contains("NCisAjsCktAltDC"));
    assert!(output.contains("1")); // 1 entry
    // Format: i si j sj k sk l sl
    assert!(output.contains("0     0     0     1     1     1     1     0"));
}

#[test]
fn cisajscktaltdc_empty() {
    let terms: Vec<Term> = vec![];
    let output = generate_cisajscktaltdc_def(&terms);
    assert!(output.contains("NCisAjsCktAltDC"));
    assert!(output.contains("0")); // 0 entries
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_mvmc_green`
Expected: FAIL — `generate_cisajs_def` and `generate_cisajscktaltdc_def` not found.

**Step 3: Implement the formatters**

Add to `src/output/mvmc.rs`:

```rust
pub fn generate_cisajs_def(terms: &[Term]) -> String {
    let mut out = String::new();
    out.push_str("======================== \n");
    out.push_str(&format!("NCisAjs        {}  \n", terms.len()));
    out.push_str("======================== \n");
    out.push_str("========i_j_s_tijs====== \n");
    out.push_str("======================== \n");

    for term in terms {
        if term.ops.len() == 2 {
            if let (Op::FermionCreate(i, si), Op::FermionAnnihilate(j, sj)) = (term.ops[0], term.ops[1]) {
                out.push_str(&format!("    {}     {}     {}     {}\n",
                    i, spin_to_idx(si), j, spin_to_idx(sj)));
            }
        }
    }

    out
}

pub fn generate_cisajscktaltdc_def(terms: &[Term]) -> String {
    let mut out = String::new();
    out.push_str("======================== \n");
    out.push_str(&format!("NCisAjsCktAltDC  {}  \n", terms.len()));
    out.push_str("======================== \n");
    out.push_str("========i_j_s_k_l_t===== \n");
    out.push_str("======================== \n");

    for term in terms {
        if term.ops.len() == 4 {
            if let (
                Op::FermionCreate(i, si),
                Op::FermionAnnihilate(j, sj),
                Op::FermionCreate(k, sk),
                Op::FermionAnnihilate(l, sl),
            ) = (term.ops[0], term.ops[1], term.ops[2], term.ops[3]) {
                out.push_str(&format!("    {}     {}     {}     {}     {}     {}     {}     {}\n",
                    i, spin_to_idx(si), j, spin_to_idx(sj),
                    k, spin_to_idx(sk), l, spin_to_idx(sl)));
            }
        }
    }

    out
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_mvmc_green`
Expected: All 3 tests PASS.

**Step 5: Commit**

```bash
git add src/output/mvmc.rs tests/unit_mvmc_green.rs
git commit -m "feat: add cisajs.def and cisajscktaltdc.def output formatters"
```

---

### Task 4: Add `correlation_summary.txt` formatter

**Files:**
- Create: `src/output/correlation.rs`
- Modify: `src/output/mod.rs` (add `pub mod correlation;`)
- Create: `tests/unit_correlation_output.rs`

**Step 1: Write the failing test**

Create `tests/unit_correlation_output.rs`:

```rust
use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::output::correlation::generate_correlation_summary;
use smallvec::smallvec;

#[test]
fn summary_format_two_body() {
    let terms = vec![
        Term::new(0.5, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Down),
            Op::FermionCreate(1, Spin::Down),
            Op::FermionAnnihilate(1, Spin::Up),
        ]),
        Term::new(-0.25, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
            Op::FermionCreate(1, Spin::Down),
            Op::FermionAnnihilate(1, Spin::Down),
        ]),
    ];
    let output = generate_correlation_summary(&terms);
    assert!(output.contains("+0.5"));
    assert!(output.contains("c†(0,up)"));
    assert!(output.contains("c(0,down)"));
    assert!(output.contains("-0.25"));
}

#[test]
fn summary_format_one_body() {
    let terms = vec![
        Term::new(1.0, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
        ]),
    ];
    let output = generate_correlation_summary(&terms);
    assert!(output.contains("+1"));
    assert!(output.contains("c†(0,up)"));
    assert!(output.contains("c(0,up)"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_correlation_output`
Expected: FAIL — module `correlation` not found.

**Step 3: Implement correlation summary formatter**

Create `src/output/correlation.rs`:

```rust
use crate::core::op::{Op, Spin, Term};

fn format_spin(s: Spin) -> &'static str {
    match s {
        Spin::Up => "up",
        Spin::Down => "down",
    }
}

fn format_op(op: &Op) -> String {
    match op {
        Op::FermionCreate(site, spin) => format!("c†({},{})", site, format_spin(*spin)),
        Op::FermionAnnihilate(site, spin) => format!("c({},{})", site, format_spin(*spin)),
        Op::SpinPlus(site) => format!("Sp({})", site),
        Op::SpinMinus(site) => format!("Sm({})", site),
        Op::SpinZ(site) => format!("Sz({})", site),
    }
}

pub fn generate_correlation_summary(terms: &[Term]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Correlation function: {} terms\n", terms.len()));

    for term in terms {
        let sign = if term.coeff >= 0.0 { "+" } else { "" };
        let ops_str: Vec<String> = term.ops.iter().map(|op| format_op(op)).collect();
        out.push_str(&format!("  {}{} * {}\n", sign, term.coeff, ops_str.join(" ")));
    }

    out
}
```

Add to `src/output/mod.rs`:

```rust
pub mod correlation;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_correlation_output`
Expected: All 2 tests PASS.

**Step 5: Commit**

```bash
git add src/output/correlation.rs src/output/mod.rs tests/unit_correlation_output.rs
git commit -m "feat: add correlation_summary.txt formatter"
```

---

### Task 5: Add `--correlation` CLI flag and wire up the pipeline

**Files:**
- Modify: `src/main.rs`

**Step 1: Add CLI flag**

In `src/main.rs`, add to the `Cli` struct:

```rust
/// Correlation function input file (generates cisajs/cisajscktaltdc files)
#[arg(long)]
correlation: Option<PathBuf>,
```

Make the `input` field optional to allow correlation-only mode:

```rust
/// Input DSL file (Hamiltonian)
input: Option<PathBuf>,
```

**Step 2: Add correlation pipeline function**

Add to `src/main.rs`:

```rust
fn run_correlation_pipeline(corr_path: &Path, output_dir: &Path) {
    let input = std::fs::read_to_string(corr_path)
        .unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", corr_path.display(), e);
            std::process::exit(1);
        });

    let model = parser::parse(&input)
        .unwrap_or_else(|e| {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        });

    eprintln!("Parsed correlation: {} sites, {} sum blocks", model.lattice.num_sites, model.sum_blocks.len());

    // Expand sum loops
    let ham = core::expand::expand(&model);
    eprintln!("Expanded: {} terms", ham.terms.len());

    // Spin to fermion
    let terms = core::transform::spin_to_fermion(&ham.terms);
    eprintln!("After spin→fermion: {} terms", terms.len());

    // Normal order
    let terms = core::normal::normal_order(&terms);
    eprintln!("Normal ordered: {} terms", terms.len());

    // Combine
    let terms = core::combine::combine(&terms);
    eprintln!("Combined: {} terms", terms.len());

    // Green's function reorder: split into one-body and two-body
    let mut one_body_terms: Vec<core::op::Term> = Vec::new();
    let mut two_body_terms: Vec<core::op::Term> = Vec::new();

    for term in &terms {
        match term.ops.len() {
            2 => one_body_terms.push(term.clone()),
            4 => {
                let decomp = core::green::reorder_green_function(&term.ops);
                for mut t in decomp.two_body {
                    t.coeff *= term.coeff;
                    two_body_terms.push(t);
                }
                for mut t in decomp.one_body_corrections {
                    t.coeff *= term.coeff;
                    one_body_terms.push(t);
                }
            }
            _ => {}
        }
    }

    eprintln!("Green reordered: {} one-body, {} two-body", one_body_terms.len(), two_body_terms.len());

    // Write output
    std::fs::create_dir_all(output_dir).unwrap_or_else(|e| {
        eprintln!("Error creating output directory: {}", e);
        std::process::exit(1);
    });

    let write = |name: &str, content: String| {
        std::fs::write(output_dir.join(name), content).unwrap_or_else(|e| {
            eprintln!("Error writing {}: {}", name, e);
            std::process::exit(1);
        });
    };

    // Combine all terms for summary
    let mut all_terms = Vec::new();
    all_terms.extend(one_body_terms.iter().cloned());
    all_terms.extend(two_body_terms.iter().cloned());

    write("cisajs.def", output::mvmc::generate_cisajs_def(&one_body_terms));
    write("cisajscktaltdc.def", output::mvmc::generate_cisajscktaltdc_def(&two_body_terms));
    write("correlation_summary.txt", output::correlation::generate_correlation_summary(&all_terms));

    eprintln!("Written correlation files to {}", output_dir.display());
}
```

**Step 3: Update main() to handle --correlation**

In `main()`, after `let cli = Cli::parse();`, add:

```rust
if let Some(ref corr_path) = cli.correlation {
    run_correlation_pipeline(corr_path, &cli.output);
    if cli.input.is_none() {
        return;
    }
}

let input_path = cli.input.as_ref().unwrap_or_else(|| {
    eprintln!("Error: either <INPUT> or --correlation is required");
    std::process::exit(1);
});
```

And update the existing `input` reading to use `input_path` instead of `cli.input`.

**Step 4: Manual test**

Create `examples/correlation_ss.qsl`:

```
lattice 1d sites=4 pbc=true

sum i=0..3:
  S(i) . S(i+1)
```

Run: `cargo run -- --correlation examples/correlation_ss.qsl -o output_corr`

Check that `output_corr/cisajscktaltdc.def`, `output_corr/cisajs.def`, and `output_corr/correlation_summary.txt` are generated correctly.

**Step 5: Commit**

```bash
git add src/main.rs examples/correlation_ss.qsl
git commit -m "feat: add --correlation CLI flag with full pipeline"
```

---

### Task 6: Integration test — S(i)·S(j) end-to-end

**Files:**
- Create: `tests/integration/test_correlation.rs`
- Modify: `Cargo.toml` (add `[[test]]` entry)

**Step 1: Write the test**

Create `tests/integration/test_correlation.rs`:

```rust
use quantum_simpl::parser::parse;
use quantum_simpl::core::expand::expand;
use quantum_simpl::core::transform::spin_to_fermion;
use quantum_simpl::core::normal::normal_order;
use quantum_simpl::core::combine::combine;
use quantum_simpl::core::green::reorder_green_function;
use quantum_simpl::core::op::{Op, Spin, Term};

fn run_correlation_pipeline(input: &str) -> (Vec<Term>, Vec<Term>) {
    let model = parse(input).unwrap();
    let ham = expand(&model);
    let terms = spin_to_fermion(&ham.terms);
    let terms = normal_order(&terms);
    let terms = combine(&terms);

    let mut one_body = Vec::new();
    let mut two_body = Vec::new();

    for term in &terms {
        match term.ops.len() {
            2 => one_body.push(term.clone()),
            4 => {
                let decomp = reorder_green_function(&term.ops);
                for mut t in decomp.two_body {
                    t.coeff *= term.coeff;
                    two_body.push(t);
                }
                for mut t in decomp.one_body_corrections {
                    t.coeff *= term.coeff;
                    one_body.push(t);
                }
            }
            _ => {}
        }
    }

    (one_body, two_body)
}

#[test]
fn ss_correlation_2site() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..0:
  S(i) . S(i+1)
"#;
    let (one_body, two_body) = run_correlation_pipeline(input);

    // S(0)·S(1) = 0.5*Sp(0)Sm(1) + 0.5*Sm(0)Sp(1) + Sz(0)Sz(1)
    //
    // Sp(0)Sm(1) → c†(0,↑)c(0,↓) c†(1,↓)c(1,↑)  (already c†cc†c)
    // Sm(0)Sp(1) → c†(0,↓)c(0,↑) c†(1,↑)c(1,↓)  (already c†cc†c)
    // Sz(0)Sz(1) → 4 terms of n(0,s)n(1,s') form (c†cc†c after normal order)
    //
    // Total: 6 two-body terms (some may combine)
    // Verify we get both one-body (from δ corrections) and two-body terms
    assert!(!two_body.is_empty(), "Should have two-body correlation terms");

    // All two-body terms should be in c†cc†c form
    for t in &two_body {
        assert_eq!(t.ops.len(), 4);
        assert!(t.ops[0].is_creation(), "ops[0] should be c†");
        assert!(t.ops[1].is_annihilation(), "ops[1] should be c");
        assert!(t.ops[2].is_creation(), "ops[2] should be c†");
        assert!(t.ops[3].is_annihilation(), "ops[3] should be c");
    }

    println!("=== S(0)·S(1) correlation ===");
    println!("Two-body terms ({}):", two_body.len());
    for t in &two_body {
        println!("  coeff={:.4} ops={:?}", t.coeff, t.ops);
    }
    println!("One-body terms ({}):", one_body.len());
    for t in &one_body {
        println!("  coeff={:.4} ops={:?}", t.coeff, t.ops);
    }
}

#[test]
fn nn_correlation_2site() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..0:
  n(i,up) n(i+1,up)
"#;
    let (one_body, two_body) = run_correlation_pipeline(input);

    // n(0,↑) n(1,↑) = c†(0,↑)c(0,↑) c†(1,↑)c(1,↑)
    // Already in c†cc†c form after normal ordering
    assert!(!two_body.is_empty());

    for t in &two_body {
        assert_eq!(t.ops.len(), 4);
        assert!(t.ops[0].is_creation());
        assert!(t.ops[1].is_annihilation());
        assert!(t.ops[2].is_creation());
        assert!(t.ops[3].is_annihilation());
    }
}

#[test]
fn ss_correlation_4site_pbc() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..3:
  S(i) . S(i+1)
"#;
    let (one_body, two_body) = run_correlation_pipeline(input);

    // 4 bonds × 6 terms per S·S = 24 terms before combining
    // Some may combine after normal ordering
    assert!(!two_body.is_empty());

    println!("=== 4-site PBC S·S correlation ===");
    println!("Two-body: {}, One-body (δ corrections): {}", two_body.len(), one_body.len());
}
```

**Step 2: Add test entry to Cargo.toml**

Add:
```toml
[[test]]
name = "test_correlation"
path = "tests/integration/test_correlation.rs"
```

**Step 3: Run the test**

Run: `cargo test --test test_correlation -- --nocapture`
Expected: All 3 tests PASS with printed output.

**Step 4: Commit**

```bash
git add tests/integration/test_correlation.rs Cargo.toml
git commit -m "test: add end-to-end correlation function integration tests"
```

---

## Summary

| Task | What | Key files |
|------|------|-----------|
| 1 | `S(i).S(j)` parser sugar | `parser/mod.rs` |
| 2 | `spin_to_fermion` transform | `transform.rs` |
| 3 | `cisajs.def` / `cisajscktaltdc.def` formatters | `mvmc.rs` |
| 4 | `correlation_summary.txt` formatter | `correlation.rs` |
| 5 | `--correlation` CLI + pipeline wiring | `main.rs` |
| 6 | End-to-end integration tests | `test_correlation.rs` |

Dependencies: Tasks 1-4 are independent. Task 5 depends on 1-4. Task 6 depends on 1-4 (can run without Task 5).
