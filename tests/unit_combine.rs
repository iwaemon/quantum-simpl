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
