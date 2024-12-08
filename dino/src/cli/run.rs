use std::fs;

use clap::Parser;

use crate::{build_project, CmdExcetor, JsEngine};

#[derive(Debug, Parser)]

pub struct RunOpts {}

impl CmdExcetor for RunOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let filename = build_project(".")?;
        let content = fs::read_to_string(&filename)?;
        let worker = JsEngine::new(&content)?;

        worker.run("await handlers.hello()")?;

        Ok(())
    }
}
