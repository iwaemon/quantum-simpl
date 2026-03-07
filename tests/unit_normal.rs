use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::normal::normal_order;
use smallvec::smallvec;

#[test]
fn already_normal_ordered() {
    let terms = vec![Term::new(1.0, smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(1, Spin::Up),
    ])];
    let result = normal_order(&terms);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].coeff, 1.0);
}

#[test]
fn swap_c_cdagger_same_site_spin() {
    // c(0,up) c†(0,up) = -c†(0,up) c(0,up) + 1
    let terms = vec![Term::new(1.0, smallvec![
        Op::FermionAnnihilate(0, Spin::Up),
        Op::FermionCreate(0, Spin::Up),
    ])];
    let result = normal_order(&terms);
    assert_eq!(result.len(), 2);

    let normal_term = result.iter().find(|t| !t.ops.is_empty()).unwrap();
    assert_eq!(normal_term.coeff, -1.0);
    assert_eq!(normal_term.ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(normal_term.ops[1], Op::FermionAnnihilate(0, Spin::Up));

    let identity_term = result.iter().find(|t| t.ops.is_empty()).unwrap();
    assert_eq!(identity_term.coeff, 1.0);
}

#[test]
fn swap_c_cdagger_different_site() {
    // c(0,up) c†(1,up) = -c†(1,up) c(0,up)  (delta=0, different sites)
    let terms = vec![Term::new(1.0, smallvec![
        Op::FermionAnnihilate(0, Spin::Up),
        Op::FermionCreate(1, Spin::Up),
    ])];
    let result = normal_order(&terms);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].coeff, -1.0);
}

#[test]
fn number_operator_product_normal_order() {
    // c†(0,up) c(0,up) c†(0,down) c(0,down) — needs c†_down moved left past c_up
    let terms = vec![Term::new(4.0, smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(0, Spin::Up),
        Op::FermionCreate(0, Spin::Down),
        Op::FermionAnnihilate(0, Spin::Down),
    ])];
    let result = normal_order(&terms);
    assert!(!result.is_empty());
}

#[test]
fn spin_operators_different_sites_unchanged() {
    let terms = vec![Term::new(1.0, smallvec![
        Op::SpinPlus(0),
        Op::SpinMinus(1),
    ])];
    let result = normal_order(&terms);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ops.len(), 2);
}
