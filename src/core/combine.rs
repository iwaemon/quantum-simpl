use crate::core::op::{Op, Term};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

const ZERO_THRESHOLD: f64 = 1e-15;

/// Combine terms with identical operator strings by summing their coefficients.
/// Remove terms whose coefficient is effectively zero.
pub fn combine(terms: &[Term]) -> Vec<Term> {
    let mut map: FxHashMap<SmallVec<[Op; 4]>, f64> = FxHashMap::default();

    for term in terms {
        *map.entry(term.ops.clone()).or_insert(0.0) += term.coeff;
    }

    map.into_iter()
        .filter(|(_, coeff)| coeff.abs() > ZERO_THRESHOLD)
        .map(|(ops, coeff)| Term::new(coeff, ops))
        .collect()
}
