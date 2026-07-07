use clap::Parser;
use neutrino_schema::cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Inspect(inspect) => inspect.run().await?,
    }

    Ok(())
}
