use std::fs;

use clap::Parser;
use dino_server::{start_server, ProjectConfig, SwappableAppRouter, TenentRouter};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

use crate::{build_project, CmdExcetor};

#[derive(Debug, Parser)]

pub struct RunOpts {
    #[arg(short, long, default_value = "8080")]
    pub port: u16,
}

impl CmdExcetor for RunOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
        tracing_subscriber::registry().with(layer).init();
        let filename = build_project(".")?;
        let code = fs::read_to_string(&filename)?;
        let config = ProjectConfig::load(filename.replace(".mjs", ".yml"))?;
        let router = vec![TenentRouter::new(
            "localhost".to_string(),
            SwappableAppRouter::new(code.to_string(), config.routes)?,
        )];
        start_server(self.port, router).await?;
        // let worker = JsEngine::new(&code)?;
        // let req = Req::builder()
        //     .method("GET")
        //     .headers(HashMap::new())
        //     .url("http://localhost:8080")
        //     .build();
        // let ret = worker.run("hello", req)?;
        // println!("Response: {:?}", ret);
        Ok(())
    }
}
