use clap::Parser;

use crate::{build_project, CmdExcetor};

#[derive(Debug, Parser)]

pub struct BuildOpts {}
impl CmdExcetor for BuildOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let filename = build_project(".")?;
        eprintln!("Build successful, output file: {}", filename);

        Ok(())
    }
}
