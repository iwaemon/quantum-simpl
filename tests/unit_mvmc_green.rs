use quantum_simpl::core::op::{Op, Spin, Term};
use quantum_simpl::output::mvmc::{generate_cisajs_def, generate_cisajscktaltdc_def};
use smallvec::smallvec;

#[test]
fn cisajs_format() {
    let terms = vec![
        Term::new(1.0, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Up),
        ]),
        Term::new(0.5, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(1, Spin::Up),
        ]),
    ];
    let output = generate_cisajs_def(&terms);
    assert!(output.contains("NCisAjs"));
    assert!(output.contains("2"));
    assert!(output.contains("0     0     0     0"));
    assert!(output.contains("0     0     1     0"));
}

#[test]
fn cisajscktaltdc_format() {
    let terms = vec![
        Term::new(0.5, smallvec![
            Op::FermionCreate(0, Spin::Up),
            Op::FermionAnnihilate(0, Spin::Down),
            Op::FermionCreate(1, Spin::Down),
            Op::FermionAnnihilate(1, Spin::Up),
        ]),
    ];
    let output = generate_cisajscktaltdc_def(&terms);
    assert!(output.contains("NCisAjsCktAltDC"));
    assert!(output.contains("1"));
    assert!(output.contains("0     0     0     1     1     1     1     0"));
}

#[test]
fn cisajscktaltdc_empty() {
    let terms: Vec<Term> = vec![];
    let output = generate_cisajscktaltdc_def(&terms);
    assert!(output.contains("NCisAjsCktAltDC"));
    assert!(output.contains("0"));
}
