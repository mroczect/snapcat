use clap::Parser;
use std::path::PathBuf;
use snapcat::handler::config::{SnapConfig, OutputFormat};
use snapcat::{snap, output};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser)]
#[command(name = "snapcat")]
#[command(about = "Gabungkan tree + cat + SHA256 ke dalam file JSON/Markdown", long_about = None)]
struct Cli {
    /// Direktori target
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Format output
    #[arg(short = 'f', long = "format", value_enum)]
    format: Option<OutputFormat>,

    /// File output
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// Verbose mode (tracing)
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

fn main() -> miette::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("snapcat=info"));
    fmt().with_env_filter(env_filter).init();

    let cli = Cli::parse();
    let config = SnapConfig::load(cli.format, cli.output, cli.verbose)?;

    let result = snap(&cli.path, &config)?;
    output(result, &config)?;
    Ok(())
}
