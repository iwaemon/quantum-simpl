use crate::core::op::{Op, Spin, Term};

fn format_spin(s: Spin) -> &'static str {
    match s {
        Spin::Up => "up",
        Spin::Down => "down",
    }
}

fn format_op(op: &Op) -> String {
    match op {
        Op::FermionCreate(site, spin) => format!("c†({},{})", site, format_spin(*spin)),
        Op::FermionAnnihilate(site, spin) => format!("c({},{})", site, format_spin(*spin)),
        Op::SpinPlus(site) => format!("Sp({})", site),
        Op::SpinMinus(site) => format!("Sm({})", site),
        Op::SpinZ(site) => format!("Sz({})", site),
    }
}

pub fn generate_correlation_summary(terms: &[Term]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Correlation function: {} terms\n", terms.len()));

    for term in terms {
        let sign = if term.coeff >= 0.0 { "+" } else { "" };
        let ops_str: Vec<String> = term.ops.iter().map(|op| format_op(op)).collect();
        out.push_str(&format!("  {}{} * {}\n", sign, term.coeff, ops_str.join(" ")));
    }

    out
}
