use clap::Parser;
use dino_server::{start_server, ProjectConfig, SwappableAppRouter, TenentRouter};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::{fs, path::Path, time::Duration};
use tokio::sync::mpsc::channel;
use tokio_stream::{wrappers::ReceiverStream, StreamExt as _};

use tracing::{info, level_filters::LevelFilter, warn};
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
        let router = SwappableAppRouter::new(code.to_string(), config.routes)?;
        let routers = vec![TenentRouter::new("localhost".to_string(), router.clone())];
        tokio::spawn(async_watch(".", router));
        start_server(self.port, routers).await?;

        Ok(())
    }
}

async fn async_watch(path: impl AsRef<Path>, router: SwappableAppRouter) -> anyhow::Result<()> {
    let (tx, rx) = channel(1);
    let mut debouncer = new_debouncer(Duration::from_secs(1), move |res: DebounceEventResult| {
        if let Err(e) = tx.blocking_send(res) {
            warn!("Fail to send event:{}", e);
        }
    })
    .unwrap();
    debouncer
        .watcher()
        .watch(path.as_ref(), RecursiveMode::Recursive)
        .unwrap();

    let mut stream = ReceiverStream::new(rx);
    while let Some(ret) = stream.next().await {
        match ret {
            Ok(events) => {
                let mut need_swap = false;

                for event in events {
                    let path = event.path;
                    // info!("File changed:{}", path.display());
                    // info!("{}", path.to_string_lossy().ends_with(".ts"));
                    if path.to_string_lossy().ends_with(".yml")
                        || path.to_string_lossy().ends_with(".ts")
                        || path.to_string_lossy().ends_with(".js")
                    {
                        info!("File changed:{}", path.display());
                        need_swap = true;
                        break;
                    }
                }
                if need_swap {
                    let filename = build_project(".")?;
                    let config = filename.replace(".mjs", ".yml");
                    let code = fs::read_to_string(&filename)?;
                    let config = ProjectConfig::load(config)?;
                    router.swap(code, config.routes)?;
                }
            }
            Err(e) => {
                warn!("Fail to get event:{}", e);
            }
        }
    }
    Ok(())
}
