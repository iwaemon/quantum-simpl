use quantum_simpl::parser::parse;
use quantum_simpl::core::expand::expand;
use quantum_simpl::core::normal::normal_order;
use quantum_simpl::core::combine::combine;
use quantum_simpl::core::symmetry::filter_sz_conserving;

fn run_pipeline(input: &str) -> Vec<quantum_simpl::core::op::Term> {
    let model = parse(input).unwrap();
    let ham = expand(&model);
    let terms = normal_order(&ham.terms);
    combine(&terms)
}

#[test]
fn heisenberg_2site_xxx() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  J * Sp(i) Sm(i+1)
  J * Sm(i) Sp(i+1)
  J * 2.0 * Sz(i) Sz(i+1)

params:
  J = 1.0
"#;
    let terms = run_pipeline(input);
    // 3 terms: Sp(0)Sm(1), Sm(0)Sp(1), 2*Sz(0)Sz(1)
    assert_eq!(terms.len(), 3);
}

#[test]
fn heisenberg_4site_pbc() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..4:
  J * Sp(i) Sm(i+1)
  J * Sm(i) Sp(i+1)
  J * 2.0 * Sz(i) Sz(i+1)

params:
  J = 1.0
"#;
    let terms = run_pipeline(input);
    // 4 bonds × 3 terms per bond = 12 terms
    assert_eq!(terms.len(), 12);
}

#[test]
fn spin_commutation_normal_order() {
    use smallvec::smallvec;
    use quantum_simpl::core::op::{Op, Term};

    // S-(0) S+(0) on same site should produce S+(0) S-(0) - 2Sz(0)
    let terms = vec![Term::new(1.0, smallvec![
        Op::SpinMinus(0),
        Op::SpinPlus(0),
    ])];
    let result = normal_order(&terms);
    let result = combine(&result);

    // Should have S+(0)S-(0) term and -2*Sz(0) term
    assert_eq!(result.len(), 2);

    let sp_sm = result.iter().find(|t| t.ops.len() == 2).unwrap();
    assert_eq!(sp_sm.coeff, 1.0);
    assert_eq!(sp_sm.ops[0], Op::SpinPlus(0));
    assert_eq!(sp_sm.ops[1], Op::SpinMinus(0));

    let sz = result.iter().find(|t| t.ops.len() == 1).unwrap();
    assert_eq!(sz.coeff, -2.0);
    assert_eq!(sz.ops[0], Op::SpinZ(0));
}

#[test]
fn sz_symmetry_preserves_heisenberg() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..4:
  J * Sp(i) Sm(i+1)
  J * Sm(i) Sp(i+1)
  J * 2.0 * Sz(i) Sz(i+1)

params:
  J = 1.0
"#;
    let terms = run_pipeline(input);
    let filtered = filter_sz_conserving(&terms);
    // All isotropic Heisenberg terms conserve Sz
    assert_eq!(terms.len(), filtered.len());
}
