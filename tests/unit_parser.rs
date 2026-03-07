use quantum_simpl::parser::parse;

#[test]
fn parse_simple_hubbard() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  -t * c†(i,up) c(i+1,up) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
"#;
    let model = parse(input).unwrap();
    assert_eq!(model.lattice.num_sites, 2);
    assert!(!model.lattice.pbc);
    assert_eq!(model.sum_blocks.len(), 1);
    assert_eq!(model.sum_blocks[0].var, "i");
    assert_eq!(model.sum_blocks[0].range_start, 0);
    assert_eq!(model.sum_blocks[0].range_end, 1);
    assert_eq!(model.sum_blocks[0].expressions.len(), 2);
    assert_eq!(model.params.len(), 2);
}

#[test]
fn parse_heisenberg() {
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
    assert_eq!(model.lattice.num_sites, 4);
    assert!(model.lattice.pbc);
    assert_eq!(model.sum_blocks[0].expressions.len(), 3);
}

#[test]
fn parse_hermitian_conjugate_flag() {
    let input = r#"
lattice 1d sites=2 pbc=false

sum i=0..1:
  -t * c†(i,up) c(i+1,up) + h.c.

params:
  t = 1.0
"#;
    let model = parse(input).unwrap();
    assert!(model.sum_blocks[0].expressions[0].hermitian_conjugate);
}

#[test]
fn parse_error_on_invalid_input() {
    let input = "this is not valid DSL";
    assert!(parse(input).is_err());
}
