use std::{collections::HashMap, io};

use anyhow::Context as _;
use clap::{CommandFactory as _, FromArgMatches as _};
use clap_complete::generate as generate_completions;
use ignore::{types::TypesBuilder, WalkBuilder};
use typeshare_model::prelude::{CrateName, FilesMode, Language};

use crate::{
    args::{add_lang_argument, add_language_params_to_clap, Command, OutputLocation, StandardArgs},
    config::{
        self, compute_args_set, load_config, load_language_config_from_file_and_args, CliArgsSet,
    },
    parser::{parse_input, parser_inputs, ParsedData},
    writer::write_output,
};

pub struct PersonalizeClap {
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub about: &'static str,
}

pub trait LanguageSet {
    type LanguageMetas;

    /// Each language has a set of configuration metadata, describing all
    /// of its configuration parameters. This metadata is used to populate
    /// the clap command with language specific parameters for each language,
    /// and to load a fully configured language. It is computed based on the
    /// serde serialization of a config.
    fn compute_language_metas() -> anyhow::Result<Self::LanguageMetas>;

    /// Add the `--language` argument to the command, such that all of the
    /// languages in this set are possible values for that argument
    fn add_lang_argument(command: clap::Command) -> clap::Command;

    /// Add all of the language-specific arguments to the clap command.
    fn add_language_specific_arguments(
        command: clap::Command,
        metas: &Self::LanguageMetas,
    ) -> clap::Command;

    fn execute_typeshare_for_language<'c, 'a: 'c>(
        language: &str,
        config: &'c config::Config,
        args: &'c clap::ArgMatches,
        metas: &'a Self::LanguageMetas,
        data: HashMap<Option<CrateName>, ParsedData>,
        destination: &OutputLocation<'_>,
    ) -> anyhow::Result<()>;
}

macro_rules! metas {
    ([$($CliArgsSet:ident)*]) => {
        ($($CliArgsSet,)*)
    };

    ([$($CliArgsSet:ident)*] $Language:ident $($Tail:ident)*) => {
        metas! {[$($CliArgsSet)* CliArgsSet] $($Tail)*}
    };
}

macro_rules! language_set_for {
    ($Language:ident $($Tail:ident)*) => {
        language_set_for! {
            [$Language] $($Tail)*
        }
    };

    ([$($Language:ident)*] $Head:ident $($Tail:ident)*) => {
        language_set_for! {[$($Language)*]}

        language_set_for! {
            [$($Language)* $Head] $($Tail)*
        }
    };

    ([$($Language:ident)+]) => {
        impl< $($Language,)*> LanguageSet for ($($Language,)*)
            where $(
                for<'config> $Language: Language<'config>,
            )*
        {
            type LanguageMetas = metas!([] $($Language)*);

            fn compute_language_metas() -> anyhow::Result<Self::LanguageMetas> {
                Ok(
                     ($(
                        compute_args_set::<$Language>()?,
                    )*),
              )
            }

            fn add_lang_argument(command: clap::Command)->clap::Command {
                add_lang_argument(command, &[$(<$Language as Language>::NAME,)*])
            }

            fn add_language_specific_arguments(
                command: clap::Command,
                metas: &Self::LanguageMetas,
            ) -> clap::Command {
                #[allow(non_snake_case)]
                let ($($Language,)*) = metas;

                $(
                    let command = add_language_params_to_clap(command, $Language);
                )*

                command
            }

            fn execute_typeshare_for_language<'c, 'a: 'c>(
                language: &str,
                config: &'c config::Config,
                args: &'c clap::ArgMatches,
                metas: &'a Self::LanguageMetas,
                data: HashMap<Option<CrateName>, ParsedData>,
                destination: &OutputLocation<'_>,
            ) -> anyhow::Result<()> {
                #[allow(non_snake_case)]
                let ($($Language,)*) = metas;

                $(
                    if language == <$Language as Language>::NAME {
                        execute_typeshare_for_language::<$Language>(
                            config,
                            args,
                            $Language,
                            data,
                            destination
                        )
                    } else
                )*
                {
                    anyhow::bail!("{language} isn't a valid language; clap should have prevented this")
                }
            }
        }
    }
}

fn execute_typeshare_for_language<'config, 'a: 'config, L: Language<'config>>(
    config: &'config config::Config,
    args: &'config clap::ArgMatches,
    meta: &'a CliArgsSet,
    data: HashMap<Option<CrateName>, ParsedData>,
    destination: &OutputLocation<'_>,
) -> anyhow::Result<()> {
    let name = L::NAME;

    let config = load_language_config_from_file_and_args::<L>(&config, &args, meta)
        .with_context(|| format!("failed to load configuration for language {name}"))?;

    let language_implementation = <L>::new_from_config(config)
        .with_context(|| format!("failed to load configuration for language {name}"))?;

    write_output(&language_implementation, data, destination)
        .with_context(|| format!("failed to generate typeshared code for language {name}"))?;

    Ok(())
}

// We support typeshare binaries for up to 16 languages. Fork us and make your
// own if that's not enough for you.
language_set_for! {
    A B C D
    E F G H
    I J K L
    M N O P
}

pub fn main_body<Languages>() -> anyhow::Result<()>
where
    Languages: LanguageSet,
{
    let language_metas = Languages::compute_language_metas()?;
    let command = StandardArgs::command();

    // let command = command
    //     .name(personalize.name)
    //     .version(personalize.version)
    //     .author(personalize.author)
    //     .about(personalize.about);

    let command = Languages::add_lang_argument(command);
    let command = Languages::add_language_specific_arguments(command, &language_metas);

    // Parse command line arguments. Need to clone here because we
    // need to be able to generate completions later.
    let args = command.clone().get_matches();

    // Load the standard arguments from the parsed arguments. Generally
    // we expect that this won't fail, because the `command` has been
    // configured to only give us valid arrangements of args
    let standard_args = StandardArgs::from_arg_matches(&args)
        .expect("StandardArgs should always be loadable from a `command`");

    // If we asked for completions, do that before anything else
    if let Some(options) = standard_args.subcommand {
        match options {
            Command::Completions { shell } => {
                let mut command = command;
                let bin_name = command.get_name().to_string();
                generate_completions(shell, &mut command, bin_name, &mut io::stdout());
            }
        }

        return Ok(());
    }

    // Load all of the language configurations
    let config = load_config(standard_args.config.as_deref())?;

    // Construct the directory walker that will produce the list of
    // files to typeshare
    let walker = {
        let directories = standard_args.directories.as_slice();
        let (first_dir, other_dirs) = directories
            .split_first()
            .expect("clap should guarantee that there's at least one input directory");

        let mut types = TypesBuilder::new();
        types.add("rust", "*.rs").unwrap();
        types.select("rust");

        let mut walker_builder = WalkBuilder::new(first_dir);
        walker_builder.types(types.build().unwrap());
        other_dirs.iter().for_each(|dir| {
            walker_builder.add(dir);
        });
        walker_builder.build()
    };

    // Collect all of the files we intend to parse with typeshare
    let parser_inputs = parser_inputs(walker);

    // Parse those files
    let data = parse_input(
        parser_inputs,
        &[],
        if standard_args.output.file.is_some() {
            FilesMode::Single
        } else {
            FilesMode::Multi(())
        },
    )
    .unwrap();

    let destination = standard_args.output.location();

    let language: &String = args
        .get_one("language")
        .expect("clap should guarantee that --lang is provided");

    let out = Languages::execute_typeshare_for_language(
        &language,
        &config,
        &args,
        &language_metas,
        data,
        &destination,
    );

    out
}
