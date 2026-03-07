use crate::core::op::{Op, Term};
use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub struct GreenDecomposition {
    pub two_body: Vec<Term>,
    pub one_body_corrections: Vec<Term>,
}

pub fn reorder_green_function(ops: &SmallVec<[Op; 4]>) -> GreenDecomposition {
    assert_eq!(ops.len(), 4, "Expected 4 operators");

    let mut decomp = GreenDecomposition {
        two_body: Vec::new(),
        one_body_corrections: Vec::new(),
    };

    // Already in c†cc†c form
    if ops[0].is_creation() && ops[1].is_annihilation() && ops[2].is_creation() && ops[3].is_annihilation() {
        decomp.two_body.push(Term::new(1.0, ops.clone()));
        return decomp;
    }

    // c†c†cc form: swap ops[1] and ops[2] using anticommutation
    // c†_a c†_b c_c c_d = -c†_a c_c c†_b c_d + δ_{bc} * c†_a c_d
    if ops[0].is_creation() && ops[1].is_creation() && ops[2].is_annihilation() && ops[3].is_annihilation() {
        let swapped_ops: SmallVec<[Op; 4]> = SmallVec::from_slice(&[ops[0], ops[2], ops[1], ops[3]]);
        decomp.two_body.push(Term::new(-1.0, swapped_ops));

        // Check delta: ops[1] (c†_b) and ops[2] (c_c) — delta if same site and spin
        if let (Op::FermionCreate(s1, sp1), Op::FermionAnnihilate(s2, sp2)) = (ops[1], ops[2]) {
            if s1 == s2 && sp1 == sp2 {
                let correction_ops: SmallVec<[Op; 4]> = SmallVec::from_slice(&[ops[0], ops[3]]);
                decomp.one_body_corrections.push(Term::new(1.0, correction_ops));
            }
        }

        return decomp;
    }

    // Fallback: use general normal ordering
    let term = Term::new(1.0, ops.clone());
    let reordered = crate::core::normal::normal_order(&[term]);
    for t in reordered {
        match t.ops.len() {
            4 => decomp.two_body.push(t),
            2 => decomp.one_body_corrections.push(t),
            _ => {}
        }
    }

    decomp
}
