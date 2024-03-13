use std::env::Args;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::path::Path;

const DEFAULT_OUTNAME: &'static str = "out.asm";

#[derive(Debug)]
pub enum CliArgError {
    MissingSourceFile,
    NonexistantSourceFile,
    MissingOutArg,
}

impl Display for CliArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error: {}",
            match self {
                Self::MissingSourceFile => "Missing source code file",
                Self::NonexistantSourceFile => "Source code file doesn't exist",
                Self::MissingOutArg => "Argument to '-o' is missing",
            }
        )
    }
}

impl Error for CliArgError {}
pub type CliResult<T> = Result<T, CliArgError>;

pub struct CliArgs {
    source: String,
    out: Option<String>,
}

impl CliArgs {
    pub fn parse(args: Args) -> CliResult<Self> {
        let mut source = String::new();
        let mut out = None;
        let mut args = args.skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-o" => {
                    if let Some(outfile) = args.next() {
                        out = Some(outfile);
                    } else {
                        return Err(CliArgError::MissingOutArg);
                    }
                }
                _ => {
                    if Path::new(arg.as_str()).exists() {
                        source = arg;
                    } else {
                        return Err(CliArgError::NonexistantSourceFile);
                    }
                }
            }
        }

        if source.is_empty() {
            return Err(CliArgError::MissingSourceFile);
        }

        Ok(CliArgs { source, out })
    }

    pub fn invoke(self) -> Result<(), Box<dyn Error>> {
        let source = fs::read_to_string(self.source)?;
        let ast = deimos_parser::parse_file(source)?;
        deimos_parser::validate(&ast)?;
        let codegen = deimos_codegen::codegen(&ast);
        codegen.write_to_file(self.out.as_deref().unwrap_or(DEFAULT_OUTNAME))?;
        Ok(())
    }
}
