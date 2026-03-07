use crate::core::op::{Op, Spin, Term};

#[derive(Debug, Clone)]
pub struct ClassifiedTerms {
    pub constants: Vec<Term>,
    pub one_body: Vec<Term>,
    pub coulomb_intra: Vec<(usize, f64)>,
    pub two_body: Vec<Term>,
}

impl ClassifiedTerms {
    pub fn offset(&self) -> f64 {
        self.constants.iter().map(|t| t.coeff).sum()
    }
}

pub fn classify_terms(terms: &[Term]) -> ClassifiedTerms {
    let mut result = ClassifiedTerms {
        constants: Vec::new(),
        one_body: Vec::new(),
        coulomb_intra: Vec::new(),
        two_body: Vec::new(),
    };

    for term in terms {
        match term.ops.len() {
            0 => result.constants.push(term.clone()),
            2 => result.one_body.push(term.clone()),
            4 => {
                if let Some((site, coeff)) = detect_coulomb_intra(term) {
                    result.coulomb_intra.push((site, coeff));
                } else {
                    result.two_body.push(term.clone());
                }
            }
            _ => {}
        }
    }

    result
}

fn detect_coulomb_intra(term: &Term) -> Option<(usize, f64)> {
    if term.ops.len() != 4 { return None; }
    match (term.ops[0], term.ops[1], term.ops[2], term.ops[3]) {
        (Op::FermionCreate(s0, Spin::Up), Op::FermionAnnihilate(s1, Spin::Up),
         Op::FermionCreate(s2, Spin::Down), Op::FermionAnnihilate(s3, Spin::Down))
            if s0 == s1 && s1 == s2 && s2 == s3 => Some((s0, term.coeff)),
        (Op::FermionCreate(s0, Spin::Down), Op::FermionAnnihilate(s1, Spin::Down),
         Op::FermionCreate(s2, Spin::Up), Op::FermionAnnihilate(s3, Spin::Up))
            if s0 == s1 && s1 == s2 && s2 == s3 => Some((s0, term.coeff)),
        _ => None,
    }
}
