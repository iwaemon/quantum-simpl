use crate::core::op::{Op, Spin, Term, Hamiltonian};
use crate::parser::ast::*;
use smallvec::SmallVec;

pub fn expand(model: &ModelDef) -> Hamiltonian {
    let num_sites = model.lattice.num_sites;
    let pbc = model.lattice.pbc;
    let mut ham = Hamiltonian::new(num_sites);
    let mut dropped = 0usize;

    let params: std::collections::HashMap<&str, f64> = model.params.iter()
        .map(|(k, v)| (k.as_str(), *v))
        .collect();

    for block in &model.sum_blocks {
        for idx in block.range_start..=block.range_end {
            for expr in &block.expressions {
                let coeff = eval_coeff(&expr.coeff, &params);
                let ops = expand_operators(&expr.operators, &block.var, idx, num_sites, pbc);

                if let Some(ops) = ops {
                    let term = Term::new(coeff, ops);

                    if expr.hermitian_conjugate {
                        let hc = term.hermitian_conjugate();
                        ham.add_term(term);
                        ham.add_term(hc);
                    } else {
                        ham.add_term(term);
                    }
                } else if !pbc {
                    dropped += 1;
                }
            }
        }
    }

    if dropped > 0 {
        eprintln!("Warning: {} term(s) dropped due to out-of-range site indices (OBC mode)", dropped);
    }

    ham
}

fn eval_coeff(expr: &CoeffExpr, params: &std::collections::HashMap<&str, f64>) -> f64 {
    match expr {
        CoeffExpr::Literal(v) => *v,
        CoeffExpr::Param(name) => *params.get(name.as_str()).unwrap_or(&0.0),
        CoeffExpr::Neg(inner) => -eval_coeff(inner, params),
        CoeffExpr::Mul(a, b) => eval_coeff(a, params) * eval_coeff(b, params),
    }
}

fn expand_operators(
    op_exprs: &[OpExpr],
    var: &str,
    idx: usize,
    num_sites: usize,
    pbc: bool,
) -> Option<SmallVec<[Op; 4]>> {
    let mut ops: SmallVec<[Op; 4]> = SmallVec::new();

    for op_expr in op_exprs {
        match op_expr {
            OpExpr::FermionCreate(index, spin) => {
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                ops.push(Op::FermionCreate(site, resolve_spin(spin)));
            }
            OpExpr::FermionAnnihilate(index, spin) => {
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                ops.push(Op::FermionAnnihilate(site, resolve_spin(spin)));
            }
            OpExpr::Number(index, spin) => {
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                let s = resolve_spin(spin);
                ops.push(Op::FermionCreate(site, s));
                ops.push(Op::FermionAnnihilate(site, s));
            }
            OpExpr::SpinPlus(index) => {
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                ops.push(Op::SpinPlus(site));
            }
            OpExpr::SpinMinus(index) => {
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                ops.push(Op::SpinMinus(site));
            }
            OpExpr::SpinZ(index) => {
                let site = resolve_index(index, var, idx, num_sites, pbc)?;
                ops.push(Op::SpinZ(site));
            }
        }
    }

    Some(ops)
}

fn resolve_index(expr: &IndexExpr, var: &str, idx: usize, num_sites: usize, pbc: bool) -> Option<usize> {
    let raw = match expr {
        IndexExpr::Var(v) if v == var => idx,
        IndexExpr::VarPlus(v, offset) if v == var => idx + offset,
        IndexExpr::VarMinus(v, offset) if v == var => {
            if idx < *offset {
                if pbc { num_sites + idx - offset } else { return None; }
            } else {
                idx - offset
            }
        }
        IndexExpr::Literal(n) => *n,
        _ => return None,
    };

    if raw >= num_sites {
        if pbc {
            Some(raw % num_sites)
        } else {
            None
        }
    } else {
        Some(raw)
    }
}

fn resolve_spin(spin: &SpinExpr) -> Spin {
    match spin {
        SpinExpr::Up => Spin::Up,
        SpinExpr::Down => Spin::Down,
    }
}
