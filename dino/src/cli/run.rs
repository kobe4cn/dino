use std::{collections::HashMap, fs};

use clap::Parser;

use crate::{build_project, CmdExcetor, JsEngine, Req};

#[derive(Debug, Parser)]

pub struct RunOpts {}

impl CmdExcetor for RunOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let filename = build_project(".")?;
        let content = fs::read_to_string(&filename)?;
        let worker = JsEngine::new(&content)?;
        let req = Req::builder()
            .method("GET")
            .headers(HashMap::new())
            .url("http://localhost:8080")
            .build();
        let ret = worker.run("hello", req)?;
        println!("Response: {:?}", ret);
        Ok(())
    }
}
