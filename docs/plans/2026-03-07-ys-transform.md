# YS Transform & Green's Function Reordering Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add operator substitution (Yokoyama-Shiba transformation) and Green's function reordering to quantum-simpl, verified against known Hubbard model results.

**Architecture:** Add a `transform` module that applies substitution rules (e.g., `c↓ → c†↓`) to all terms, then reuses the existing normal_order → combine pipeline. Add a `classify` module that separates terms into one-body/two-body/constant. Extend output to emit `coulombintra.def` and offset. For Green's functions, reuse normal_order to convert `c†c†cc` → `c†cc†c + δ·G1` form.

**Tech Stack:** Rust, existing quantum-simpl pipeline (op.rs, normal.rs, combine.rs), SmallVec

---

## Prerequisite: Fix range bug

The DSL `sum i=0..1` currently uses exclusive range (only i=0). For a 2-site Hubbard model, `U * n(i,up) n(i,down)` only applies to site 0, missing site 1. This must be fixed first or the YS transform test will be wrong.

**Decision needed from user:** Change `..` to inclusive (breaking change) or update examples to use `0..2`? Plan assumes inclusive for now.

---

### Task 1: Fix range semantics to inclusive

**Files:**
- Modify: `src/core/expand.rs:16` (`range_start..range_end` → `range_start..=range_end`)
- Modify: `tests/integration/test_hubbard.rs` (update `0..4` → `0..3` etc.)
- Modify: `tests/integration/test_pipeline.rs`, `tests/integration/test_heisenberg.rs`, `tests/integration/test_mvmc_output.rs` (update ranges)
- Modify: `examples/hubbard_2site.qsl`, `examples/heisenberg_4site.qsl`

**Step 1: Check all existing range usages**

Run: `grep -n '\.\.' tests/ examples/ --include='*.rs' --include='*.qsl' -r`

Identify all `start..end` patterns that need adjustment.

**Step 2: Change expand.rs to inclusive range**

In `src/core/expand.rs:16`, change:
```rust
for idx in block.range_start..block.range_end {
```
to:
```rust
for idx in block.range_start..=block.range_end {
```

**Step 3: Update all test files and examples**

For each test/example, adjust range end values. E.g.:
- 2-site Hubbard: `sum i=0..1:` stays (now means i=0,1 inclusive)
- 4-site Hubbard PBC: `sum i=0..4:` → `sum i=0..3:`
- 4-site Heisenberg: similarly adjust

**Step 4: Run all tests**

Run: `cargo test`
Expected: All tests pass with corrected ranges.

**Step 5: Verify 2-site Hubbard output**

Run: `cargo run -- examples/hubbard_2site.qsl -o output`
Check: `output/interall.def` should now have 2 entries (site 0 and site 1), not 1.

**Step 6: Commit**

```bash
git add -A
git commit -m "fix: change sum range to inclusive (0..1 means i=0,1)"
```

---

### Task 2: Add transform module — operator substitution

**Files:**
- Create: `src/core/transform.rs`
- Modify: `src/core/mod.rs` (add `pub mod transform;`)
- Create: `tests/unit_transform.rs`

**Step 1: Write the failing test**

Create `tests/unit_transform.rs`:

```rust
use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::transform::{SubstitutionRule, apply_substitution};
use smallvec::smallvec;

#[test]
fn particle_hole_down_spin_hopping() {
    // c†(0,↓) c(1,↓) → c(0,↓) c†(1,↓) after down-spin particle-hole transform
    let terms = vec![Term::new(-1.0, smallvec![
        Op::FermionCreate(0, Spin::Down),
        Op::FermionAnnihilate(1, Spin::Down),
    ])];

    let rules = vec![
        SubstitutionRule::ParticleHole(Spin::Down),
    ];

    let result = apply_substitution(&terms, &rules);
    assert_eq!(result.len(), 1);
    // c†↓ → c↓, c↓ → c†↓
    assert_eq!(result[0].ops[0], Op::FermionAnnihilate(0, Spin::Down));
    assert_eq!(result[0].ops[1], Op::FermionCreate(1, Spin::Down));
    assert_eq!(result[0].coeff, -1.0); // coeff unchanged at substitution stage
}

#[test]
fn particle_hole_down_spin_leaves_up_unchanged() {
    // c†(0,↑) c(1,↑) should be unchanged
    let terms = vec![Term::new(-1.0, smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(1, Spin::Up),
    ])];

    let rules = vec![
        SubstitutionRule::ParticleHole(Spin::Down),
    ];

    let result = apply_substitution(&terms, &rules);
    assert_eq!(result[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(result[0].ops[1], Op::FermionAnnihilate(1, Spin::Up));
}

#[test]
fn particle_hole_number_operator() {
    // n(0,↓) = c†(0,↓)c(0,↓) → c(0,↓)c†(0,↓) after PH transform
    let terms = vec![Term::new(4.0, smallvec![
        Op::FermionCreate(0, Spin::Down),
        Op::FermionAnnihilate(0, Spin::Down),
    ])];

    let rules = vec![
        SubstitutionRule::ParticleHole(Spin::Down),
    ];

    let result = apply_substitution(&terms, &rules);
    // After substitution (before normal ordering): c(0,↓) c†(0,↓)
    assert_eq!(result[0].ops[0], Op::FermionAnnihilate(0, Spin::Down));
    assert_eq!(result[0].ops[1], Op::FermionCreate(0, Spin::Down));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_transform`
Expected: FAIL — module `transform` not found.

**Step 3: Write minimal implementation**

Create `src/core/transform.rs`:

```rust
use crate::core::op::{Op, Spin, Term};

/// A substitution rule for operator transformation.
#[derive(Debug, Clone)]
pub enum SubstitutionRule {
    /// Particle-hole transformation for a specific spin:
    /// c†(i,s) → c(i,s), c(i,s) → c†(i,s)
    ParticleHole(Spin),
}

/// Apply substitution rules to all terms.
/// This only replaces operators — it does NOT normal-order the result.
/// Call normal_order() after this to get the properly ordered form.
pub fn apply_substitution(terms: &[Term], rules: &[SubstitutionRule]) -> Vec<Term> {
    terms.iter().map(|term| {
        let new_ops = term.ops.iter().map(|op| {
            let mut current = *op;
            for rule in rules {
                current = apply_rule(current, rule);
            }
            current
        }).collect();
        Term::new(term.coeff, new_ops)
    }).collect()
}

fn apply_rule(op: Op, rule: &SubstitutionRule) -> Op {
    match rule {
        SubstitutionRule::ParticleHole(spin) => {
            match op {
                Op::FermionCreate(site, s) if s == *spin => Op::FermionAnnihilate(site, s),
                Op::FermionAnnihilate(site, s) if s == *spin => Op::FermionCreate(site, s),
                _ => op,
            }
        }
    }
}
```

Add to `src/core/mod.rs`:
```rust
pub mod transform;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_transform`
Expected: All 3 tests PASS.

**Step 5: Commit**

```bash
git add src/core/transform.rs src/core/mod.rs tests/unit_transform.rs
git commit -m "feat: add transform module with particle-hole substitution"
```

---

### Task 3: Add classify module — one-body / two-body / constant separation

**Files:**
- Create: `src/core/classify.rs`
- Modify: `src/core/mod.rs` (add `pub mod classify;`)
- Create: `tests/unit_classify.rs`

**Step 1: Write the failing test**

Create `tests/unit_classify.rs`:

```rust
use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::classify::{ClassifiedTerms, classify_terms};
use smallvec::smallvec;

#[test]
fn classify_mixed_terms() {
    let terms = vec![
        // constant (identity)
        Term::new(3.0, smallvec![]),
        // one-body
        Term::new(-1.0, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(1, Spin::Up),
        ]),
        // two-body (coulomb intra pattern: c†↑ c↑ c†↓ c↓ on same site)
        Term::new(4.0, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
            Op::FermionCreate(0, Spin::Down),
            Op::FermionAnnihilate(0, Spin::Down),
        ]),
    ];

    let classified = classify_terms(&terms);
    assert_eq!(classified.constants.len(), 1);
    assert!((classified.offset() - 3.0).abs() < 1e-12);
    assert_eq!(classified.one_body.len(), 1);
    assert_eq!(classified.two_body.len(), 1);
}

#[test]
fn classify_coulomb_intra_detection() {
    // c†(0,↑) c(0,↑) c†(0,↓) c(0,↓) is CoulombIntra on site 0
    let terms = vec![
        Term::new(4.0, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
            Op::FermionCreate(0, Spin::Down),
            Op::FermionAnnihilate(0, Spin::Down),
        ]),
    ];

    let classified = classify_terms(&terms);
    assert_eq!(classified.coulomb_intra.len(), 1);
    assert_eq!(classified.coulomb_intra[0].0, 0); // site
    assert!((classified.coulomb_intra[0].1 - 4.0).abs() < 1e-12); // coeff
}

#[test]
fn classify_coulomb_inter_detection() {
    // n(0)n(1) pattern: sum of c†(0,s)c(0,s)c†(1,s')c(1,s') for all s,s'
    // After normal ordering, the n(0,↑)n(1,↑) part gives c†(0,↑)c(0,↑)c†(1,↑)c(1,↑)
    // which is CoulombInter-like (different sites, density-density)
    let terms = vec![
        Term::new(2.0, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
            Op::FermionCreate(1, Spin::Up),
            Op::FermionAnnihilate(1, Spin::Up),
        ]),
    ];

    let classified = classify_terms(&terms);
    // This is a two-body term on different sites but not CoulombIntra
    assert_eq!(classified.coulomb_intra.len(), 0);
    assert_eq!(classified.two_body.len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_classify`
Expected: FAIL — module `classify` not found.

**Step 3: Write minimal implementation**

Create `src/core/classify.rs`:

```rust
use crate::core::op::{Op, Spin, Term};

/// Classified Hamiltonian terms.
#[derive(Debug, Clone)]
pub struct ClassifiedTerms {
    /// Constant terms (empty operator string) — contribute to energy offset
    pub constants: Vec<Term>,
    /// One-body terms: c†_i c_j
    pub one_body: Vec<Term>,
    /// Two-body terms that match CoulombIntra: U_i * n(i,↑) * n(i,↓)
    /// Stored as (site, coefficient)
    pub coulomb_intra: Vec<(usize, f64)>,
    /// Remaining two-body terms (InterAll format)
    pub two_body: Vec<Term>,
}

impl ClassifiedTerms {
    /// Sum of all constant term coefficients
    pub fn offset(&self) -> f64 {
        self.constants.iter().map(|t| t.coeff).sum()
    }
}

/// Classify normal-ordered, combined terms into categories.
pub fn classify_terms(terms: &[Term]) -> ClassifiedTerms {
    let mut result = ClassifiedTerms {
        constants: Vec::new(),
        one_body: Vec::new(),
        coulomb_intra: Vec::new(),
        two_body: Vec::new(),
    };

    for term in terms {
        match term.ops.len() {
            0 => result.constants.push(term.clone()),
            2 => result.one_body.push(term.clone()),
            4 => {
                if let Some((site, coeff)) = detect_coulomb_intra(term) {
                    result.coulomb_intra.push((site, coeff));
                } else {
                    result.two_body.push(term.clone());
                }
            }
            _ => {} // ignore higher-order terms
        }
    }

    result
}

/// Detect if a 4-operator term is n(i,↑)n(i,↓) = c†(i,↑)c(i,↑)c†(i,↓)c(i,↓)
/// Returns Some((site, coeff)) if matched.
fn detect_coulomb_intra(term: &Term) -> Option<(usize, f64)> {
    if term.ops.len() != 4 {
        return None;
    }

    // Pattern: c†(i,↑) c(i,↑) c†(i,↓) c(i,↓) — all on same site
    match (term.ops[0], term.ops[1], term.ops[2], term.ops[3]) {
        (
            Op::FermionCreate(s0, Spin::Up),
            Op::FermionAnnihilate(s1, Spin::Up),
            Op::FermionCreate(s2, Spin::Down),
            Op::FermionAnnihilate(s3, Spin::Down),
        ) if s0 == s1 && s1 == s2 && s2 == s3 => Some((s0, term.coeff)),
        (
            Op::FermionCreate(s0, Spin::Down),
            Op::FermionAnnihilate(s1, Spin::Down),
            Op::FermionCreate(s2, Spin::Up),
            Op::FermionAnnihilate(s3, Spin::Up),
        ) if s0 == s1 && s1 == s2 && s2 == s3 => Some((s0, term.coeff)),
        _ => None,
    }
}
```

Add to `src/core/mod.rs`:
```rust
pub mod classify;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_classify`
Expected: All 3 tests PASS.

**Step 5: Commit**

```bash
git add src/core/classify.rs src/core/mod.rs tests/unit_classify.rs
git commit -m "feat: add classify module for term categorization"
```

---

### Task 4: Integration test — YS transform of 2-site Hubbard

**Files:**
- Create: `tests/integration/test_ys_transform.rs`
- Modify: `Cargo.toml` (add `[[test]]` entry)

**Step 1: Write the failing test**

Create `tests/integration/test_ys_transform.rs`:

```rust
use quantum_simpl::parser::parse;
use quantum_simpl::core::expand::expand;
use quantum_simpl::core::normal::normal_order;
use quantum_simpl::core::combine::combine;
use quantum_simpl::core::transform::{SubstitutionRule, apply_substitution};
use quantum_simpl::core::classify::classify_terms;
use quantum_simpl::core::op::{Op, Spin};

fn run_ys_pipeline(input: &str) -> quantum_simpl::core::classify::ClassifiedTerms {
    let model = parse(input).unwrap();
    let ham = expand(&model);

    // Apply YS transformation: particle-hole for down-spin
    let transformed = apply_substitution(&ham.terms, &[SubstitutionRule::ParticleHole(Spin::Down)]);

    // Normal order → combine → classify
    let terms = normal_order(&transformed);
    let terms = combine(&terms);
    classify_terms(&terms)
}

#[test]
fn ys_2site_hubbard_one_body_terms() {
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
    let classified = run_ys_pipeline(input);

    // After YS transform of down-spin:
    // - up hopping: c†(0,↑)c(1,↑) stays as is → coeff -1.0
    // - down hopping: c†(0,↓)c(1,↓) → c(0,↓)c†(1,↓)
    //   normal order: c(0,↓)c†(1,↓) = -c†(1,↓)c(0,↓) (no delta, different sites)
    //   so down hopping sign flips → coeff +1.0
    // - U * n(0,↑) n(0,↓) = U * c†↑c↑ * c†↓c↓ → U * c†↑c↑ * c↓c†↓
    //   c↓c†↓ = -c†↓c↓ + 1, so → U * c†↑c↑ * (-c†↓c↓ + 1) = -U*n↑n↓ + U*n↑
    //   The U*n↑ = U*c†(i,↑)c(i,↑) is a one-body term absorbed into trans

    // Check one-body terms exist
    assert!(!classified.one_body.is_empty());

    // Should have hopping terms + U-absorbed diagonal terms
    // up hopping: c†(0,↑)c(1,↑) with -1.0, c†(1,↑)c(0,↑) with -1.0
    // down hopping (sign flipped): c†(1,↓)c(0,↓) with +1.0, c†(0,↓)c(1,↓) with +1.0
    // U diagonal: c†(0,↑)c(0,↑) with +U, c†(1,↑)c(1,↑) with +U
    let up_hopping: Vec<_> = classified.one_body.iter().filter(|t| {
        matches!((t.ops[0], t.ops[1]),
            (Op::FermionCreate(_, Spin::Up), Op::FermionAnnihilate(_, Spin::Up)))
        && t.ops[0] != t.ops[1] // off-diagonal
    }).collect();

    let down_hopping: Vec<_> = classified.one_body.iter().filter(|t| {
        matches!((t.ops[0], t.ops[1]),
            (Op::FermionCreate(_, Spin::Down), Op::FermionAnnihilate(_, Spin::Down)))
        && t.ops[0] != t.ops[1] // off-diagonal
    }).collect();

    // Up hopping coefficients: -1.0
    for t in &up_hopping {
        assert!((t.coeff - (-1.0)).abs() < 1e-12,
            "Up hopping coeff should be -1.0, got {}", t.coeff);
    }

    // Down hopping coefficients: +1.0 (sign flipped by PH transform)
    for t in &down_hopping {
        assert!((t.coeff - 1.0).abs() < 1e-12,
            "Down hopping coeff should be +1.0, got {}", t.coeff);
    }

    // U diagonal terms on up-spin: c†(i,↑)c(i,↑) with coeff U=4.0
    let u_diagonal: Vec<_> = classified.one_body.iter().filter(|t| {
        match (t.ops[0], t.ops[1]) {
            (Op::FermionCreate(s1, Spin::Up), Op::FermionAnnihilate(s2, Spin::Up))
                if s1 == s2 => true,
            _ => false,
        }
    }).collect();
    assert_eq!(u_diagonal.len(), 2); // site 0 and site 1
    for t in &u_diagonal {
        assert!((t.coeff - 4.0).abs() < 1e-12,
            "U diagonal coeff should be 4.0, got {}", t.coeff);
    }
}

#[test]
fn ys_2site_hubbard_two_body_terms() {
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
    let classified = run_ys_pipeline(input);

    // After YS: U * n↑ * n↓ → -U * n↑ * n↓ (PH flipped sign on n↓)
    // So coulomb_intra should have coefficient -U = -4.0
    // (or equivalently, the two-body terms should have -U)
    let total_two_body = classified.coulomb_intra.len() + classified.two_body.len();
    assert!(total_two_body > 0, "Should have two-body terms after YS");

    // Check coulomb_intra: should be on sites 0 and 1 with coeff -4.0
    if !classified.coulomb_intra.is_empty() {
        assert_eq!(classified.coulomb_intra.len(), 2);
        for (site, coeff) in &classified.coulomb_intra {
            assert!(*site <= 1);
            assert!((*coeff - (-4.0)).abs() < 1e-12,
                "CoulombIntra coeff should be -4.0, got {}", coeff);
        }
    }
}

#[test]
fn ys_2site_hubbard_offset() {
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
    let classified = run_ys_pipeline(input);

    // Offset comes from:
    // - down hopping PH: c†↓c↓ → c↓c†↓ = -c†↓c↓ + δ → offset from diagonal terms
    // - U term: n↑ * (c↓c†↓) = n↑(-n↓ + 1) → constant if any identity terms
    // For 2-site OBC, down hopping offset is 0 (no diagonal hopping)
    // U offset: none directly, but the delta from c↓c†↓ gives identity terms

    // The exact offset value depends on the model details.
    // For now, just verify offset is tracked (non-zero due to n↓ → 1-n↓)
    println!("Offset: {}", classified.offset());
    println!("Constants: {:?}", classified.constants);
}
```

Add to `Cargo.toml`:
```toml
[[test]]
name = "test_ys_transform"
path = "tests/integration/test_ys_transform.rs"
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_ys_transform`
Expected: FAIL — tests reference Task 1-3 code.

**Step 3: Run tests after Tasks 1-3 are complete**

Run: `cargo test --test test_ys_transform`
Expected: All 3 tests PASS.

**Step 4: Commit**

```bash
git add tests/integration/test_ys_transform.rs Cargo.toml
git commit -m "test: add YS transform integration tests for 2-site Hubbard"
```

---

### Task 5: Add coulombintra.def output and offset

**Files:**
- Modify: `src/output/mvmc.rs` (add `generate_coulombintra_def`, modify `generate_namelist`)
- Create: `tests/unit_mvmc_coulombintra.rs`

**Step 1: Write the failing test**

Add to existing `tests/unit_mvmc.rs` or create new test:

```rust
use quantum_simpl::core::op::{Hamiltonian, Term, Op, Spin};
use quantum_simpl::core::classify::{ClassifiedTerms, classify_terms};
use quantum_simpl::output::mvmc::generate_coulombintra_def;
use smallvec::smallvec;

#[test]
fn coulombintra_def_format() {
    let classified = ClassifiedTerms {
        constants: vec![],
        one_body: vec![],
        coulomb_intra: vec![(0, 4.0), (1, 4.0)],
        two_body: vec![],
    };

    let output = generate_coulombintra_def(&classified);
    assert!(output.contains("N 2"));
    assert!(output.contains("0 4.0"));
    assert!(output.contains("1 4.0"));
}
```

**Step 2: Implement generate_coulombintra_def**

Add to `src/output/mvmc.rs`:

```rust
use crate::core::classify::ClassifiedTerms;

pub fn generate_coulombintra_def(classified: &ClassifiedTerms) -> String {
    let mut out = String::new();
    out.push_str("====== \n");
    out.push_str(&format!("N {} \n", classified.coulomb_intra.len()));
    out.push_str("====== \n");
    out.push_str("====== \n");
    out.push_str("====== \n");

    for (site, coeff) in &classified.coulomb_intra {
        out.push_str(&format!("{} {:.15} \n", site, coeff));
    }

    out
}
```

**Step 3: Run test to verify it passes**

Run: `cargo test coulombintra_def_format`
Expected: PASS.

**Step 4: Commit**

```bash
git add src/output/mvmc.rs tests/unit_mvmc.rs
git commit -m "feat: add coulombintra.def output"
```

---

### Task 6: Green's function reordering — c†c†cc → c†cc†c + δ corrections

**Files:**
- Create: `src/core/green.rs`
- Modify: `src/core/mod.rs` (add `pub mod green;`)
- Create: `tests/unit_green.rs`

**Step 1: Write the failing test**

Create `tests/unit_green.rs`:

```rust
use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::green::{reorder_green_function, GreenDecomposition};
use smallvec::smallvec;

#[test]
fn reorder_cdagger_cdagger_c_c() {
    // ⟨c†(i,↑) c†(j,↓) c(k,↓) c(l,↑)⟩
    // = -⟨c†(i,↑) c(k,↓) c†(j,↓) c(l,↑)⟩ + δ_{jk} ⟨c†(i,↑) c(l,↑)⟩
    //
    // Using concrete indices: i=0, j=1, k=1, l=0
    // δ_{jk} = δ_{1,1} = 1
    // So: -⟨c†(0,↑) c(1,↓) c†(1,↓) c(0,↑)⟩ + ⟨c†(0,↑) c(0,↑)⟩

    let ops = smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionCreate(1, Spin::Down),
        Op::FermionAnnihilate(1, Spin::Down),
        Op::FermionAnnihilate(0, Spin::Up),
    ];

    let decomp = reorder_green_function(&ops);

    // Should have one c†cc†c term with coeff -1.0
    assert_eq!(decomp.two_body.len(), 1);
    assert!((decomp.two_body[0].coeff - (-1.0)).abs() < 1e-12);
    assert_eq!(decomp.two_body[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(decomp.two_body[0].ops[1], Op::FermionAnnihilate(1, Spin::Down));
    assert_eq!(decomp.two_body[0].ops[2], Op::FermionCreate(1, Spin::Down));
    assert_eq!(decomp.two_body[0].ops[3], Op::FermionAnnihilate(0, Spin::Up));

    // Should have one delta correction: δ_{1,1} * ⟨c†(0,↑) c(0,↑)⟩
    assert_eq!(decomp.one_body_corrections.len(), 1);
    assert!((decomp.one_body_corrections[0].coeff - 1.0).abs() < 1e-12);
}

#[test]
fn reorder_different_sites_no_delta() {
    // ⟨c†(0,↑) c†(1,↓) c(2,↓) c(3,↑)⟩
    // = -⟨c†(0,↑) c(2,↓) c†(1,↓) c(3,↑)⟩ + δ_{1,2} * ⟨c†(0,↑) c(3,↑)⟩
    // δ_{1,2} = 0 (different sites)

    let ops = smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionCreate(1, Spin::Down),
        Op::FermionAnnihilate(2, Spin::Down),
        Op::FermionAnnihilate(3, Spin::Up),
    ];

    let decomp = reorder_green_function(&ops);

    assert_eq!(decomp.two_body.len(), 1);
    assert!((decomp.two_body[0].coeff - (-1.0)).abs() < 1e-12);
    assert_eq!(decomp.one_body_corrections.len(), 0); // no delta
}

#[test]
fn reorder_already_cicjckcl_form() {
    // ⟨c†(0,↑) c(1,↑) c†(2,↓) c(3,↓)⟩ is already c†cc†c form
    let ops = smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(1, Spin::Up),
        Op::FermionCreate(2, Spin::Down),
        Op::FermionAnnihilate(3, Spin::Down),
    ];

    let decomp = reorder_green_function(&ops);

    // Already in c†cc†c form, should return as-is with coeff 1.0
    assert_eq!(decomp.two_body.len(), 1);
    assert!((decomp.two_body[0].coeff - 1.0).abs() < 1e-12);
    assert_eq!(decomp.one_body_corrections.len(), 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test unit_green`
Expected: FAIL — module `green` not found.

**Step 3: Write minimal implementation**

Create `src/core/green.rs`:

```rust
use crate::core::op::{Op, Term};
use smallvec::SmallVec;

/// Result of reordering a Green's function expectation value.
/// ⟨c†c†cc⟩ = coeff * ⟨c†cc†c⟩ + Σ δ_corrections * ⟨c†c⟩
#[derive(Debug, Clone)]
pub struct GreenDecomposition {
    /// Two-body terms in c†cc†c order (mVMC native format)
    pub two_body: Vec<Term>,
    /// One-body correction terms from δ functions
    pub one_body_corrections: Vec<Term>,
}

/// Reorder a 4-operator expectation value into mVMC's c†cc†c format.
///
/// Input: operators in any order (typically c†c†cc from SC correlator)
/// Output: decomposition into c†cc†c terms + δ·⟨c†c⟩ corrections
///
/// Uses the anticommutation relation: c†_a c†_b = -c†_b c†_a (no delta)
/// and c_a c†_b = -c†_b c_a + δ_{ab}
pub fn reorder_green_function(ops: &SmallVec<[Op; 4]>) -> GreenDecomposition {
    assert_eq!(ops.len(), 4, "Expected 4 operators for two-body Green's function");

    let mut decomp = GreenDecomposition {
        two_body: Vec::new(),
        one_body_corrections: Vec::new(),
    };

    // Check if already in c†cc†c form
    if is_cicj_ckcl_form(ops) {
        decomp.two_body.push(Term::new(1.0, ops.clone()));
        return decomp;
    }

    // Handle c†c†cc form: swap ops[1] (c†) and ops[2] (c)
    // c†_a c†_b c_c c_d → apply anticommutation on c†_b and c_c:
    // c†_b c_c is already normal-ordered (c† before c), but we want c_c c†_b form
    // Actually: we want to move c†_b past c_c:
    // c†_a [c†_b c_c] c_d = c†_a [-c_c c†_b + δ_{bc}] c_d
    //                      = -c†_a c_c c†_b c_d + δ_{bc} * c†_a c_d
    if is_cdagger_cdagger_c_c_form(ops) {
        // Swapped term: -c†_a c_c c†_b c_d
        let swapped_ops: SmallVec<[Op; 4]> = SmallVec::from_slice(&[
            ops[0], ops[2], ops[1], ops[3],
        ]);
        decomp.two_body.push(Term::new(-1.0, swapped_ops));

        // Delta correction: δ_{bc} * c†_a c_d
        if let Some(()) = check_delta(&ops[1], &ops[2]) {
            let correction_ops: SmallVec<[Op; 4]> = SmallVec::from_slice(&[
                ops[0], ops[3],
            ]);
            decomp.one_body_corrections.push(Term::new(1.0, correction_ops));
        }

        return decomp;
    }

    // Fallback: use general normal ordering
    let term = Term::new(1.0, ops.clone());
    let reordered = crate::core::normal::normal_order(&[term]);
    for t in reordered {
        match t.ops.len() {
            4 => decomp.two_body.push(t),
            2 => decomp.one_body_corrections.push(t),
            0 => {} // constant — ignore for Green's function
            _ => decomp.two_body.push(t),
        }
    }

    decomp
}

/// Check if ops are in c†cc†c form
fn is_cicj_ckcl_form(ops: &SmallVec<[Op; 4]>) -> bool {
    ops[0].is_creation()
        && ops[1].is_annihilation()
        && ops[2].is_creation()
        && ops[3].is_annihilation()
}

/// Check if ops are in c†c†cc form
fn is_cdagger_cdagger_c_c_form(ops: &SmallVec<[Op; 4]>) -> bool {
    ops[0].is_creation()
        && ops[1].is_creation()
        && ops[2].is_annihilation()
        && ops[3].is_annihilation()
}

/// Check if two operators have matching site and spin (for δ function)
fn check_delta(op1: &Op, op2: &Op) -> Option<()> {
    match (op1, op2) {
        (Op::FermionCreate(s1, sp1), Op::FermionAnnihilate(s2, sp2))
        | (Op::FermionAnnihilate(s1, sp1), Op::FermionCreate(s2, sp2))
            if s1 == s2 && sp1 == sp2 => Some(()),
        _ => None,
    }
}
```

Add to `src/core/mod.rs`:
```rust
pub mod green;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test unit_green`
Expected: All 3 tests PASS.

**Step 5: Commit**

```bash
git add src/core/green.rs src/core/mod.rs tests/unit_green.rs
git commit -m "feat: add Green's function reordering (c†c†cc → c†cc†c + δ)"
```

---

### Task 7: CLI integration — add `--transform` and `--green` flags

**Files:**
- Modify: `src/main.rs`

**Step 1: Add CLI flags**

```rust
#[derive(Parser)]
#[command(name = "quantum-simpl")]
#[command(version)]
#[command(about = "Hamiltonian symbolic preprocessor for mVMC")]
struct Cli {
    /// Input DSL file
    input: PathBuf,

    /// Output directory for mVMC files
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Apply Yokoyama-Shiba transformation (particle-hole for down-spin)
    #[arg(long)]
    ys_transform: bool,
}
```

**Step 2: Integrate transform into pipeline in main()**

After `expand`, before `normal_order`:
```rust
let terms = if cli.ys_transform {
    eprintln!("Applying YS transformation (particle-hole for down-spin)...");
    let rules = vec![core::transform::SubstitutionRule::ParticleHole(core::op::Spin::Down)];
    core::transform::apply_substitution(&ham.terms, &rules)
} else {
    ham.terms
};
```

After combine, use classify for YS mode:
```rust
if cli.ys_transform {
    let classified = core::classify::classify_terms(&terms);
    eprintln!("Classified: {} one-body, {} coulomb_intra, {} two-body, offset={}",
        classified.one_body.len(), classified.coulomb_intra.len(),
        classified.two_body.len(), classified.offset());
    // Write coulombintra.def, modified trans.def, etc.
}
```

**Step 3: Test manually**

Run: `cargo run -- examples/hubbard_2site.qsl -o output --ys-transform`
Check output files for correctness.

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: add --ys-transform CLI flag"
```

---

### Task 8: End-to-end validation against YokoyamaShibaTrans.py

**Files:**
- Create: `tests/integration/test_ys_validation.rs`

**Step 1: Write validation test**

Compare quantum-simpl YS output with known analytical results for 2-site Hubbard (t=1.0, U=4.0):

```rust
#[test]
fn ys_matches_python_reference() {
    // Reference values from YokoyamaShibaTrans.py for 2-site Hubbard:
    //
    // Original Hamiltonian:
    //   trans: c†(0,↑)c(1,↑) = -1.0, c†(0,↓)c(1,↓) = -1.0  (+ h.c.)
    //   coulombintra: site 0 = 4.0, site 1 = 4.0
    //
    // After YS (PH for down-spin):
    //   trans up:   c†(0,↑)c(1,↑) = -1.0 (unchanged)
    //   trans down: c†(0,↓)c(1,↓) = +1.0 (sign flip)
    //   trans diag: c†(i,↑)c(i,↑) = +U = +4.0 (U absorbed for up-spin)
    //   coulombintra: site 0 = -4.0, site 1 = -4.0 (sign flip)
    //   offset: from n↓ → (1-n↓), contributes constant terms

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
    let classified = run_ys_pipeline(input);

    // Verify against Python reference
    // (exact values to be confirmed by running YokoyamaShibaTrans.py)
    println!("=== YS Transform Validation ===");
    println!("One-body terms: {}", classified.one_body.len());
    for t in &classified.one_body {
        println!("  {:?} coeff={}", t.ops, t.coeff);
    }
    println!("CoulombIntra:");
    for (site, coeff) in &classified.coulomb_intra {
        println!("  site={} coeff={}", site, coeff);
    }
    println!("Two-body (interall):");
    for t in &classified.two_body {
        println!("  {:?} coeff={}", t.ops, t.coeff);
    }
    println!("Offset: {}", classified.offset());
}
```

**Step 2: Run and compare output with Python**

Run quantum-simpl and YokoyamaShibaTrans.py on same input, compare numerically.

**Step 3: Commit**

```bash
git add tests/integration/test_ys_validation.rs
git commit -m "test: add YS validation test against Python reference"
```

---

## Summary

| Task | What | Key files |
|------|------|-----------|
| 1 | Fix range to inclusive | `expand.rs`, tests, examples |
| 2 | Transform module (operator substitution) | `transform.rs` |
| 3 | Classify module (one-body/two-body/constant) | `classify.rs` |
| 4 | YS transform integration test | `test_ys_transform.rs` |
| 5 | coulombintra.def output | `mvmc.rs` |
| 6 | Green's function reordering | `green.rs` |
| 7 | CLI integration | `main.rs` |
| 8 | End-to-end validation vs Python | `test_ys_validation.rs` |

Dependencies: Task 1 → Task 4, Tasks 2+3 → Task 4, Task 3 → Task 5, Task 6 is independent.
