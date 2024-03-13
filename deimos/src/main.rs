mod cli;

use std::process::ExitCode;

fn main() -> ExitCode {
    let options = match cli::CliArgs::parse(std::env::args()) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };

    ExitCode::SUCCESS
}
