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

#[test]
fn coulombintra_def_output() {
    use quantum_simpl::core::classify::ClassifiedTerms;
    use quantum_simpl::output::mvmc::generate_coulombintra_def;

    let classified = ClassifiedTerms {
        constants: vec![],
        one_body: vec![],
        coulomb_intra: vec![(0, 4.0), (1, 4.0)],
        two_body: vec![],
    };

    let output = generate_coulombintra_def(&classified);
    assert!(output.contains("N 2"));
    // Check both sites appear
    let lines: Vec<&str> = output.lines().collect();
    let data_lines: Vec<&str> = lines.iter().filter(|l| l.starts_with("0 ") || l.starts_with("1 ")).copied().collect();
    assert_eq!(data_lines.len(), 2);
}
