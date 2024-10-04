//! Command line argument parsing.
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

// pub const ARG_TYPE: &str = "TYPE";
// pub const ARG_SWIFT_PREFIX: &str = "SWIFTPREFIX";
// pub const ARG_KOTLIN_PREFIX: &str = "KOTLINPREFIX";
// pub const ARG_JAVA_PACKAGE: &str = "JAVAPACKAGE";
// pub const ARG_MODULE_NAME: &str = "MODULENAME";
// pub const ARG_SCALA_PACKAGE: &str = "SCALAPACKAGE";
// pub const ARG_SCALA_MODULE_NAME: &str = "SCALAMODULENAME";
// #[cfg(feature = "go")]
// pub const ARG_GO_PACKAGE: &str = "GOPACKAGE";
// pub const ARG_CONFIG_FILE_NAME: &str = "CONFIGFILENAME";
// pub const ARG_GENERATE_CONFIG: &str = "generate-config-file";
// pub const ARG_OUTPUT_FILE: &str = "output-file";
// pub const ARG_OUTPUT_FOLDER: &str = "output-folder";
// pub const ARG_FOLLOW_LINKS: &str = "follow-links";
// pub const ARG_TARGET_OS: &str = "target_os";

// #[cfg(feature = "go")]
// const AVAILABLE_LANGUAGES: [&str; 5] = ["kotlin", "scala", "swift", "typescript", "go"];

// #[cfg(not(feature = "go"))]
// const AVAILABLE_LANGUAGES: [&str; 4] = ["kotlin", "scala", "swift", "typescript"];

#[derive(Debug, Clone, Copy, ValueEnum)]
#[non_exhaustive]
pub enum AvailableLanguage {
    Kotlin,
    Scala,
    Swift,
    Typescript,
    #[cfg(feature = "go")]
    Go,
}

#[derive(Parser)]
#[command(args_conflicts_with_subcommands = true, subcommand_negates_reqs = true)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Option<Command>,

    /// Language of generated types
    #[arg(short, long = "lang")]
    pub language: AvailableLanguage,

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

    /// Generates a configuration file based on the other options specified.
    /// The file will be written to typeshare.toml by default or to the file
    /// path specified by the --config-file option.
    #[arg(short, long)]
    pub generate_config: bool,

    /// File to write output to. mtime will be preserved if the file contents don't change
    #[arg(short, long)]
    pub output_file: Option<PathBuf>,

    /// Folder to write output to. mtime will be preserved if the file contents don't change
    #[arg(short = 'd', long)]
    pub output_folder: Option<PathBuf>,

    /// Follow symbolic links to directories instead of ignoring them.
    #[arg(short = 'L', long)]
    pub follow_links: bool,

    /// Directories within which to recursively find and process rust files
    #[arg(num_args = 1..)]
    pub directories: Vec<PathBuf>,

    /// Optional restrict to target_os
    #[arg(short, long, num_args = 1..)]
    pub target_os: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Subcommand)]
pub enum Command {
    /// Generate shell completions
    Completions {
        /// The shell to generate the completions for
        shell: clap_complete::Shell,
    },
}

// /// Parse command line arguments.
// pub(crate) fn build_command() -> Command<'static> {
//     command!("typeshare")
//         .version(VERSION)
//         .args_conflicts_with_subcommands(true)
//         .subcommand_negates_reqs(true)
//         .subcommand(
//             Command::new("completions")
//                 .about("Generate shell completions")
//                 .arg(
//                     Arg::new("shell")
//                         .value_name("SHELL")
//                         .help("The shell to generate the completions for")
//                         .required(true)
//                         .possible_values(clap_complete_command::Shell::possible_values()),
//                 ),
//         )
//         .arg(
//             Arg::new(ARG_TYPE)
//                 .short('l')
//                 .long("lang")
//                 .help("Language of generated types")
//                 .takes_value(true)
//                 .possible_values(AVAILABLE_LANGUAGES)
//                 .required_unless(ARG_GENERATE_CONFIG),
//         )
//         .arg(
//             Arg::new(ARG_SWIFT_PREFIX)
//                 .short('s')
//                 .long("swift-prefix")
//                 .help("Prefix for generated Swift types")
//                 .takes_value(true)
//                 .required(false),
//         )
//         .arg(
//             Arg::new(ARG_KOTLIN_PREFIX)
//                 .short('k')
//                 .long("kotlin-prefix")
//                 .help("Prefix for generated Kotlin types")
//                 .takes_value(true)
//                 .required(false),
//         )
//         .arg(
//             Arg::new(ARG_JAVA_PACKAGE)
//                 .short('j')
//                 .long("java-package")
//                 .help("JAVA package name")
//                 .takes_value(true)
//                 .required(false),
//         )
//         .arg(
//             Arg::new(ARG_MODULE_NAME)
//                 .short('m')
//                 .long("module-name")
//                 .help("Kotlin serializer module name")
//                 .takes_value(true)
//                 .required(false),
//         )
//         .arg(
//             Arg::new(ARG_SCALA_PACKAGE)
//                 .long("scala-package")
//                 .help("Scala package name")
//                 .takes_value(true)
//                 .required(false),
//         )
//         .arg(
//             Arg::new(ARG_SCALA_MODULE_NAME)
//                 .long("scala-module-name")
//                 .help("Scala serializer module name")
//                 .takes_value(true)
//                 .required(false),
//         )

//         .arg(
//             Arg::new(ARG_CONFIG_FILE_NAME)
//                 .short('c')
//                 .long("config-file")
//                 .help("Configuration file for typeshare")
//                 .takes_value(true)
//                 .required(false),
//         )
//         .arg(
//             Arg::new(ARG_GENERATE_CONFIG)
//                 .short('g')
//                 .long("generate-config-file")
//                 .help("Generates a configuration file based on the other options specified. The file will be written to typeshare.toml by default or to the file path specified by the --config-file option.")
//                 .takes_value(false)
//                 .required(false),
//         )
//         .arg(
//             Arg::new(ARG_OUTPUT_FILE)
//                 .short('o')
//                 .long("output-file")
//                 .help("File to write output to. mtime will be preserved if the file contents don't change")
//                 .required_unless_present_any([ARG_GENERATE_CONFIG, ARG_OUTPUT_FOLDER])
//                 .takes_value(true)
//                 .long(ARG_OUTPUT_FILE)
//                 .conflicts_with(ARG_OUTPUT_FOLDER)
//         )
//         .arg(
//             Arg::new(ARG_OUTPUT_FOLDER)
//                 .short('d')
//                 .long("output-folder")
//                 .help("Folder to write output to. mtime will be preserved if the file contents don't change")
//                 .required_unless_present_any([ARG_GENERATE_CONFIG, ARG_OUTPUT_FILE])
//                 .takes_value(true)
//                 .long(ARG_OUTPUT_FOLDER)
//                 .conflicts_with(ARG_OUTPUT_FILE)
//         )
//         .group(ArgGroup::new("output").args(&["output-file", "output-folder"]))
//         .arg(
//             Arg::new(ARG_FOLLOW_LINKS)
//             .short('L')
//             .long("follow-links")
//             .help("Follow symbolic links to directories instead of ignoring them.")
//             .takes_value(false)
//             .required(false)
//         )
//         .arg(
//             Arg::new("directories")
//                 .help("Directories within which to recursively find and process rust files")
//                 .required_unless(ARG_GENERATE_CONFIG)
//                 .min_values(1),
//         ).arg(
//             Arg::new(ARG_TARGET_OS)
//                 .short('t')
//                 .long("target-os")
//                 .help("Optional restrict to target_os")
//                 .takes_value(true)
//                 .multiple_values(true)
//                 .required(false)
//         )
// }
