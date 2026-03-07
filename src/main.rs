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
    /// Input DSL file (Hamiltonian)
    input: Option<PathBuf>,

    /// Output directory for mVMC files
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Apply Yokoyama-Shiba transformation (particle-hole for down-spin)
    #[arg(long)]
    ys_transform: bool,

    /// Correlation function input file (generates cisajs/cisajscktaltdc files)
    #[arg(long)]
    correlation: Option<PathBuf>,
}

fn run_correlation_pipeline(corr_path: &std::path::Path, output_dir: &std::path::Path) {
    let input = std::fs::read_to_string(corr_path)
        .unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", corr_path.display(), e);
            std::process::exit(1);
        });

    let model = parser::parse(&input)
        .unwrap_or_else(|e| {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        });

    eprintln!("Parsed correlation: {} sites, {} sum blocks", model.lattice.num_sites, model.sum_blocks.len());

    let ham = core::expand::expand(&model);
    eprintln!("Expanded: {} terms", ham.terms.len());

    let terms = core::transform::spin_to_fermion(&ham.terms);
    eprintln!("After spin→fermion: {} terms", terms.len());

    let terms = core::normal::normal_order(&terms);
    eprintln!("Normal ordered: {} terms", terms.len());

    let terms = core::combine::combine(&terms);
    eprintln!("Combined: {} terms", terms.len());

    // Green's function reorder: split into one-body and two-body
    let mut one_body_terms: Vec<core::op::Term> = Vec::new();
    let mut two_body_terms: Vec<core::op::Term> = Vec::new();

    for term in &terms {
        match term.ops.len() {
            2 => one_body_terms.push(term.clone()),
            4 => {
                let decomp = core::green::reorder_green_function(&term.ops);
                for mut t in decomp.two_body {
                    t.coeff *= term.coeff;
                    two_body_terms.push(t);
                }
                for mut t in decomp.one_body_corrections {
                    t.coeff *= term.coeff;
                    one_body_terms.push(t);
                }
            }
            _ => {}
        }
    }

    eprintln!("Green reordered: {} one-body, {} two-body", one_body_terms.len(), two_body_terms.len());

    std::fs::create_dir_all(output_dir).unwrap_or_else(|e| {
        eprintln!("Error creating output directory: {}", e);
        std::process::exit(1);
    });

    let write = |name: &str, content: String| {
        std::fs::write(output_dir.join(name), content).unwrap_or_else(|e| {
            eprintln!("Error writing {}: {}", name, e);
            std::process::exit(1);
        });
    };

    let mut all_terms = Vec::new();
    all_terms.extend(one_body_terms.iter().cloned());
    all_terms.extend(two_body_terms.iter().cloned());

    write("cisajs.def", output::mvmc::generate_cisajs_def(&one_body_terms));
    write("cisajscktaltdc.def", output::mvmc::generate_cisajscktaltdc_def(&two_body_terms));
    write("correlation_summary.txt", output::correlation::generate_correlation_summary(&all_terms));

    eprintln!("Written correlation files to {}", output_dir.display());
}

fn main() {
    let cli = Cli::parse();

    if let Some(ref corr_path) = cli.correlation {
        run_correlation_pipeline(corr_path, &cli.output);
        if cli.input.is_none() {
            return;
        }
    }

    let input_path = cli.input.as_ref().unwrap_or_else(|| {
        eprintln!("Error: either <INPUT> or --correlation is required");
        std::process::exit(1);
    });

    let input = std::fs::read_to_string(input_path)
        .unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", input_path.display(), e);
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
