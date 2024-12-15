use anyhow::Result;
use dino_server::{ProjectConfig, SwappableAppRouter, TenentRouter};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};
#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = include_str!("../fixtures/config.yml");
    let config: ProjectConfig = serde_yaml::from_str(config)?;

    let code = r#"
            (function(){
                async function hello(req){
                print("hello world");
                return{
                status:200,
                headers:{

                "content-type":"application/json"
                },
                body:JSON.stringify(req),
                };
                }
                return{hello:yang};})();
        "#;
    // router.insert(
    //     "localhost".to_string(),
    //     SwappableAppRouter::new(code.to_string(), config.routes)?,
    // );

    let router = vec![TenentRouter::new(
        "localhost".to_string(),
        SwappableAppRouter::new(code.to_string(), config.routes)?,
    )];

    // let router = Arc::new(router);
    dino_server::start_server(8080, router).await?;
    Ok(())
}
