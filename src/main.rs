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

    let terms = core::normal::normal_order(&ham.terms);
    eprintln!("Normal ordered: {} terms", terms.len());

    let terms = core::combine::combine(&terms);
    eprintln!("Combined: {} terms", terms.len());

    let terms = core::symmetry::filter_sz_conserving(&terms);
    eprintln!("After Sz filter: {} terms", terms.len());

    let mut final_ham = core::op::Hamiltonian::new(model.lattice.num_sites);
    for t in terms {
        final_ham.add_term(t);
    }

    output::mvmc::write_all_files(&final_ham, &cli.output)
        .unwrap_or_else(|e| {
            eprintln!("Error writing output: {}", e);
            std::process::exit(1);
        });

    eprintln!("Written mVMC files to {}", cli.output.display());
}
