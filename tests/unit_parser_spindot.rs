use quantum_simpl::parser::parse;
use quantum_simpl::parser::ast::*;

#[test]
fn parse_spindot_expression() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..3:
  S(i) . S(i+1)
"#;
    let model = parse(input).unwrap();
    assert_eq!(model.sum_blocks.len(), 1);
    let block = &model.sum_blocks[0];
    // S(i).S(j) should expand into 3 expressions
    assert_eq!(block.expressions.len(), 3);

    // First: 0.5 * Sp(i) Sm(i+1)
    let e0 = &block.expressions[0];
    assert!(matches!(e0.coeff, CoeffExpr::Literal(c) if (c - 0.5).abs() < 1e-12));
    assert_eq!(e0.operators.len(), 2);
    assert!(matches!(&e0.operators[0], OpExpr::SpinPlus(IndexExpr::Var(v)) if v == "i"));
    assert!(matches!(&e0.operators[1], OpExpr::SpinMinus(IndexExpr::VarPlus(v, 1)) if v == "i"));

    // Second: 0.5 * Sm(i) Sp(i+1)
    let e1 = &block.expressions[1];
    assert!(matches!(e1.coeff, CoeffExpr::Literal(c) if (c - 0.5).abs() < 1e-12));
    assert!(matches!(&e1.operators[0], OpExpr::SpinMinus(IndexExpr::Var(v)) if v == "i"));
    assert!(matches!(&e1.operators[1], OpExpr::SpinPlus(IndexExpr::VarPlus(v, 1)) if v == "i"));

    // Third: 1.0 * Sz(i) Sz(i+1)
    let e2 = &block.expressions[2];
    assert!(matches!(e2.coeff, CoeffExpr::Literal(c) if (c - 1.0).abs() < 1e-12));
    assert!(matches!(&e2.operators[0], OpExpr::SpinZ(IndexExpr::Var(v)) if v == "i"));
    assert!(matches!(&e2.operators[1], OpExpr::SpinZ(IndexExpr::VarPlus(v, 1)) if v == "i"));
}

#[test]
fn parse_spindot_with_coefficient() {
    let input = r#"
lattice 1d sites=4 pbc=true

sum i=0..3:
  J * S(i) . S(i+1)

params:
  J = 1.5
"#;
    let model = parse(input).unwrap();
    let block = &model.sum_blocks[0];
    assert_eq!(block.expressions.len(), 3);
    // Coefficients should include J multiplied in
    let e0 = &block.expressions[0];
    assert!(matches!(&e0.coeff, CoeffExpr::Mul(_, _)));
}
