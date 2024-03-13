use std::path::Path;

pub struct MipsCodegen;

impl MipsCodegen {
    pub fn write_to_file(&self, _p: impl AsRef<Path>) -> std::io::Result<()> {
        Ok(())
    }
}
