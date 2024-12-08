mod build;
mod init;
mod run;
pub use build::BuildOpts;
use clap::{command, Parser};
use enum_dispatch::enum_dispatch;
pub use init::InitOpts;
pub use run::RunOpts;

#[derive(Debug, Parser)]
#[command(name = "dino", version, author, about,long_about=None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: Subcommand,
}

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExcetor)]
pub enum Subcommand {
    #[command(name = "init", about = "Initialize a new project")]
    Init(InitOpts),
    #[command(name = "build", about = "Build user's project")]
    Build(BuildOpts),
    #[command(name = "run", about = "Run user's project")]
    Run(RunOpts),
}
