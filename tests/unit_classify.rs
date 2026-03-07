use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::classify::classify_terms;
use smallvec::smallvec;

#[test]
fn classify_mixed_terms() {
    let terms = vec![
        // constant term (0 ops)
        Term::new(3.0, smallvec![]),
        // one-body term (2 ops)
        Term::new(-1.0, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(1, Spin::Up),
        ]),
        // two-body term (4 ops, different sites so not coulomb intra)
        Term::new(0.5, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
            Op::FermionCreate(1, Spin::Down),
            Op::FermionAnnihilate(1, Spin::Down),
        ]),
    ];

    let classified = classify_terms(&terms);
    assert_eq!(classified.constants.len(), 1);
    assert_eq!(classified.one_body.len(), 1);
    assert_eq!(classified.two_body.len(), 1);
    assert_eq!(classified.coulomb_intra.len(), 0);
    assert!((classified.offset() - 3.0).abs() < 1e-12);
}

#[test]
fn classify_coulomb_intra_detection() {
    // c†(0,up) c(0,up) c†(0,down) c(0,down) — same site, CoulombIntra
    let terms = vec![
        Term::new(4.0, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
            Op::FermionCreate(0, Spin::Down),
            Op::FermionAnnihilate(0, Spin::Down),
        ]),
    ];

    let classified = classify_terms(&terms);
    assert_eq!(classified.coulomb_intra.len(), 1);
    assert_eq!(classified.coulomb_intra[0].0, 0);
    assert!((classified.coulomb_intra[0].1 - 4.0).abs() < 1e-12);
    assert_eq!(classified.two_body.len(), 0);
}

#[test]
fn classify_coulomb_inter_not_intra() {
    // c†(0,up) c(0,up) c†(1,up) c(1,up) — different sites, NOT CoulombIntra
    let terms = vec![
        Term::new(2.0, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
            Op::FermionCreate(1, Spin::Up),
            Op::FermionAnnihilate(1, Spin::Up),
        ]),
    ];

    let classified = classify_terms(&terms);
    assert_eq!(classified.coulomb_intra.len(), 0);
    assert_eq!(classified.two_body.len(), 1);
}
