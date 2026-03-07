use quantum_simpl::parser::parse;
use quantum_simpl::core::expand::expand;
use quantum_simpl::core::normal::normal_order;
use quantum_simpl::core::combine::combine;
use quantum_simpl::core::op::{Op, Spin};

fn run_pipeline(input: &str) -> Vec<quantum_simpl::core::op::Term> {
    let model = parse(input).unwrap();
    let ham = expand(&model);
    let terms = normal_order(&ham.terms);
    combine(&terms)
}

#[test]
fn hubbard_2site_term_count() {
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
    let terms = run_pipeline(input);
    // 4 hopping (one-body) + 1 interaction (two-body) + identity terms from normal ordering
    // One-body terms: c†(0,up)c(1,up), c†(1,up)c(0,up), c†(0,down)c(1,down), c†(1,down)c(0,down)
    let one_body: Vec<_> = terms.iter().filter(|t| t.ops.len() == 2).collect();
    let two_body: Vec<_> = terms.iter().filter(|t| t.ops.len() == 4).collect();
    assert_eq!(one_body.len(), 4);
    assert!(two_body.len() >= 1);
}

#[test]
fn hubbard_2site_coefficients() {
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
    let terms = run_pipeline(input);
    let one_body: Vec<_> = terms.iter().filter(|t| t.ops.len() == 2).collect();

    // All hopping coefficients should be -1.0
    for t in &one_body {
        assert!((t.coeff.abs() - 1.0).abs() < 1e-12,
            "Expected hopping coeff ±1.0, got {}", t.coeff);
    }
}

#[test]
fn hubbard_4site_pbc() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..4:
  -t * c†(i,up) c(i+1,up) + h.c.
  -t * c†(i,down) c(i+1,down) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
"#;
    let terms = run_pipeline(input);
    // 4 sites × 2 spins × 2 directions = 16 one-body terms
    // 4 sites × 1 interaction = 4 two-body terms (plus possible normal-ordering contributions)
    let one_body: Vec<_> = terms.iter().filter(|t| t.ops.len() == 2).collect();
    assert_eq!(one_body.len(), 16);
}

#[test]
fn normal_ordering_fermion_sign() {
    // c(0,up) c†(0,up) should become -c†(0,up) c(0,up) + 1
    use smallvec::smallvec;
    use quantum_simpl::core::op::Term;

    let terms = vec![Term::new(1.0, smallvec![
        Op::FermionAnnihilate(0, Spin::Up),
        Op::FermionCreate(0, Spin::Up),
    ])];
    let result = normal_order(&terms);
    let result = combine(&result);

    let normal = result.iter().find(|t| !t.ops.is_empty()).unwrap();
    assert_eq!(normal.coeff, -1.0);
    assert_eq!(normal.ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(normal.ops[1], Op::FermionAnnihilate(0, Spin::Up));

    let identity = result.iter().find(|t| t.ops.is_empty()).unwrap();
    assert_eq!(identity.coeff, 1.0);
}

#[test]
fn combine_like_terms() {
    use smallvec::smallvec;
    use quantum_simpl::core::op::Term;

    let terms = vec![
        Term::new(2.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Up)]),
        Term::new(3.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Up)]),
    ];
    let result = combine(&terms);
    assert_eq!(result.len(), 1);
    assert!((result[0].coeff - 5.0).abs() < 1e-12);
}

#[test]
fn combine_eliminates_zeros() {
    use smallvec::smallvec;
    use quantum_simpl::core::op::Term;

    let terms = vec![
        Term::new(1.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Up)]),
        Term::new(-1.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Up)]),
    ];
    let result = combine(&terms);
    assert_eq!(result.len(), 0);
}
