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
