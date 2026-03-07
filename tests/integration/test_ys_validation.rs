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
    let transformed = apply_substitution(&ham.terms, &[SubstitutionRule::ParticleHole(Spin::Down)]);
    let terms = normal_order(&transformed);
    let terms = combine(&terms);
    classify_terms(&terms)
}

/// Validate against analytical YS transformation of 2-site Hubbard model.
///
/// Original: H = -t Σ_σ (c†_{0σ}c_{1σ} + h.c.) + U Σ_i n_{i↑}n_{i↓}
///
/// YS (c↓ → c†↓, c†↓ → c↓):
///   - Up hopping unchanged: -t c†_{0↑}c_{1↑} + h.c.
///   - Down hopping: -t c†_{0↓}c_{1↓} → -t c_{0↓}c†_{1↓} = +t c†_{1↓}c_{0↓}
///     so down hopping sign flips
///   - U n↑ n↓ = U c†↑c↑ c†↓c↓ → U c†↑c↑ c↓c†↓ = U c†↑c↑(-c†↓c↓ + 1)
///     = -U n↑n↓ + U n↑
///   - Normal ordering -U c†↑c↑c†↓c↓: swap c↑ past c†↓ (different spin, δ=0)
///     → +U c†↑c†↓c↑c↓ (CoulombIntra with coeff +U)
#[test]
fn ys_2site_hubbard_full_validation() {
    let input = "\
lattice 1d sites=2 pbc=false

sum i=0..1:
  -t * c†(i,up) c(i+1,up) + h.c.
  -t * c†(i,down) c(i+1,down) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
";
    let classified = run_ys_pipeline(input);

    // === One-body terms (trans.def) ===
    // Expected 6 terms:
    //   Up hopping: c†(1,↑)c(0,↑) = -1.0, c†(0,↑)c(1,↑) = -1.0
    //   Down hopping (flipped): c†(1,↓)c(0,↓) = +1.0, c†(0,↓)c(1,↓) = +1.0
    //   U diagonal: c†(0,↑)c(0,↑) = +4.0, c†(1,↑)c(1,↑) = +4.0
    assert_eq!(classified.one_body.len(), 6, "Expected 6 one-body terms");

    // Check up hopping: coeff = -1.0
    let up_hopping: Vec<_> = classified.one_body.iter().filter(|t| {
        match (t.ops[0], t.ops[1]) {
            (Op::FermionCreate(s0, Spin::Up), Op::FermionAnnihilate(s1, Spin::Up))
                if s0 != s1 => true,
            _ => false,
        }
    }).collect();
    assert_eq!(up_hopping.len(), 2, "Expected 2 up-hopping terms");
    for t in &up_hopping {
        assert!((t.coeff - (-1.0)).abs() < 1e-12,
            "Up hopping coeff should be -1.0, got {}", t.coeff);
    }

    // Check down hopping: coeff = +1.0 (sign flipped by PH)
    let down_hopping: Vec<_> = classified.one_body.iter().filter(|t| {
        match (t.ops[0], t.ops[1]) {
            (Op::FermionCreate(s0, Spin::Down), Op::FermionAnnihilate(s1, Spin::Down))
                if s0 != s1 => true,
            _ => false,
        }
    }).collect();
    assert_eq!(down_hopping.len(), 2, "Expected 2 down-hopping terms");
    for t in &down_hopping {
        assert!((t.coeff - 1.0).abs() < 1e-12,
            "Down hopping coeff should be +1.0 (YS sign flip), got {}", t.coeff);
    }

    // Check U diagonal on up-spin: coeff = +U = +4.0
    let u_diag: Vec<_> = classified.one_body.iter().filter(|t| {
        match (t.ops[0], t.ops[1]) {
            (Op::FermionCreate(s0, Spin::Up), Op::FermionAnnihilate(s1, Spin::Up))
                if s0 == s1 => true,
            _ => false,
        }
    }).collect();
    assert_eq!(u_diag.len(), 2, "Expected U diagonal on both sites");
    for t in &u_diag {
        assert!((t.coeff - 4.0).abs() < 1e-12,
            "U diagonal coeff should be +4.0, got {}", t.coeff);
    }

    // === CoulombIntra ===
    // YS: U*n↑n↓ → -U*n↑n↓ (sign flip), so CoulombIntra = -U = -4.0
    assert_eq!(classified.coulomb_intra.len(), 2, "Expected CoulombIntra on 2 sites");
    for &(site, coeff) in &classified.coulomb_intra {
        assert!(site <= 1, "CoulombIntra site should be 0 or 1, got {}", site);
        assert!((coeff - (-4.0)).abs() < 1e-12,
            "CoulombIntra coeff should be -4.0 (YS sign flip), got {}", coeff);
    }

    // === No remaining interall terms ===
    assert_eq!(classified.two_body.len(), 0,
        "All two-body terms should be classified as CoulombIntra");

    // === Verify sites covered ===
    let mut intra_sites: Vec<usize> = classified.coulomb_intra.iter().map(|&(s, _)| s).collect();
    intra_sites.sort();
    assert_eq!(intra_sites, vec![0, 1], "CoulombIntra should cover sites 0 and 1");
}

/// Verify that without YS transform, the original Hamiltonian is preserved.
#[test]
fn original_hamiltonian_unchanged() {
    let input = "\
lattice 1d sites=2 pbc=false

sum i=0..1:
  -t * c†(i,up) c(i+1,up) + h.c.
  -t * c†(i,down) c(i+1,down) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
";
    let model = parse(input).unwrap();
    let ham = expand(&model);
    // No YS transform — go straight to normal order
    let terms = normal_order(&ham.terms);
    let terms = combine(&terms);
    let classified = classify_terms(&terms);

    // Original: 4 hopping terms (2 up + 2 down, all coeff -1.0), 2 CoulombIntra
    assert_eq!(classified.one_body.len(), 4, "Expected 4 hopping terms");
    for t in &classified.one_body {
        assert!((t.coeff - (-1.0)).abs() < 1e-12,
            "All hopping coeffs should be -1.0, got {}", t.coeff);
    }
    // detect_coulomb_intra converts normal-ordered c†↑c†↓c↑c↓ back to physical n↑n↓ coeff
    // Original U = +4.0
    assert_eq!(classified.coulomb_intra.len(), 2, "Expected CoulombIntra on 2 sites");
    for &(_site, coeff) in &classified.coulomb_intra {
        assert!((coeff - 4.0).abs() < 1e-12,
            "Original CoulombIntra should be +4.0, got {}", coeff);
    }
    assert_eq!(classified.two_body.len(), 0);
}
