use smallvec::SmallVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Spin {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    FermionCreate(usize, Spin),
    FermionAnnihilate(usize, Spin),
    SpinPlus(usize),
    SpinMinus(usize),
    SpinZ(usize),
}

impl Op {
    pub fn delta_sz(&self) -> i32 {
        match self {
            Op::FermionCreate(_, Spin::Up) => 1,
            Op::FermionCreate(_, Spin::Down) => -1,
            Op::FermionAnnihilate(_, Spin::Up) => -1,
            Op::FermionAnnihilate(_, Spin::Down) => 1,
            Op::SpinPlus(_) => 1,
            Op::SpinMinus(_) => -1,
            Op::SpinZ(_) => 0,
        }
    }

    pub fn is_creation(&self) -> bool {
        matches!(self, Op::FermionCreate(_, _))
    }

    pub fn is_annihilation(&self) -> bool {
        matches!(self, Op::FermionAnnihilate(_, _))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Term {
    pub coeff: f64,
    pub ops: SmallVec<[Op; 4]>,
}

impl Term {
    pub fn new(coeff: f64, ops: SmallVec<[Op; 4]>) -> Self {
        Self { coeff, ops }
    }

    pub fn delta_sz(&self) -> i32 {
        self.ops.iter().map(|op| op.delta_sz()).sum()
    }

    pub fn hermitian_conjugate(&self) -> Self {
        let new_ops: SmallVec<[Op; 4]> = self.ops.iter().rev().map(|op| match op {
            Op::FermionCreate(s, spin) => Op::FermionAnnihilate(*s, *spin),
            Op::FermionAnnihilate(s, spin) => Op::FermionCreate(*s, *spin),
            Op::SpinPlus(s) => Op::SpinMinus(*s),
            Op::SpinMinus(s) => Op::SpinPlus(*s),
            Op::SpinZ(s) => Op::SpinZ(*s),
        }).collect();
        Self { coeff: self.coeff, ops: new_ops }
    }

}

#[derive(Debug, Clone)]
pub struct Hamiltonian {
    pub terms: Vec<Term>,
    pub num_sites: usize,
}

impl Hamiltonian {
    pub fn new(num_sites: usize) -> Self {
        Self { terms: vec![], num_sites }
    }

    pub fn add_term(&mut self, term: Term) {
        self.terms.push(term);
    }
}
