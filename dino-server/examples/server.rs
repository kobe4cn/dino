use anyhow::Result;
use dashmap::DashMap;
use dino_server::{ProjectConfig, SwappableAppRouter};

#[tokio::main]
async fn main() -> Result<()> {
    let config = include_str!("../fixtures/config.yml");
    let config: ProjectConfig = serde_yaml::from_str(config)?;
    let router = DashMap::new();
    router.insert(
        "localhost".to_string(),
        SwappableAppRouter::new(config.routes)?,
    );
    // let router = Arc::new(router);
    dino_server::start_server(8080, router).await?;
    Ok(())
}
