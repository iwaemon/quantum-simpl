use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::output::correlation::generate_correlation_summary;
use smallvec::smallvec;

#[test]
fn summary_format_two_body() {
    let terms = vec![
        Term::new(0.5, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Down),
            Op::FermionCreate(1, Spin::Down),
            Op::FermionAnnihilate(1, Spin::Up),
        ]),
        Term::new(-0.25, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
            Op::FermionCreate(1, Spin::Down),
            Op::FermionAnnihilate(1, Spin::Down),
        ]),
    ];
    let output = generate_correlation_summary(&terms);
    assert!(output.contains("+0.5"));
    assert!(output.contains("c†(0,up)"));
    assert!(output.contains("c(0,down)"));
    assert!(output.contains("-0.25"));
}

#[test]
fn summary_format_one_body() {
    let terms = vec![
        Term::new(1.0, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
        ]),
    ];
    let output = generate_correlation_summary(&terms);
    assert!(output.contains("+1"));
    assert!(output.contains("c†(0,up)"));
    assert!(output.contains("c(0,up)"));
}
