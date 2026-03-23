use anyhow::Result;
use clap::Parser;

mod cli;
mod cmd;
mod template;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(log_level));

    match cli.command {
        cli::Commands::New(args) => cmd::new::run(args),
        cli::Commands::Build(args) => cmd::build::run(args),
        cli::Commands::Migrate(args) => cmd::migrate::run(args),
    }
}
