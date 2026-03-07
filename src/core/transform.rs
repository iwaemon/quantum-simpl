use crate::core::op::{Op, Spin, Term};

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
