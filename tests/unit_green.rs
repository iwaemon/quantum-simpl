use quantum_simpl::core::op::{Op, Spin};
use quantum_simpl::core::green::{reorder_green_function};
use smallvec::SmallVec;

#[test]
fn reorder_cdagger_cdagger_c_c() {
    // c†(0,↑) c†(1,↓) c(1,↓) c(0,↑)
    // → -c†(0,↑) c(1,↓) c†(1,↓) c(0,↑) + δ_{1↓,1↓} * c†(0,↑) c(0,↑)
    let ops: SmallVec<[Op; 4]> = SmallVec::from_slice(&[
        Op::FermionCreate(0, Spin::Up),
        Op::FermionCreate(1, Spin::Down),
        Op::FermionAnnihilate(1, Spin::Down),
        Op::FermionAnnihilate(0, Spin::Up),
    ]);

    let decomp = reorder_green_function(&ops);

    assert_eq!(decomp.two_body.len(), 1);
    assert_eq!(decomp.two_body[0].coeff, -1.0);
    assert_eq!(decomp.two_body[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(decomp.two_body[0].ops[1], Op::FermionAnnihilate(1, Spin::Down));
    assert_eq!(decomp.two_body[0].ops[2], Op::FermionCreate(1, Spin::Down));
    assert_eq!(decomp.two_body[0].ops[3], Op::FermionAnnihilate(0, Spin::Up));

    assert_eq!(decomp.one_body_corrections.len(), 1);
    assert_eq!(decomp.one_body_corrections[0].coeff, 1.0);
    assert_eq!(decomp.one_body_corrections[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(decomp.one_body_corrections[0].ops[1], Op::FermionAnnihilate(0, Spin::Up));
}

#[test]
fn reorder_different_sites_no_delta() {
    // c†(0,↑) c†(1,↓) c(2,↓) c(3,↑)
    // → -c†(0,↑) c(2,↓) c†(1,↓) c(3,↑), no delta (sites 1≠2)
    let ops: SmallVec<[Op; 4]> = SmallVec::from_slice(&[
        Op::FermionCreate(0, Spin::Up),
        Op::FermionCreate(1, Spin::Down),
        Op::FermionAnnihilate(2, Spin::Down),
        Op::FermionAnnihilate(3, Spin::Up),
    ]);

    let decomp = reorder_green_function(&ops);

    assert_eq!(decomp.two_body.len(), 1);
    assert_eq!(decomp.two_body[0].coeff, -1.0);
    assert_eq!(decomp.two_body[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(decomp.two_body[0].ops[1], Op::FermionAnnihilate(2, Spin::Down));
    assert_eq!(decomp.two_body[0].ops[2], Op::FermionCreate(1, Spin::Down));
    assert_eq!(decomp.two_body[0].ops[3], Op::FermionAnnihilate(3, Spin::Up));

    assert_eq!(decomp.one_body_corrections.len(), 0);
}

#[test]
fn reorder_already_cicj_ckcl_form() {
    // c†(0,↑) c(1,↑) c†(2,↓) c(3,↓) — already in c†cc†c form
    let ops: SmallVec<[Op; 4]> = SmallVec::from_slice(&[
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(1, Spin::Up),
        Op::FermionCreate(2, Spin::Down),
        Op::FermionAnnihilate(3, Spin::Down),
    ]);

    let decomp = reorder_green_function(&ops);

    assert_eq!(decomp.two_body.len(), 1);
    assert_eq!(decomp.two_body[0].coeff, 1.0);
    assert_eq!(decomp.two_body[0].ops, ops);

    assert_eq!(decomp.one_body_corrections.len(), 0);
}
