use crate::core::op::{Op, Term};
use smallvec::SmallVec;

/// Normal-order all terms: move all c† to the left of c,
/// applying anticommutation relations {c_i, c†_j} = δ_{ij}.
/// For spin operators on the same site, apply [S+, S-] = 2Sz.
pub fn normal_order(terms: &[Term]) -> Vec<Term> {
    let mut result = Vec::new();
    for term in terms {
        let mut expanded = vec![term.clone()];
        let mut all_done = false;

        while !all_done {
            all_done = true;
            let mut next_expanded = Vec::new();

            for t in &expanded {
                if let Some(swap_pos) = find_swap_position(&t.ops) {
                    all_done = false;
                    let swapped = apply_swap(t, swap_pos);
                    next_expanded.extend(swapped);
                } else {
                    next_expanded.push(t.clone());
                }
            }

            expanded = next_expanded;
        }

        result.extend(expanded);
    }
    result
}

/// Find the leftmost position where a non-creation op appears before a creation op
/// (for fermions) or where S- appears before S+ on the same site (for spins).
fn find_swap_position(ops: &SmallVec<[Op; 4]>) -> Option<usize> {
    for i in 0..ops.len().saturating_sub(1) {
        let left = &ops[i];
        let right = &ops[i + 1];

        if left.is_annihilation() && right.is_creation() {
            return Some(i);
        }

        if let (Op::SpinMinus(s1), Op::SpinPlus(s2)) = (left, right) {
            if s1 == s2 {
                return Some(i);
            }
        }
    }
    None
}

/// Apply anticommutation/commutation at position pos, producing new terms.
fn apply_swap(term: &Term, pos: usize) -> Vec<Term> {
    let left = &term.ops[pos];
    let right = &term.ops[pos + 1];

    match (left, right) {
        // Fermion: c_a c†_b = -c†_b c_a + δ_{ab}
        (Op::FermionAnnihilate(s1, sp1), Op::FermionCreate(s2, sp2)) => {
            let mut swapped_ops = term.ops.clone();
            swapped_ops[pos] = *right;
            swapped_ops[pos + 1] = *left;
            let swapped = Term::new(-term.coeff, swapped_ops);

            let mut result = vec![swapped];

            if s1 == s2 && sp1 == sp2 {
                let mut delta_ops: SmallVec<[Op; 4]> = SmallVec::new();
                for (j, op) in term.ops.iter().enumerate() {
                    if j != pos && j != pos + 1 {
                        delta_ops.push(*op);
                    }
                }
                result.push(Term::new(term.coeff, delta_ops));
            }

            result
        }

        // Spin: S-(i) S+(i) = S+(i) S-(i) - 2Sz(i)
        (Op::SpinMinus(s1), Op::SpinPlus(s2)) if s1 == s2 => {
            let mut swapped_ops = term.ops.clone();
            swapped_ops[pos] = *right;
            swapped_ops[pos + 1] = *left;
            let swapped = Term::new(term.coeff, swapped_ops);

            let mut sz_ops: SmallVec<[Op; 4]> = SmallVec::new();
            for (j, op) in term.ops.iter().enumerate() {
                if j == pos {
                    sz_ops.push(Op::SpinZ(*s1));
                } else if j == pos + 1 {
                    // skip — replaced by single Sz above
                } else {
                    sz_ops.push(*op);
                }
            }
            let sz_term = Term::new(-2.0 * term.coeff, sz_ops);

            vec![swapped, sz_term]
        }

        _ => vec![term.clone()],
    }
}
