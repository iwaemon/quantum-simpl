use crate::core::op::Term;

/// Keep only terms that conserve total Sz (i.e., ΔSz = 0).
pub fn filter_sz_conserving(terms: &[Term]) -> Vec<Term> {
    terms.iter()
        .filter(|t| t.delta_sz() == 0)
        .cloned()
        .collect()
}
