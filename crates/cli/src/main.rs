use clap::Parser;
use jsontolang::cli::Cli;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match jsontolang::run(&cli, &mut std::io::stdin().lock()) {
        Ok(stdout) => {
            print!("{stdout}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
