use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::transform::spin_to_fermion;
use smallvec::smallvec;

#[test]
fn spin_plus_to_fermion() {
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
    let terms = vec![Term::new(1.0, smallvec![Op::SpinMinus(0)])];
    let result = spin_to_fermion(&terms);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ops.len(), 2);
    assert_eq!(result[0].ops[0], Op::FermionCreate(0, Spin::Down));
    assert_eq!(result[0].ops[1], Op::FermionAnnihilate(0, Spin::Up));
}

#[test]
fn spin_z_to_fermion() {
    let terms = vec![Term::new(1.0, smallvec![Op::SpinZ(0)])];
    let result = spin_to_fermion(&terms);
    assert_eq!(result.len(), 2);
    assert!((result[0].coeff - 0.5).abs() < 1e-12);
    assert_eq!(result[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(result[0].ops[1], Op::FermionAnnihilate(0, Spin::Up));
    assert!((result[1].coeff - (-0.5)).abs() < 1e-12);
    assert_eq!(result[1].ops[0], Op::FermionCreate(0, Spin::Down));
    assert_eq!(result[1].ops[1], Op::FermionAnnihilate(0, Spin::Down));
}

#[test]
fn spin_z_with_coefficient() {
    let terms = vec![Term::new(2.0, smallvec![Op::SpinZ(0)])];
    let result = spin_to_fermion(&terms);
    assert_eq!(result.len(), 2);
    assert!((result[0].coeff - 1.0).abs() < 1e-12);
    assert!((result[1].coeff - (-1.0)).abs() < 1e-12);
}

#[test]
fn sp_sm_product() {
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
    let terms = vec![Term::new(1.0, smallvec![Op::SpinZ(0), Op::SpinZ(1)])];
    let result = spin_to_fermion(&terms);
    assert_eq!(result.len(), 4);

    // +0.25 * c†(0,↑) c(0,↑) c†(1,↑) c(1,↑)
    assert!((result[0].coeff - 0.25).abs() < 1e-12);
    assert_eq!(result[0].ops.len(), 4);
    assert_eq!(result[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(result[0].ops[1], Op::FermionAnnihilate(0, Spin::Up));
    assert_eq!(result[0].ops[2], Op::FermionCreate(1, Spin::Up));
    assert_eq!(result[0].ops[3], Op::FermionAnnihilate(1, Spin::Up));

    // -0.25 * c†(0,↑) c(0,↑) c†(1,↓) c(1,↓)
    assert!((result[1].coeff - (-0.25)).abs() < 1e-12);

    // -0.25 * c†(0,↓) c(0,↓) c†(1,↑) c(1,↑)
    assert!((result[2].coeff - (-0.25)).abs() < 1e-12);

    // +0.25 * c†(0,↓) c(0,↓) c†(1,↓) c(1,↓)
    assert!((result[3].coeff - 0.25).abs() < 1e-12);
    assert_eq!(result[3].ops[0], Op::FermionCreate(0, Spin::Down));
    assert_eq!(result[3].ops[1], Op::FermionAnnihilate(0, Spin::Down));
    assert_eq!(result[3].ops[2], Op::FermionCreate(1, Spin::Down));
    assert_eq!(result[3].ops[3], Op::FermionAnnihilate(1, Spin::Down));
}

#[test]
fn fermion_ops_pass_through() {
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
