use clap::Parser;
use std::path::PathBuf;
use snapcat::handler::config::{SnapConfig, OutputFormat};
use snapcat::{snap, output};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser)]
#[command(name = "snapcat")]
#[command(about = "Combine tree + cat + SHA256 into JSON or Markdown", long_about = None)]
struct Cli {
    /// Target directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format
    #[arg(short = 'f', long = "format", value_enum)]
    format: Option<OutputFormat>,

    /// Output file (stdout if not set)
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// Verbose mode (enable trace logging)
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Maximum directory depth
    #[arg(long = "max-depth")]
    max_depth: Option<usize>,

    /// Include hidden files and directories
    #[arg(long = "include-hidden")]
    include_hidden: bool,

    /// Follow symbolic links (dangerous, may loop)
    #[arg(long = "follow-symlinks")]
    follow_symlinks: bool,

    /// Number of parallel threads (default: number of CPUs)
    #[arg(short = 'j', long = "jobs")]
    jobs: Option<usize>,

    /// Maximum file size to read (in bytes)
    #[arg(long = "max-file-size")]
    max_file_size: Option<u64>,
}

fn main() -> miette::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("snapcat=info"));
    fmt().with_env_filter(env_filter).init();

    let cli = Cli::parse();
    let config = SnapConfig::load(
        cli.format,
        cli.output,
        cli.verbose,
        cli.max_depth,
        cli.include_hidden,
        cli.follow_symlinks,
        cli.jobs,
        cli.max_file_size,
    )?;

    let result = snap(&cli.path, &config)?;
    output(result, &config)?;
    Ok(())
}
