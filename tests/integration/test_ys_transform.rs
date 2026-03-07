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

#[test]
fn ys_2site_hopping_signs() {
    // Test that up hopping stays -1.0 and down hopping flips to +1.0
    let input = "\
lattice 1d sites=2 pbc=false

sum i=0..0:
  -t * c†(i,up) c(i+1,up) + h.c.
  -t * c†(i,down) c(i+1,down) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
";
    let classified = run_ys_pipeline(input);

    // Find off-diagonal hopping terms
    for t in &classified.one_body {
        let (s0, sp0) = match t.ops[0] { Op::FermionCreate(s, sp) => (s, sp), _ => continue };
        let (s1, _sp1) = match t.ops[1] { Op::FermionAnnihilate(s, sp) => (s, sp), _ => continue };
        if s0 == s1 { continue; } // skip diagonal
        match sp0 {
            Spin::Up => assert!((t.coeff - (-1.0)).abs() < 1e-12, "Up hopping should be -1.0, got {}", t.coeff),
            Spin::Down => assert!((t.coeff - 1.0).abs() < 1e-12, "Down hopping should be +1.0, got {}", t.coeff),
        }
    }
}

#[test]
fn ys_2site_u_absorbed_into_trans() {
    // U*n_up*n_down -> -U*n_up*n_down + U*n_up
    // The U*n_up = U*c†(i,up)c(i,up) should appear as one-body diagonal terms
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

    // Check diagonal up-spin one-body terms with coeff U=4.0
    let u_diag: Vec<_> = classified.one_body.iter().filter(|t| {
        match (t.ops[0], t.ops[1]) {
            (Op::FermionCreate(s1, Spin::Up), Op::FermionAnnihilate(s2, Spin::Up)) if s1 == s2 => true,
            _ => false,
        }
    }).collect();
    assert_eq!(u_diag.len(), 2, "Should have U diagonal on both sites");
    for t in &u_diag {
        assert!((t.coeff - 4.0).abs() < 1e-12, "U diagonal coeff should be 4.0, got {}", t.coeff);
    }
}

#[test]
fn ys_2site_coulomb_intra_sign_flip() {
    // After YS transform and normal ordering:
    // U*n_up*n_down = U*c†↑c↑*c†↓c↓ → U*c†↑c↑*c↓c†↓ → U*c†↑c↑*(-c†↓c↓+1)
    //   = -U*c†↑c↑c†↓c↓ + U*c†↑c↑
    // Normal ordering c†↑c↑c†↓c↓ → -c†↑c†↓c↑c↓ (swapping c↑ past c†↓)
    // So: -U*(-c†↑c†↓c↑c↓) = +U*c†↑c†↓c↑c↓
    //
    // The fully normal-ordered 4-op terms have pattern c†↑c†↓c↑c↓ with coeff +U,
    // which is NOT the n↑n↓ = c†↑c↑c†↓c↓ pattern that detect_coulomb_intra expects.
    // They end up in two_body instead of coulomb_intra.
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

    // After YS + normal ordering, the interaction terms are in two_body
    // with fully normal-ordered form: c†(i,Up) c†(i,Down) c(i,Up) c(i,Down)
    // and coefficient +U = +4.0 (the sign flip from -U is absorbed by normal ordering)
    let intra_terms: Vec<_> = classified.two_body.iter().filter(|t| {
        match (t.ops[0], t.ops[1], t.ops[2], t.ops[3]) {
            (Op::FermionCreate(s0, Spin::Up), Op::FermionCreate(s1, Spin::Down),
             Op::FermionAnnihilate(s2, Spin::Up), Op::FermionAnnihilate(s3, Spin::Down))
                if s0 == s1 && s1 == s2 && s2 == s3 => true,
            _ => false,
        }
    }).collect();
    assert_eq!(intra_terms.len(), 2, "Should have on-site interaction on 2 sites");
    for t in &intra_terms {
        assert!((t.coeff - 4.0).abs() < 1e-12,
            "On-site interaction coeff should be +4.0 (from -U*(-normal_order_sign)), got {}", t.coeff);
    }
}
