use std::env::Args;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::path::Path;

const DEFAULT_OUTNAME: &'static str = "out.asm";
static LIB_SOURCE: &'static str = include_str!("lib.dei");

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

pub enum DebugStage {
    Lex,
    Parse,
}

pub struct CliArgs {
    source: String,
    out: Option<String>,
    debug_stage: Option<DebugStage>,
}

impl CliArgs {
    pub fn parse(args: Args) -> CliResult<Self> {
        let mut source = String::new();
        let mut out = None;
        let mut args = args.skip(1);
        let mut debug_stage = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-o" => {
                    if let Some(outfile) = args.next() {
                        out = Some(outfile);
                    } else {
                        return Err(CliArgError::MissingOutArg);
                    }
                }
                "-debug-stage=lex" => {
                    debug_stage = Some(DebugStage::Lex);
                }
                "-debug-stage=parse" => {
                    debug_stage = Some(DebugStage::Parse);
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

        Ok(CliArgs {
            source,
            out,
            debug_stage,
        })
    }

    pub fn invoke(self) -> Result<(), Box<dyn Error>> {
        let mut source = fs::read_to_string(self.source)?;
        source.push_str(&LIB_SOURCE);

        let tokens = deimos_parser::lex(&source)?;
        if let Some(DebugStage::Lex) = self.debug_stage {
            println!("{:?}", tokens);
            return Ok(());
        }

        let ast = deimos_parser::parse(tokens)?;
        if let Some(DebugStage::Parse) = self.debug_stage {
            println!("{:?}", ast);
            return Ok(());
        }

        let codegen = deimos_codegen::codegen(&ast)?;
        std::fs::write(self.out.as_deref().unwrap_or(DEFAULT_OUTNAME), codegen)?;

        Ok(())
    }
}
