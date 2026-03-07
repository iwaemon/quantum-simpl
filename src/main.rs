mod core;
mod parser;
mod output;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "quantum-simpl")]
#[command(version)]
#[command(about = "Hamiltonian symbolic preprocessor for mVMC")]
struct Cli {
    /// Input DSL file
    input: PathBuf,

    /// Output directory for mVMC files
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Apply Yokoyama-Shiba transformation (particle-hole for down-spin)
    #[arg(long)]
    ys_transform: bool,
}

fn main() {
    let cli = Cli::parse();

    let input = std::fs::read_to_string(&cli.input)
        .unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", cli.input.display(), e);
            std::process::exit(1);
        });

    let model = parser::parse(&input)
        .unwrap_or_else(|e| {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        });

    eprintln!("Parsed: {} sites, {} sum blocks, {} params",
        model.lattice.num_sites, model.sum_blocks.len(), model.params.len());

    let ham = core::expand::expand(&model);
    eprintln!("Expanded: {} terms", ham.terms.len());

    let terms = if cli.ys_transform {
        eprintln!("Applying YS transformation (particle-hole for down-spin)...");
        let rules = vec![core::transform::SubstitutionRule::ParticleHole(core::op::Spin::Down)];
        core::transform::apply_substitution(&ham.terms, &rules)
    } else {
        ham.terms
    };

    let terms = core::normal::normal_order(&terms);
    eprintln!("Normal ordered: {} terms", terms.len());

    let terms = core::combine::combine(&terms);
    eprintln!("Combined: {} terms", terms.len());

    let terms = core::symmetry::filter_sz_conserving(&terms);
    eprintln!("After Sz filter: {} terms", terms.len());

    if cli.ys_transform {
        let classified = core::classify::classify_terms(&terms);
        eprintln!("Classified: {} one-body, {} coulomb_intra, {} two-body, offset={}",
            classified.one_body.len(), classified.coulomb_intra.len(),
            classified.two_body.len(), classified.offset());

        let mut one_body_ham = core::op::Hamiltonian::new(model.lattice.num_sites);
        for t in &classified.one_body {
            one_body_ham.add_term(t.clone());
        }
        let mut two_body_ham = core::op::Hamiltonian::new(model.lattice.num_sites);
        for t in &classified.two_body {
            two_body_ham.add_term(t.clone());
        }

        std::fs::create_dir_all(&cli.output).unwrap_or_else(|e| {
            eprintln!("Error creating output directory: {}", e);
            std::process::exit(1);
        });

        let namelist = if classified.coulomb_intra.is_empty() {
            output::mvmc::generate_namelist()
        } else {
            let mut nl = output::mvmc::generate_namelist();
            nl.push_str("CoulombIntra  coulombintra.def\n");
            nl
        };

        let write = |name: &str, content: String| {
            std::fs::write(cli.output.join(name), content).unwrap_or_else(|e| {
                eprintln!("Error writing {}: {}", name, e);
                std::process::exit(1);
            });
        };

        write("namelist.def", namelist);
        write("modpara.def", output::mvmc::generate_modpara_def(&one_body_ham));
        write("locspn.def", output::mvmc::generate_locspn_def(&one_body_ham));
        write("trans.def", output::mvmc::generate_trans_def(&one_body_ham));
        write("interall.def", output::mvmc::generate_interall_def(&two_body_ham));
        if !classified.coulomb_intra.is_empty() {
            write("coulombintra.def", output::mvmc::generate_coulombintra_def(&classified));
        }
        write("gutzwilleridx.def", output::mvmc::generate_gutzwilleridx_def(&one_body_ham));
        write("jastrowidx.def", output::mvmc::generate_jastrowidx_def(&one_body_ham));
        write("orbitalidx.def", output::mvmc::generate_orbitalidx_def(&one_body_ham));
        write("qptransidx.def", output::mvmc::generate_qptransidx_def(&one_body_ham));

        if classified.offset().abs() > 1e-15 {
            eprintln!("Energy offset (constant terms): {:.15}", classified.offset());
        }
    } else {
        let mut final_ham = core::op::Hamiltonian::new(model.lattice.num_sites);
        for t in terms {
            final_ham.add_term(t);
        }

        output::mvmc::write_all_files(&final_ham, &cli.output)
            .unwrap_or_else(|e| {
                eprintln!("Error writing output: {}", e);
                std::process::exit(1);
            });
    }

    eprintln!("Written mVMC files to {}", cli.output.display());
}
