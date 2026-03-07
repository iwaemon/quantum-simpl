use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::symmetry::filter_sz_conserving;
use smallvec::smallvec;

#[test]
fn sz_conserving_terms_kept() {
    let terms = vec![
        Term::new(1.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Up)]),
        Term::new(1.0, smallvec![Op::SpinPlus(0), Op::SpinMinus(1)]),
        Term::new(1.0, smallvec![Op::SpinZ(0), Op::SpinZ(1)]),
    ];
    let result = filter_sz_conserving(&terms);
    assert_eq!(result.len(), 3);
}

#[test]
fn sz_nonconserving_terms_removed() {
    let terms = vec![
        Term::new(1.0, smallvec![Op::FermionCreate(0, Spin::Up), Op::FermionAnnihilate(1, Spin::Down)]),
        Term::new(1.0, smallvec![Op::SpinPlus(0), Op::SpinPlus(1)]),
    ];
    let result = filter_sz_conserving(&terms);
    assert_eq!(result.len(), 0);
}

#[test]
fn identity_term_preserved() {
    let terms = vec![
        Term::new(5.0, smallvec![]),
    ];
    let result = filter_sz_conserving(&terms);
    assert_eq!(result.len(), 1);
}
