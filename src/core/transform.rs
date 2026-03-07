use crate::core::op::{Op, Spin, Term};
use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub enum SubstitutionRule {
    /// Particle-hole transformation for a specific spin:
    /// c†(i,s) -> c(i,s), c(i,s) -> c†(i,s)
    ParticleHole(Spin),
}

pub fn apply_substitution(terms: &[Term], rules: &[SubstitutionRule]) -> Vec<Term> {
    terms
        .iter()
        .map(|term| {
            let new_ops = term
                .ops
                .iter()
                .map(|op| {
                    let mut current = *op;
                    for rule in rules {
                        current = apply_rule(current, rule);
                    }
                    current
                })
                .collect();
            Term::new(term.coeff, new_ops)
        })
        .collect()
}

fn apply_rule(op: Op, rule: &SubstitutionRule) -> Op {
    match rule {
        SubstitutionRule::ParticleHole(spin) => match op {
            Op::FermionCreate(site, s) if s == *spin => Op::FermionAnnihilate(site, s),
            Op::FermionAnnihilate(site, s) if s == *spin => Op::FermionCreate(site, s),
            _ => op,
        },
    }
}

/// Convert all spin operators in terms to fermionic form.
/// - Sp(i) → c†(i,↑) c(i,↓)
/// - Sm(i) → c†(i,↓) c(i,↑)
/// - Sz(i) → 0.5*c†(i,↑)c(i,↑) - 0.5*c†(i,↓)c(i,↓)
///
/// Sz causes Term splitting: one Term with Sz becomes two Terms.
/// For products like Sz(i)*Sz(j), this produces 2×2=4 Terms.
pub fn spin_to_fermion(terms: &[Term]) -> Vec<Term> {
    let mut result = Vec::new();
    for term in terms {
        let mut partials: Vec<(f64, SmallVec<[Op; 4]>)> = vec![(term.coeff, SmallVec::new())];
        for op in &term.ops {
            match op {
                Op::SpinPlus(site) => {
                    for (_, ops) in &mut partials {
                        ops.push(Op::FermionCreate(*site, Spin::Up));
                        ops.push(Op::FermionAnnihilate(*site, Spin::Down));
                    }
                }
                Op::SpinMinus(site) => {
                    for (_, ops) in &mut partials {
                        ops.push(Op::FermionCreate(*site, Spin::Down));
                        ops.push(Op::FermionAnnihilate(*site, Spin::Up));
                    }
                }
                Op::SpinZ(site) => {
                    let mut new_partials = Vec::with_capacity(partials.len() * 2);
                    for (coeff, ops) in &partials {
                        let mut ops_up = ops.clone();
                        ops_up.push(Op::FermionCreate(*site, Spin::Up));
                        ops_up.push(Op::FermionAnnihilate(*site, Spin::Up));
                        new_partials.push((*coeff * 0.5, ops_up));

                        let mut ops_down = ops.clone();
                        ops_down.push(Op::FermionCreate(*site, Spin::Down));
                        ops_down.push(Op::FermionAnnihilate(*site, Spin::Down));
                        new_partials.push((*coeff * -0.5, ops_down));
                    }
                    partials = new_partials;
                }
                other => {
                    for (_, ops) in &mut partials {
                        ops.push(*other);
                    }
                }
            }
        }
        for (coeff, ops) in partials {
            result.push(Term::new(coeff, ops));
        }
    }
    result
}
