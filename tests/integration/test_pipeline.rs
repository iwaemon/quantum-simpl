use quantum_simpl::parser::parse;
use quantum_simpl::core::expand::expand;
use quantum_simpl::core::normal::normal_order;
use quantum_simpl::core::combine::combine;
use quantum_simpl::core::symmetry::filter_sz_conserving;
use quantum_simpl::output::mvmc::write_all_files;
use tempfile::TempDir;

#[test]
fn full_pipeline_hubbard_2site() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  -t * c†(i,up) c(i+1,up) + h.c.
  -t * c†(i,down) c(i+1,down) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
"#;
    let model = parse(input).unwrap();
    let ham = expand(&model);
    let terms = normal_order(&ham.terms);
    let terms = combine(&terms);
    let terms = filter_sz_conserving(&terms);

    let mut final_ham = quantum_simpl::core::op::Hamiltonian::new(model.lattice.num_sites);
    for t in terms {
        final_ham.add_term(t);
    }

    let dir = TempDir::new().unwrap();
    write_all_files(&final_ham, dir.path()).unwrap();

    assert!(dir.path().join("namelist.def").exists());
    assert!(dir.path().join("trans.def").exists());
    assert!(dir.path().join("interall.def").exists());
    assert!(dir.path().join("modpara.def").exists());
    assert!(dir.path().join("locspn.def").exists());
    assert!(dir.path().join("gutzwilleridx.def").exists());
    assert!(dir.path().join("jastrowidx.def").exists());
    assert!(dir.path().join("orbitalidx.def").exists());
    assert!(dir.path().join("qptransidx.def").exists());

    let trans = std::fs::read_to_string(dir.path().join("trans.def")).unwrap();
    assert!(trans.contains("NTransfer"));
}

#[test]
fn full_pipeline_heisenberg_4site() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..3:
  J * Sp(i) Sm(i+1)
  J * Sm(i) Sp(i+1)
  J * 2.0 * Sz(i) Sz(i+1)

params:
  J = 1.0
"#;
    let model = parse(input).unwrap();
    let ham = expand(&model);
    let terms = normal_order(&ham.terms);
    let terms = combine(&terms);
    let terms = filter_sz_conserving(&terms);

    // All isotropic Heisenberg terms conserve Sz
    assert!(terms.len() >= 12);
}
