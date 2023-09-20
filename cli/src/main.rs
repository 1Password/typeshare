//! This is the command line tool for Typeshare. It is used to generate source files in other
//! languages based on Rust code.

use clap::{command, Parser, Subcommand};

use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(long, short = 'c')]
    config_file: Option<PathBuf>,
}
#[derive(Parser)]
pub struct GenerateCommandWithConfig {
    #[clap(long, short = 'c')]
    config_file: Option<PathBuf>,
}
#[derive(Subcommand)]
enum Commands {
    #[clap(
        about = "Initialize a typeshare.toml file in the current directory or the directory specified by -c"
    )]
    Init,
}
fn main() {
    let _command = Cli::parse();
}
