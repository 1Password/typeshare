//! Command line argument parsing.
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[non_exhaustive]
pub enum AvailableLanguage {
    Kotlin,
    Scala,
    Swift,
    Typescript,
    #[cfg(feature = "go")]
    Go,
}

#[derive(clap::Parser)]
#[command(
    version,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
    name = "typeshare"
)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Option<Command>,

    /// Language of generated types
    #[arg(short, long = "lang", required_unless_present = "generate_config")]
    pub language: Option<AvailableLanguage>,

    /// Prefix for generated Swift types
    #[arg(short, long)]
    pub swift_prefix: Option<String>,

    /// Prefix for generated Kotlin types
    #[arg(short, long)]
    pub kotlin_prefix: Option<String>,

    /// JAVA package name
    #[arg(short, long)]
    pub java_package: Option<String>,

    /// Kotlin serializer module name
    #[arg(short = 'm', long = "module-name")]
    pub kotlin_module_name: Option<String>,

    /// Scala package name
    #[arg(long)]
    pub scala_package: Option<String>,

    /// Scala serializer module name
    #[arg(long)]
    pub scala_module_name: Option<String>,

    #[cfg(feature = "go")]
    /// Go package name
    #[arg(long)]
    pub go_package: Option<String>,

    /// Configuration file for typeshare
    #[arg(short, long)]
    pub config_file: Option<PathBuf>,

    #[command(flatten)]
    pub output: Output,

    /// Follow symbolic links to directories instead of ignoring them.
    #[arg(short = 'L', long)]
    pub follow_links: bool,

    /// Directories within which to recursively find and process rust files
    #[arg(required=true, num_args = 1..)]
    pub directories: Vec<PathBuf>,

    /// Optional restrict to target_os
    #[arg(short, long, num_args = 1..)]
    pub target_os: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, clap::Subcommand)]
pub enum Command {
    /// Generate shell completions
    Completions {
        /// The shell to generate the completions for
        shell: clap_complete::Shell,
    },
}

#[derive(clap::Args, Debug)]
#[group(multiple = false, required = true)]
pub struct Output {
    /// File to write output to. mtime will be preserved if the file contents
    /// don't change
    #[arg(short = 'o', long = "output-file")]
    pub file: Option<PathBuf>,

    /// Folder to write output to. mtime will be preserved if the file contents
    /// don't change
    #[arg(short = 'd', long = "output-folder")]
    pub folder: Option<PathBuf>,

    // If given, we're going to output a new template configuration file
    // instead of running typeshare normally, so we make it mutually exclusive
    // with running normally
    /// Generates a configuration file based on the other options specified.
    /// The file will be written to typeshare.toml by default or to the file
    /// path specified by the --config-file option.
    #[arg(short, long)]
    pub generate_config: bool,
}
