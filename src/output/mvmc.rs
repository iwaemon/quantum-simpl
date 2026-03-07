use crate::core::classify::ClassifiedTerms;
use crate::core::op::{Op, Spin, Hamiltonian, Term};
use std::path::Path;
use std::fs;

fn spin_to_idx(s: Spin) -> usize {
    match s {
        Spin::Up => 0,
        Spin::Down => 1,
    }
}

fn classify_terms(ham: &Hamiltonian) -> (Vec<&Term>, Vec<&Term>) {
    let mut one_body = Vec::new();
    let mut two_body = Vec::new();

    for term in &ham.terms {
        match term.ops.len() {
            2 => one_body.push(term),
            4 => two_body.push(term),
            _ => {}
        }
    }

    (one_body, two_body)
}

pub fn generate_trans_def(ham: &Hamiltonian) -> String {
    let (one_body, _) = classify_terms(ham);
    let mut out = String::new();

    out.push_str("======================== \n");
    out.push_str(&format!("NTransfer      {}  \n", one_body.len()));
    out.push_str("======================== \n");
    out.push_str("========i_j_s_tijs====== \n");
    out.push_str("======================== \n");

    for term in &one_body {
        if let (Op::FermionCreate(i, si), Op::FermionAnnihilate(j, sj)) = (term.ops[0], term.ops[1]) {
            out.push_str(&format!("    {}     {}     {}     {}         {:.15}         {:.15}\n",
                i, spin_to_idx(si), j, spin_to_idx(sj), term.coeff, 0.0));
        }
    }

    out
}

pub fn generate_interall_def(ham: &Hamiltonian) -> String {
    let (_, two_body) = classify_terms(ham);
    let mut out = String::new();

    out.push_str("========================\n");
    out.push_str(&format!("TotalNumber {}\n", two_body.len()));
    out.push_str("Comment: interall\n");
    out.push_str("========================\n");
    out.push_str("========================\n");

    for term in &two_body {
        match (term.ops[0], term.ops[1], term.ops[2], term.ops[3]) {
            // Pattern: c†_i c_j c†_k c_l (mVMC native format)
            (Op::FermionCreate(i, si), Op::FermionAnnihilate(j, sj),
             Op::FermionCreate(k, sk), Op::FermionAnnihilate(l, sl)) => {
                out.push_str(&format!("{} {} {} {} {} {} {} {} {:.1} {:.1} \n",
                    i, spin_to_idx(si), j, spin_to_idx(sj),
                    k, spin_to_idx(sk), l, spin_to_idx(sl),
                    term.coeff, 0.0));
            }
            // Pattern: c†_a c†_b c_c c_d (normal-ordered form)
            // Convert: c†_a c†_b c_c c_d = -c†_a c_c c†_b c_d
            (Op::FermionCreate(a, sa), Op::FermionCreate(b, sb),
             Op::FermionAnnihilate(c, sc), Op::FermionAnnihilate(d, sd)) => {
                out.push_str(&format!("{} {} {} {} {} {} {} {} {:.1} {:.1} \n",
                    a, spin_to_idx(sa), c, spin_to_idx(sc),
                    b, spin_to_idx(sb), d, spin_to_idx(sd),
                    -term.coeff, 0.0));
            }
            _ => {}
        }
    }

    out
}

pub fn generate_locspn_def(ham: &Hamiltonian) -> String {
    let mut out = String::new();
    out.push_str("================================ \n");
    out.push_str(&format!("NlocalSpin     {}  \n", 0));
    out.push_str("================================ \n");
    out.push_str("========i_0LocSpn_1IteElc ====== \n");
    out.push_str("================================ \n");
    for i in 0..ham.num_sites {
        out.push_str(&format!("    {}      0\n", i));
    }
    out
}

pub fn generate_modpara_def(ham: &Hamiltonian) -> String {
    let nsite = ham.num_sites;
    let ncond = nsite;
    format!(
"--------------------
Model_Parameters   0
--------------------
VMC_Cal_Parameters
--------------------
CDataFileHead  zvo
CParaFileHead  zqp
--------------------
NVMCCalMode    0
--------------------
NDataIdxStart  1
NDataQtySmp    1
--------------------
Nsite          {nsite}
Ncond          {ncond}
2Sz            0
NSPGaussLeg    1
NSPStot        0
NMPTrans       1
NSROptItrStep  1000
NSROptItrSmp   100
DSROptRedCut   1e-10
DSROptStaDel   0.001
DSROptStepDt   0.003
NVMCWarmUp     10
NVMCInterval   1
NVMCSample     100
NExUpdatePath  0
NSplitSize     1
NStore         1
NSRCG          0
RndSeed  12345
")
}

pub fn generate_namelist() -> String {
    "ModPara  modpara.def\n\
LocSpin  locspn.def\n\
Trans  trans.def\n\
InterAll  interall.def\n\
Gutzwiller  gutzwilleridx.def\n\
Jastrow  jastrowidx.def\n\
Orbital  orbitalidx.def\n\
TransSym  qptransidx.def\n"
        .to_string()
}

pub fn generate_gutzwilleridx_def(ham: &Hamiltonian) -> String {
    let mut out = String::new();
    out.push_str("=============================================\n");
    out.push_str("NGutzwillerIdx          1\n");
    out.push_str("ComplexType          0\n");
    out.push_str("=============================================\n");
    out.push_str("=============================================\n");
    for i in 0..ham.num_sites {
        out.push_str(&format!("    {}      0\n", i));
    }
    out.push_str("    0      1\n");
    out
}

pub fn generate_jastrowidx_def(ham: &Hamiltonian) -> String {
    let n = ham.num_sites;
    let mut out = String::new();
    out.push_str("=============================================\n");
    out.push_str(&format!("NJastrowIdx          {}\n", n / 2));
    out.push_str("ComplexType          0\n");
    out.push_str("=============================================\n");
    out.push_str("=============================================\n");
    for i in 0..n {
        for j in 0..n {
            if i != j {
                let dist = if i > j { i - j } else { j - i };
                let idx = dist.min(n - dist) - 1;
                out.push_str(&format!("    {}      {}      {}\n", i, j, idx));
            }
        }
    }
    for i in 0..(n / 2) {
        out.push_str(&format!("    {}      1\n", i));
    }
    out
}

pub fn generate_orbitalidx_def(ham: &Hamiltonian) -> String {
    let n = ham.num_sites;
    let mut out = String::new();
    out.push_str("=============================================\n");
    out.push_str(&format!("NOrbitalIdx         {}\n", n));
    out.push_str("ComplexType          0\n");
    out.push_str("=============================================\n");
    out.push_str("=============================================\n");
    for i in 0..n {
        for j in 0..n {
            let idx = (j + n - i) % n;
            out.push_str(&format!("    {}      {}      {}\n", i, j, idx));
        }
    }
    for i in 0..n {
        out.push_str(&format!("    {}      1\n", i));
    }
    out
}

pub fn generate_qptransidx_def(ham: &Hamiltonian) -> String {
    let n = ham.num_sites;
    let mut out = String::new();
    out.push_str("=============================================\n");
    out.push_str("NQPTrans          1\n");
    out.push_str("=============================================\n");
    out.push_str("======== TrIdx_TrWeight_and_TrIdx_i_xi ======\n");
    out.push_str("=============================================\n");
    out.push_str("0    1.00000\n");
    for i in 0..n {
        out.push_str(&format!("    0      {}      {}\n", i, i));
    }
    out
}

pub fn generate_coulombintra_def(classified: &ClassifiedTerms) -> String {
    let mut out = String::new();
    out.push_str("====== \n");
    out.push_str(&format!("N {} \n", classified.coulomb_intra.len()));
    out.push_str("====== \n");
    out.push_str("====== \n");
    out.push_str("====== \n");

    for (site, coeff) in &classified.coulomb_intra {
        out.push_str(&format!("{} {:.15} \n", site, coeff));
    }

    out
}

pub fn write_all_files(ham: &Hamiltonian, output_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(output_dir)?;

    fs::write(output_dir.join("namelist.def"), generate_namelist())?;
    fs::write(output_dir.join("modpara.def"), generate_modpara_def(ham))?;
    fs::write(output_dir.join("locspn.def"), generate_locspn_def(ham))?;
    fs::write(output_dir.join("trans.def"), generate_trans_def(ham))?;
    fs::write(output_dir.join("interall.def"), generate_interall_def(ham))?;
    fs::write(output_dir.join("gutzwilleridx.def"), generate_gutzwilleridx_def(ham))?;
    fs::write(output_dir.join("jastrowidx.def"), generate_jastrowidx_def(ham))?;
    fs::write(output_dir.join("orbitalidx.def"), generate_orbitalidx_def(ham))?;
    fs::write(output_dir.join("qptransidx.def"), generate_qptransidx_def(ham))?;

    Ok(())
}
