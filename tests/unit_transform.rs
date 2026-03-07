use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::core::transform::{apply_substitution, SubstitutionRule};
use smallvec::smallvec;

#[test]
fn particle_hole_down_spin_hopping() {
    // c†(0,↓)c(1,↓) → c(0,↓)c†(1,↓)
    let terms = vec![Term::new(
        1.0,
        smallvec![
            Op::FermionCreate(0, Spin::Down),
            Op::FermionAnnihilate(1, Spin::Down),
        ],
    )];
    let rules = vec![SubstitutionRule::ParticleHole(Spin::Down)];
    let result = apply_substitution(&terms, &rules);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].coeff, 1.0);
    assert_eq!(result[0].ops.len(), 2);
    assert_eq!(result[0].ops[0], Op::FermionAnnihilate(0, Spin::Down));
    assert_eq!(result[0].ops[1], Op::FermionCreate(1, Spin::Down));
}

#[test]
fn particle_hole_down_spin_leaves_up_unchanged() {
    // c†(0,↑)c(1,↑) should remain unchanged under ParticleHole(Down)
    let terms = vec![Term::new(
        -1.0,
        smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(1, Spin::Up),
        ],
    )];
    let rules = vec![SubstitutionRule::ParticleHole(Spin::Down)];
    let result = apply_substitution(&terms, &rules);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].coeff, -1.0);
    assert_eq!(result[0].ops[0], Op::FermionCreate(0, Spin::Up));
    assert_eq!(result[0].ops[1], Op::FermionAnnihilate(1, Spin::Up));
}

#[test]
fn particle_hole_number_operator() {
    // c†(0,↓)c(0,↓) → c(0,↓)c†(0,↓)
    let terms = vec![Term::new(
        1.0,
        smallvec![
            Op::FermionCreate(0, Spin::Down),
            Op::FermionAnnihilate(0, Spin::Down),
        ],
    )];
    let rules = vec![SubstitutionRule::ParticleHole(Spin::Down)];
    let result = apply_substitution(&terms, &rules);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].coeff, 1.0);
    assert_eq!(result[0].ops[0], Op::FermionAnnihilate(0, Spin::Down));
    assert_eq!(result[0].ops[1], Op::FermionCreate(0, Spin::Down));
}
