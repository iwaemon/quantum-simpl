#[derive(Debug, Clone)]
pub struct ModelDef {
    pub lattice: LatticeDef,
    pub sum_blocks: Vec<SumBlock>,
    pub params: Vec<(String, f64)>,
}

#[derive(Debug, Clone)]
pub struct LatticeDef {
    pub dimension: String,
    pub num_sites: usize,
    pub pbc: bool,
}

#[derive(Debug, Clone)]
pub struct SumBlock {
    pub var: String,
    pub range_start: usize,
    pub range_end: usize,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub coeff: CoeffExpr,
    pub operators: Vec<OpExpr>,
    pub hermitian_conjugate: bool,
}

#[derive(Debug, Clone)]
pub enum CoeffExpr {
    Literal(f64),
    Param(String),
    Neg(Box<CoeffExpr>),
    Mul(Box<CoeffExpr>, Box<CoeffExpr>),
}

#[derive(Debug, Clone)]
pub enum OpExpr {
    FermionCreate(IndexExpr, SpinExpr),
    FermionAnnihilate(IndexExpr, SpinExpr),
    Number(IndexExpr, SpinExpr),
    SpinPlus(IndexExpr),
    SpinMinus(IndexExpr),
    SpinZ(IndexExpr),
}

#[derive(Debug, Clone)]
pub enum IndexExpr {
    Var(String),
    VarPlus(String, usize),
    VarMinus(String, usize),
    Literal(usize),
}

#[derive(Debug, Clone)]
pub enum SpinExpr {
    Up,
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_definition_creation() {
        let model = ModelDef {
            lattice: LatticeDef {
                dimension: "1d".to_string(),
                num_sites: 10,
                pbc: true,
            },
            sum_blocks: vec![],
            params: vec![("t".to_string(), 1.0)],
        };
        assert_eq!(model.lattice.num_sites, 10);
        assert!(model.lattice.pbc);
    }
}
