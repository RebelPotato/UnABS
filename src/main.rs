pub mod interpreter;

use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use interpreter::parse_term;

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

fn main() {
    let args = Cli::parse();

    let program = args.program.unwrap_or_else(|| {
        std::fs::read_to_string(args.file.unwrap()).expect("Could not read file")
    });

    let term = parse_term(program.as_str());
    println!("Term:\n{}\n", term);
    let state = interpreter::new(term);

    if args.interactive {
        let mut state = state;
        println!("{}", state);
        println!("Press enter to step, or Ctrl-C to exit. `r` to run to completion.");
        let result = loop {
            print!("> ");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            match input.trim() {
                "r" => break state.run(),
                _ => (),
            }
            match state.step() {
                interpreter::SEither::S(s) => state = s,
                interpreter::SEither::V(v) => break v,
            }
            println!("{}", state);
        };
        println!("-----\nResult:\n{}", result);
    } else {
        let result = state.run();
        println!("Result:\n{}", result);
    }
}
