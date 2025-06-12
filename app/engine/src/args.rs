use std::path::{Path, PathBuf};

use clap::builder::PossibleValuesParser;

use crate::serde::args::{ArgType, CliArgsSet};

#[derive(Debug, Clone, Copy)]
pub enum OutputLocation<'a> {
    File(&'a Path),
    Folder(&'a Path),
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
    pub directory: Option<PathBuf>,
}

impl Output {
    pub fn location(&self) -> OutputLocation<'_> {
        match (&self.directory, &self.file) {
            (Some(dir), None) => OutputLocation::Folder(dir),
            (None, Some(file)) => OutputLocation::File(file),
            (None, None) => panic!("got neither a file nor a directory; clap should prevent this"),
            (Some(dir), Some(file)) => {
                panic!("got both file '{file:?}' and directory '{dir:?}'; clap should prevent this")
            }
        }
    }
}

#[derive(clap::Parser, Debug)]
#[command(args_conflicts_with_subcommands = true, subcommand_negates_reqs = true)]
pub struct StandardArgs {
    #[command(subcommand)]
    pub subcommand: Option<Command>,

    /// Path to the config file for this typeshare
    #[arg(short, long, visible_alias("config-file"))]
    pub config: Option<PathBuf>,

    /// The directories within which to recursively find and process rust files
    #[arg(num_args(1..), required=true)]
    pub directories: Vec<PathBuf>,

    #[arg(long, exclusive(true))]
    pub completions: Option<String>,

    #[command(flatten)]
    pub output: Output,

    /// If given, only fields / types / variants matching at least one of these
    /// OSes (per `cfg(target_os)`) will be emitted. If any `--target-os`
    /// arguments are passed, they will override ALL target OSes passed via
    /// a config file.
    ///
    /// Generally, typeshare will err on the side of generating things. For
    /// instance, given `--target-os=ios` and `cfg(any(target_os="android", test))`,
    /// it WILL generate a type, because that type does exist on iOS in test
    /// mode: there exists a configuration where that type exists on iOS.
    ///
    /// In the future typeshare may be able to consider other cfgs.
    #[arg(long, num_args=1..)]
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

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PersonalizeClap {
    name: Option<&'static str>,
    version: Option<&'static str>,
    author: Option<&'static str>,
    about: Option<&'static str>,
}

impl PersonalizeClap {
    pub const fn new() -> Self {
        Self {
            name: None,
            version: None,
            author: None,
            about: None,
        }
    }

    pub const fn name(self, name: &'static str) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    pub const fn version(self, version: &'static str) -> Self {
        Self {
            version: Some(version),
            ..self
        }
    }

    pub const fn author(self, author: &'static str) -> Self {
        Self {
            author: Some(author),
            ..self
        }
    }

    pub const fn about(self, about: &'static str) -> Self {
        Self {
            about: Some(about),
            ..self
        }
    }
}

pub fn add_personalizations(
    command: clap::Command,
    personalizations: PersonalizeClap,
) -> clap::Command {
    let command = match personalizations.name {
        Some(name) => command.name(name),
        None => command,
    };

    let command = match personalizations.version {
        Some(version) => command.version(version),
        None => command,
    };

    let command = match personalizations.author {
        Some(author) => command.author(author),
        None => command,
    };

    let command = match personalizations.about {
        Some(about) => command.about(about),
        None => command,
    };

    command
}

/// Add a `--lang` argument to the command. This argument will be optional if
/// there is only one language
pub fn add_lang_argument(command: clap::Command, languages: &[&'static str]) -> clap::Command {
    let arg = clap::Arg::new("language")
        .short('l')
        .long("lang")
        .value_name("LANGUAGE")
        .value_parser(PossibleValuesParser::new(languages))
        .action(clap::ArgAction::Set)
        .help("the output language of generated types");

    command.arg(match languages {
        [] => panic!("need at least one language"),
        [lang] => arg.required(false).default_value(lang),
        _ => arg.required(true),
    })
}

/// Given a CliArgsSet for a language, use the name of the language and
/// information about its configuration to populate a clap command with
/// args specific to that language
pub fn add_language_params_to_clap(
    command: clap::Command,
    language: &'static str,
    args: &CliArgsSet,
) -> clap::Command {
    if let Some(arg) = command
        .get_arguments()
        .find(|arg| arg.get_id().as_str().starts_with(language))
    {
        panic!(
            "existing argument {id:?} conflicts with language {language}",
            id = arg.get_id().as_str(),
        )
    }

    args.iter().fold(command, |command, spec| {
        let arg = clap::Arg::new(spec.full_key.to_owned())
            .long(spec.full_key.to_owned())
            .required(false);

        command.arg(match spec.arg_type {
            ArgType::Bool => arg.action(clap::ArgAction::SetTrue),
            ArgType::Value => arg.action(clap::ArgAction::Set).value_name(spec.key),
        })
    })
}
