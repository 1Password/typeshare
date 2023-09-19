//! This is the command line tool for Typeshare. It is used to generate source files in other
//! languages based on Rust code.

use clap::{command, Arg, ArgMatches, Command, CommandFactory, Parser, Subcommand, ValueEnum};

use std::path::PathBuf;

mod config;

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
    let mut command = Cli::parse();
}
