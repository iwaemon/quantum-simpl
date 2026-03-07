use quantum_simpl::parser::parse;
use quantum_simpl::core::expand::expand;
use quantum_simpl::core::normal::normal_order;
use quantum_simpl::core::combine::combine;
use quantum_simpl::core::symmetry::filter_sz_conserving;
use quantum_simpl::core::op::Hamiltonian;
use quantum_simpl::output::mvmc::{generate_trans_def, generate_interall_def, write_all_files};
use tempfile::TempDir;

fn run_full_pipeline(input: &str) -> Hamiltonian {
    let model = parse(input).unwrap();
    let ham = expand(&model);
    let terms = normal_order(&ham.terms);
    let terms = combine(&terms);
    let terms = filter_sz_conserving(&terms);
    let mut final_ham = Hamiltonian::new(model.lattice.num_sites);
    for t in terms {
        final_ham.add_term(t);
    }
    final_ham
}

#[test]
fn trans_def_format() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  -t * c†(i,up) c(i+1,up) + h.c.

params:
  t = 1.0
"#;
    let ham = run_full_pipeline(input);
    let output = generate_trans_def(&ham);

    // Should have 2 transfer terms (hop + h.c.)
    assert!(output.contains("NTransfer      2"));
    // Check format: site_i spin_i site_j spin_j Re Im
    assert!(output.contains("-1.0000"));
}

#[test]
fn interall_def_format() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  U * n(i,up) n(i,down)

params:
  U = 4.0
"#;
    let ham = run_full_pipeline(input);
    let output = generate_interall_def(&ham);

    assert!(output.contains("TotalNumber"));
    assert!(output.contains("4.0"));
}

#[test]
fn default_template_files_generated() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..3:
  -t * c†(i,up) c(i+1,up) + h.c.

params:
  t = 1.0
"#;
    let ham = run_full_pipeline(input);
    let dir = TempDir::new().unwrap();
    write_all_files(&ham, dir.path()).unwrap();

    assert!(dir.path().join("modpara.def").exists());
    assert!(dir.path().join("locspn.def").exists());
    assert!(dir.path().join("namelist.def").exists());
    assert!(dir.path().join("gutzwilleridx.def").exists());
    assert!(dir.path().join("jastrowidx.def").exists());
    assert!(dir.path().join("orbitalidx.def").exists());
    assert!(dir.path().join("qptransidx.def").exists());

    // Verify modpara contains correct site count
    let modpara = std::fs::read_to_string(dir.path().join("modpara.def")).unwrap();
    assert!(modpara.contains("Nsite          4"));
}

#[test]
fn large_scale_term_preservation() {
    let input = r#"
lattice 1d sites=100 pbc=true

sum i=0..99:
  -t * c†(i,up) c(i+1,up) + h.c.
  -t * c†(i,down) c(i+1,down) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
"#;
    let ham = run_full_pipeline(input);
    let trans = generate_trans_def(&ham);
    let interall = generate_interall_def(&ham);

    // 100 sites PBC: 100 bonds × 2 spins × 2 directions = 400 one-body terms
    assert!(trans.contains("NTransfer      400"));

    // 100 interaction terms (plus possible normal-ordering contributions)
    assert!(interall.contains("TotalNumber"));

    // Count data lines in interall (lines after header with numbers)
    let data_lines: Vec<_> = interall.lines()
        .filter(|l| l.trim().starts_with(|c: char| c.is_ascii_digit()))
        .collect();
    assert!(data_lines.len() >= 100);
}
