use quantum_simpl::core::op::{Op, Spin, Term, Hamiltonian};
use quantum_simpl::output::mvmc::{generate_trans_def, generate_interall_def, generate_namelist};
use smallvec::smallvec;

#[test]
fn trans_def_one_body_format() {
    let mut ham = Hamiltonian::new(2);
    ham.add_term(Term::new(-1.0, smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(1, Spin::Up),
    ]));
    let output = generate_trans_def(&ham);
    assert!(output.contains("NTransfer"));
    assert!(output.contains("    0     0     1     0"));
    assert!(output.contains("-1.0000"));
}

#[test]
fn interall_def_two_body_format() {
    let mut ham = Hamiltonian::new(2);
    ham.add_term(Term::new(4.0, smallvec![
        Op::FermionCreate(0, Spin::Up),
        Op::FermionAnnihilate(0, Spin::Up),
        Op::FermionCreate(0, Spin::Down),
        Op::FermionAnnihilate(0, Spin::Down),
    ]));
    let output = generate_interall_def(&ham);
    assert!(output.contains("TotalNumber"));
    assert!(output.contains("4.0"));
}

#[test]
fn namelist_references_all_files() {
    let output = generate_namelist();
    assert!(output.contains("ModPara"));
    assert!(output.contains("Trans"));
    assert!(output.contains("LocSpin"));
    assert!(output.contains("InterAll"));
}

#[test]
fn trans_def_empty_if_no_one_body() {
    let ham = Hamiltonian::new(2);
    let output = generate_trans_def(&ham);
    assert!(output.contains("NTransfer      0"));
}
