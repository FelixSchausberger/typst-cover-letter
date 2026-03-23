use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "coverletter", version, about = "Cover letter management tool")]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new cover letter application
    New(NewArgs),
    /// Compile cover letter(s) to PDF
    Build(BuildArgs),
    /// Migrate existing files to @local/cover-letter (temporary, delete after use)
    #[command(hide = true)]
    Migrate(MigrateArgs),
}

#[derive(Parser, Debug)]
pub struct NewArgs {
    /// Company name (used for directory name)
    #[arg(long)]
    pub company: Option<String>,

    /// Job position / title
    #[arg(long)]
    pub position: Option<String>,

    /// Language: de or en
    #[arg(long, value_parser = ["de", "en"])]
    pub lang: Option<String>,

    /// Date in DD.MM.YYYY format (default: today)
    #[arg(long)]
    pub date: Option<String>,

    /// Contact person, e.g. "Dr. Max Mustermann" (optional)
    #[arg(long)]
    pub contact: Option<String>,

    /// Base directory for the new application folder (default: current dir)
    #[arg(long)]
    pub dir: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct BuildArgs {
    /// Path to a .typ file or directory containing one (default: current dir)
    pub path: Option<PathBuf>,

    /// Compile all .typ files found recursively under path
    #[arg(long)]
    pub all: bool,

    /// Recompile even if the PDF is already newer than the source
    #[arg(long)]
    pub force: bool,
}

#[derive(Parser, Debug)]
pub struct MigrateArgs {
    /// Directory to migrate recursively
    pub path: PathBuf,

    /// Preview changes without writing any files
    #[arg(long)]
    pub dry_run: bool,
}
