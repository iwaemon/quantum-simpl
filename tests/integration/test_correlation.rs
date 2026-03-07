use quantum_simpl::parser::parse;
use quantum_simpl::core::expand::expand;
use quantum_simpl::core::transform::spin_to_fermion;
use quantum_simpl::core::normal::normal_order;
use quantum_simpl::core::combine::combine;
use quantum_simpl::core::green::reorder_green_function;
use quantum_simpl::core::op::Term;

fn run_correlation_pipeline(input: &str) -> (Vec<Term>, Vec<Term>) {
    let model = parse(input).unwrap();
    let ham = expand(&model);
    let terms = spin_to_fermion(&ham.terms);
    let terms = normal_order(&terms);
    let terms = combine(&terms);

    let mut one_body = Vec::new();
    let mut two_body = Vec::new();

    for term in &terms {
        match term.ops.len() {
            2 => one_body.push(term.clone()),
            4 => {
                let decomp = reorder_green_function(&term.ops);
                for mut t in decomp.two_body {
                    t.coeff *= term.coeff;
                    two_body.push(t);
                }
                for mut t in decomp.one_body_corrections {
                    t.coeff *= term.coeff;
                    one_body.push(t);
                }
            }
            _ => {}
        }
    }

    (one_body, two_body)
}

#[test]
fn ss_correlation_2site() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..0:
  S(i) . S(i+1)
"#;
    let (one_body, two_body) = run_correlation_pipeline(input);

    // S(0)·S(1) = 0.5*Sp(0)Sm(1) + 0.5*Sm(0)Sp(1) + Sz(0)Sz(1)
    // Should produce two-body terms (some may combine)
    assert!(!two_body.is_empty(), "Should have two-body correlation terms");

    // All two-body terms should be in c†cc†c form
    for t in &two_body {
        assert_eq!(t.ops.len(), 4);
        assert!(t.ops[0].is_creation(), "ops[0] should be c†");
        assert!(t.ops[1].is_annihilation(), "ops[1] should be c");
        assert!(t.ops[2].is_creation(), "ops[2] should be c†");
        assert!(t.ops[3].is_annihilation(), "ops[3] should be c");
    }

    println!("=== S(0)·S(1) correlation ===");
    println!("Two-body terms ({}):", two_body.len());
    for t in &two_body {
        println!("  coeff={:.4} ops={:?}", t.coeff, t.ops);
    }
    println!("One-body terms ({}):", one_body.len());
    for t in &one_body {
        println!("  coeff={:.4} ops={:?}", t.coeff, t.ops);
    }
}

#[test]
fn nn_correlation_2site() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..0:
  n(i,up) n(i+1,up)
"#;
    let (_one_body, two_body) = run_correlation_pipeline(input);

    // n(0,up) n(1,up) = c†(0,up)c(0,up) c†(1,up)c(1,up)
    // Already in c†cc†c form after normal ordering
    assert!(!two_body.is_empty());

    for t in &two_body {
        assert_eq!(t.ops.len(), 4);
        assert!(t.ops[0].is_creation());
        assert!(t.ops[1].is_annihilation());
        assert!(t.ops[2].is_creation());
        assert!(t.ops[3].is_annihilation());
    }
}

#[test]
fn ss_correlation_4site_pbc() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..3:
  S(i) . S(i+1)
"#;
    let (one_body, two_body) = run_correlation_pipeline(input);

    // 4 bonds x terms per S·S before combining
    assert!(!two_body.is_empty());

    // All must be in c†cc†c form
    for t in &two_body {
        assert_eq!(t.ops.len(), 4);
        assert!(t.ops[0].is_creation());
        assert!(t.ops[1].is_annihilation());
        assert!(t.ops[2].is_creation());
        assert!(t.ops[3].is_annihilation());
    }

    println!("=== 4-site PBC S·S correlation ===");
    println!("Two-body: {}, One-body (delta corrections): {}", two_body.len(), one_body.len());
}
