use enum_dispatch::enum_dispatch;
mod cli;

mod utils;
pub use cli::{BuildOpts, InitOpts, Opts, RunOpts, Subcommand};

pub(crate) use utils::*;

pub const BUILD_DIR: &str = ".build";
#[allow(async_fn_in_trait)]
#[enum_dispatch]
pub trait CmdExcetor {
    async fn execute(self) -> anyhow::Result<()>;
}
