pub mod machines;
pub mod term;

use std::path::PathBuf;

use clap::Parser;
use term::parse_term;

/// UnABS: Unlambda At Breakneck Speed
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Runs an unlambda program from command line
    #[arg(group = "input")]
    program: Option<String>,

    /// Runs an unlambda  program from file
    #[arg(short, long, group = "input")]
    file: Option<PathBuf>,

    /// Enter interactive mode
    #[arg(short, long)]
    interactive: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let program = args.program.unwrap_or_else(|| {
        std::fs::read_to_string(args.file.unwrap()).expect("Could not read file")
    });

    let term = parse_term(program.trim())?;
    // println!("Term:\n{}\n", term);
    machines::anaive::main(term, args.interactive);
    Ok(())
}
