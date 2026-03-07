use quantum_simpl::parser::parse;
use quantum_simpl::core::expand::expand;
use quantum_simpl::core::op::{Op, Spin};

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
    assert_eq!(ham.num_sites, 2);
    // i=0 only: 1 hop up + 1 hc up + 1 hop down + 1 hc down + 1 interaction = 5
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
    assert_eq!(ham.terms[0].num_ops(), 4);
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
    assert_eq!(ham.terms.len(), 8);
    let has_wrap = ham.terms.iter().any(|t| {
        t.ops.len() == 2
            && t.ops[0] == Op::FermionCreate(3, Spin::Up)
            && t.ops[1] == Op::FermionAnnihilate(0, Spin::Up)
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
    assert_eq!(ham.terms.len(), 6);
}
