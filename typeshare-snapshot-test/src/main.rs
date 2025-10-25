//! Binary to generate or run snapshot tests.
mod config;
mod sorted_iter;

use crate::{
    config::read_toml,
    sorted_iter::{EitherOrBoth, SortedPairsIter},
};
use anyhow::Context;
use clap::Parser;
use core::str;
use indent_write::{indentable::Indentable, io::IndentWriter};
use lazy_format::lazy_format;
use similar::TextDiff;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::Display,
    fs,
    io::{self, Write as _, stderr},
    panic::resume_unwind,
    path::{Path, PathBuf},
    process::{Command, Stdio, exit},
    thread,
};

/**
Utility for capturing and running snapshot tests for your implementation of
typeshare. See the README.md for a tutorial on how to use this; this usage
message is just a reference.

typeshare-snapshot-test expects one directory per test, all contained in a
single parent directory. When run, it will either capture a new snapshot for
each test in that directory, or will test that the typeshare output matches
the existing snapshot.

If the test directory contains a subdirectory called input, we assume we're
running this test in multi-file mode. In this case, that input directory
should contain one or more dummy rust crates with the standard layout
(crate_name/src/{files}.rs). Otherwise, we'll assume the test is for
single-file output, and the directory should just contain one or more .rs
files.

If any directory doesn't have a captured snapshot, it will be skipped with a
non-fatal warning.

Afterwards, typeshare-snapshot-test will print a report and exit nonzero if
anything went wrong. This could include snapshot test failures (if the output
didn't match the output), nonzero exits from you `typeshare` binary, or any
filesystem errors or layout issues in your tests.

## Config

If the snapshot tests directory includes a `typeshare.toml` file, it will be
used for every test. If any individual directory includes a `typeshare.toml`,
it will be used for that specific test; if both configs exist, they will be
merged, and the merged version passed into typeshare.
 */
#[derive(Parser)]
struct Args {
    /// The directory containing all of the snapshot tests. Each snapshot test
    /// should be a subdirectory inside of this parent directory, and is
    /// identified by the name of that directory.
    snapshots: PathBuf,

    /// name or path to your typeshare binary. If omitted, we assume the
    /// binary is called `typeshare-{LANGUAGE}`.
    #[arg(short, long)]
    typeshare: Option<PathBuf>,

    /// Name of the language for which we're performing a snapshot test.
    /// This is passed as `--language` to the underlying typeshare call, and
    /// also is used as the name of the snapshot output in some circumstances.
    #[arg(short, long)]
    language: String,

    /// Suffix for the output file for this snapshot test. This should always
    /// be the typical suffix for your language (.go for golang, .ts for
    /// typescript, etc).
    #[arg(short, long)]
    suffix: String,

    /// Which mode to run (capture a new snapshot, or perform a snapshot test)
    #[arg(short, long)]
    mode: Mode,

    /// If given, only these paths will be targeted by `typeshare-snapshot-test`.
    #[arg(short, long)]
    include: Option<Vec<String>>,

    /// Any additional arguments to pass to the underlying `typeshare`
    /// binary. Make sure to use `--` to separate options passed to `typeshare`
    /// from options passed to `typeshare-snapshot-test`
    additional_args: Vec<String>,
    // TODO: concurrency limiter
}

impl Args {
    // We want to allow people to write `--suffix ts` or `--suffix .tx`,
    // whatever brings them joy.
    fn trimmed_suffix(&self) -> &str {
        match self.suffix.strip_prefix('.') {
            Some(suffix) => suffix,
            None => self.suffix.as_str(),
        }
    }

    fn typeshare_binary(&self) -> Cow<'_, Path> {
        self.typeshare
            .as_deref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(PathBuf::from(format!("typeshare-{}", self.language))))
    }

    fn include_names(&self) -> Option<HashSet<&str>> {
        self.include
            .as_ref()
            .map(|include| include.iter().map(|name| name.as_str()).collect())
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum Mode {
    /// Recreate typeshare output, but only for snapshot tests that already
    /// have output in this language.
    Regenerate,

    /// Capture a new snapshot for all snapshot tests.
    Generate,

    /// Execute a typeshare snapshot test
    Test,
}

#[derive(Debug, Default)]
struct Stats {
    success: i32,
    warning: i32,
    failed: i32,
}

impl Stats {
    pub fn total(&self) -> i32 {
        self.success + self.warning + self.failed
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Stats {
            success,
            warning,
            failed,
        } = *self;
        let total = self.total();
        let total_success = success + warning;

        let warnings = lazy_format!(
            if warning > 0 => " ({warning} with warnings)"
        );

        write!(
            f,
            "{total_success}/{total} passed{warnings}. {failed}/{total} failures."
        )
    }
}

enum Report {
    Success,

    /// Skip is the same as success, but it doesn't increment the stats counters.
    /// Usually Warning is preferable.
    Skip,

    /// Something interesting happened that we should tell the user about, but
    /// not enough to cause a nonzero exit
    Warning {
        // Feel free to switch this to a String or Cow<str> if need be
        message: &'static str,
    },

    /// The typeshare command exited nonzero
    CommandError {
        command: Vec<String>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },

    /// The operation– either a test, or a generate– encountered an error. This
    /// is different than a test failure.
    OperationError {
        command: Vec<String>,
        error: anyhow::Error,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },

    /// There was a difference between the snapshot and the actual typeshare
    /// output
    TestFailure {
        diff: ReportDiff,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
}

impl Report {
    pub fn is_problem(&self) -> bool {
        matches!(
            *self,
            Report::TestFailure { .. }
                | Report::OperationError { .. }
                | Report::CommandError { .. }
        )
    }
}

fn write_captured_output(mut dest: impl io::Write, stdout: &[u8], stderr: &[u8]) -> io::Result<()> {
    let mut tail = false;
    if !stdout.trim_ascii().is_empty() {
        writeln!(dest, "--------captured stdout--------")?;
        dest.write_all(stdout)?;
        tail = true;
    }
    if !stderr.trim_ascii().is_empty() {
        writeln!(dest, "\n--------captured stderr--------")?;
        dest.write_all(stderr)?;
        tail = true;
    }
    if tail {
        writeln!(dest, "\n-------------------------------")?
    }
    Ok(())
}

impl Report {
    fn print_report(&self, name: &str, dest: &mut impl io::Write) -> io::Result<()> {
        match *self {
            Report::Success | Report::Skip => Ok(()),
            Report::Warning { ref message } => writeln!(dest, "warning from {name}: {message}"),
            Report::CommandError {
                ref command,
                ref stdout,
                ref stderr,
            } => {
                writeln!(dest, "error in snapshot test {name}:")?;
                let mut dest = IndentWriter::new("  ", dest);
                writeln!(dest, "typeshare command exited nonzero")?;
                writeln!(dest, "typeshare arguments: {:#?}", command)?;

                write_captured_output(dest, stdout, stderr)
            }
            Report::OperationError {
                ref command,
                ref error,
                ref stdout,
                ref stderr,
            } => {
                writeln!(dest, "error in snapshot test {name}:")?;
                let mut dest = IndentWriter::new("    ", dest);
                writeln!(dest, "typeshare arguments: {:#?}", command)?;
                write_captured_output(&mut dest, stdout, stderr)?;
                writeln!(dest, "{error:?}")
            }
            Report::TestFailure {
                ref diff,
                ref stdout,
                ref stderr,
            } => match diff {
                ReportDiff::Directory(report) => {
                    writeln!(dest, "test failure in {name}:")?;
                    let mut dest = IndentWriter::new("    ", dest);
                    if !report.unexpected_files.is_empty() {
                        writeln!(dest, "these files were not present in the snapshot:")?;
                        let mut dest = IndentWriter::new("  ", &mut dest);
                        report
                            .unexpected_files
                            .iter()
                            .try_for_each(|filename| writeln!(dest, "{filename}"))?;
                    }

                    if !report.absent_files.is_empty() {
                        writeln!(dest, "these expected files were absent:")?;
                        let mut dest = IndentWriter::new("  ", &mut dest);
                        report
                            .absent_files
                            .iter()
                            .try_for_each(|filename| writeln!(dest, "{filename}"))?;
                    }

                    report.diffs.iter().try_for_each(|(filename, diff)| {
                        let diff = diff.indented("| ");
                        writeln!(
                            dest,
                            "the typeshare output didn't match the snapshot for {filename}:\n{diff}"
                        )
                    })
                }
                ReportDiff::File(diff) => {
                    let diff = diff.indented("|   ");
                    writeln!(dest, "test failure in {name}:")?;
                    write_captured_output(&mut *dest, stdout, stderr)?;
                    writeln!(dest, "{diff}")
                }
            },
        }
    }
}

/// This should always have some kind of problem
#[derive(Debug)]
enum ReportDiff {
    Directory(DirDiffReport),
    File(FileDiff),
}

#[derive(Debug, Default)]
struct DirDiffReport {
    /// Filenames of files that were expected but didn't appear
    absent_files: BTreeSet<String>,

    /// Filenames of files that appeared but werent' expected
    unexpected_files: BTreeSet<String>,

    /// Diffs. Mapping from filename to the printable diff
    diffs: BTreeMap<String, FileDiff>,
}

impl DirDiffReport {
    pub fn any_problems(&self) -> bool {
        !(self.absent_files.is_empty() && self.unexpected_files.is_empty() && self.diffs.is_empty())
    }
}

/// Given a directory, return a listing of all of the things in that directory.
fn file_listing(directory: &Path) -> anyhow::Result<BTreeSet<String>> {
    let entries = fs::read_dir(directory).context("failed to read directory")?;

    entries
        .map(|entry| {
            let entry = entry.context("failed to list directory")?;
            let name = entry.file_name();
            let name = name.to_str().with_context(|| {
                let name = name.to_string_lossy();
                format!("file '{name}' had an invalid (non-utf8) filename")
            })?;
            Ok(name.to_owned())
        })
        .collect()
}

fn dir_diff(correct_path: &Path, test_path: &Path) -> anyhow::Result<DirDiffReport> {
    // We assume that all of the contents of both of the input directories;
    // the alternative is an error. In the future we can have better error
    // messages if, for example, someone creates a nested directory.

    let correct_listing = file_listing(correct_path)
        .with_context(|| format!("error reading contents of '{}'", correct_path.display()))?;

    let test_listing = file_listing(test_path)
        .with_context(|| format!("error reading contents of '{}'", test_path.display()))?;

    let listings = SortedPairsIter::new(correct_listing, test_listing);

    thread::scope(|s| {
        let mut report = DirDiffReport::default();

        let mut diff_threads = Vec::new();

        for entry in listings {
            match entry {
                EitherOrBoth::Left(expected_file) => {
                    report.absent_files.insert(expected_file);
                }
                EitherOrBoth::Right(new_file) => {
                    report.unexpected_files.insert(new_file);
                }
                EitherOrBoth::Both(file) => diff_threads.push(s.spawn(|| {
                    let correct_path = correct_path.join(&file);
                    let test_path = test_path.join(&file);
                    let result = file_diff(&correct_path, &test_path, &file);
                    (file, result)
                })),
            }
        }

        for thread in diff_threads {
            let (filename, diff) = thread.join().unwrap_or_else(|panic| resume_unwind(panic));
            let diff = diff.with_context(|| format!("error computing diff for '{filename}'"))?;
            if let Some(diff) = diff {
                report.diffs.insert(filename, diff);
            }
        }

        Ok(report)
    })
}

#[derive(Debug)]
struct FileDiff {
    human_readable_diff: String,
}

impl Display for FileDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.human_readable_diff, f)
    }
}

fn file_diff(
    correct_path: &Path,
    test_path: &Path,
    diff_filename: &str,
) -> anyhow::Result<Option<FileDiff>> {
    let correct_content = fs::read(correct_path)
        .with_context(|| format!("failed to read '{}'", correct_path.display()))?;

    let test_content =
        fs::read(test_path).with_context(|| format!("failed to read '{}'", test_path.display()))?;

    if correct_content == test_content {
        return Ok(None);
    }

    let diff = TextDiff::configure()
        .algorithm(similar::Algorithm::Patience)
        .diff_lines(&correct_content, &test_content);

    let correct_file_name = format!("expected  {diff_filename}");
    let test_file_name = format!("  actual  {diff_filename}");

    let human_readable_diff = diff
        .unified_diff()
        .header(&correct_file_name, &test_file_name)
        .to_string();

    Ok(Some(FileDiff {
        human_readable_diff,
    }))
}

/// Struct that, when dropped, will make a best effort to delete the given path
struct TempFileGuard<'a> {
    path: &'a Path,
}

impl Drop for TempFileGuard<'_> {
    fn drop(&mut self) {
        clear_item(self.path);
    }
}

/// Remove something from the filesystem, without worrying about if it
/// actually succeeded
fn clear_item(path: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(path);
}

#[expect(clippy::too_many_arguments)]
fn snapshot_test(
    snapshot_directory: &Path,
    mode: Mode,
    typeshare: &Path,
    base_config: &toml::Table,
    base_config_path: &Path,
    language: &str,
    suffix: &str,
    additional_arguments: &[String],
) -> anyhow::Result<Report> {
    // `output_path` is the path into which typeshare will be told to direct
    // its output. It is a temporary path which shouldn't ever exist but
    // ephemerally; used as an input to a diff operation, or renamed to the
    // more permanent path when capturing a new snapshot.
    let output_path = snapshot_directory.join("TYPESHARE-TEMP-OUTPUT");
    let merged_config_path = snapshot_directory.join("TYPESHARE-TEMP-CONFIG.toml");
    clear_item(&output_path);
    clear_item(&merged_config_path);
    let guard = (
        TempFileGuard { path: &output_path },
        TempFileGuard {
            path: &merged_config_path,
        },
    );

    // Check if there exists a directory called `input`. If there does, we need
    // to treat it as a container of fake crates for multi-file output;
    // otherwise, we're doing single-file output.
    let mutli_file_input_dir = snapshot_directory.join("input");

    let multi_file = match mutli_file_input_dir.metadata() {
        Ok(metadata) => match metadata.is_dir() {
            true => true,
            false => anyhow::bail!(
                "'input' exists, but it was a file. It should be a directory \
                (for multi-file mode) or include `.rs` suffix."
            ),
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => false,
        Err(err) => Err(err).with_context(|| {
            format!(
                "i/o error trying to detect '{}'",
                mutli_file_input_dir.display(),
            )
        })?,
    };

    let (filename, destination_path) = if multi_file {
        (String::new(), snapshot_directory.join(language))
    } else {
        let filename = format!("output.{suffix}");
        let destination_path = snapshot_directory.join(&filename);

        (filename, destination_path)
    };

    // Unless we are generating a new snapshot, the previous snapshot must
    // already exist
    if !destination_path.exists() && matches!(mode, Mode::Regenerate | Mode::Test) {
        return Ok(Report::Warning {
            message: "skipped (no existing snapshot)",
        });
    }

    let mut command = Command::new(typeshare);
    command.arg("--lang").arg(language);

    // Create a config file for this test by merging the test's config with
    // the outer config
    {
        let local_config_path = snapshot_directory.join("typeshare.toml");
        let local_config = config::read_toml(&local_config_path).with_context(|| {
            format!(
                "failed to read local typeshare config at '{}'",
                local_config_path.display()
            )
        })?;

        let path = match (base_config.is_empty(), local_config.is_empty()) {
            (true, true) => None,
            (false, true) => Some(base_config_path),
            (true, false) => Some(local_config_path.as_path()),
            (false, false) => {
                // In this case, we need to create and write a new, merged
                // config
                let mut merged_config = base_config.clone();
                config::merge_configs(&mut merged_config, local_config);
                let merged_config = toml::to_string(&merged_config).expect(
                    "merging a pair of toml documents together \
                     should always make valid toml",
                );
                fs::write(&merged_config_path, &merged_config)
                    .context("i/o error creating merged typeshare config")?;
                Some(merged_config_path.as_path())
            }
        };

        if let Some(path) = path {
            command.arg("--config").arg(path);
        }
    }

    if multi_file {
        command.arg("--output-folder")
    } else {
        command.arg("--output-file")
    }
    .arg(&output_path);

    if multi_file {
        command.arg(&mutli_file_input_dir)
    } else {
        command.arg(snapshot_directory)
    };

    command.args(additional_arguments);

    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let command_args = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    let handle = command.spawn().with_context(|| {
        format!(
            "failed to spawn '{}' (arguments: {:?})",
            typeshare.display(),
            &command_args
        )
    })?;

    let out = handle
        .wait_with_output()
        .context("failed to await typeshare command")?;

    if !out.status.success() {
        return Ok(Report::CommandError {
            command: command_args,
            stdout: out.stdout,
            stderr: out.stderr,
        });
    }

    let operation_result = match mode {
        Mode::Generate | Mode::Regenerate => {
            // In both single-file and multi-file mode, capturing a new
            // screenshot is equivelent to moving the output file or output
            // directory to `destination_path`
            clear_item(&destination_path);

            fs::rename(&output_path, &destination_path)
                .with_context(|| {
                    format!(
                        "failed to capture snapshot to '{}'",
                        destination_path.display()
                    )
                })
                .map(|()| None)
        }
        Mode::Test => match multi_file {
            false => file_diff(&destination_path, &output_path, &filename)
                .context("error computing diff")
                .map(|report| report.map(ReportDiff::File)),
            true => dir_diff(&destination_path, &output_path)
                .context("error computing multi-file diff")
                .map(|report| match report.any_problems() {
                    false => None,
                    true => Some(ReportDiff::Directory(report)),
                }),
        },
    };

    // This causes any remaining temporary file / directory to be cleared.
    // technically we don't need the explicit drop here, but I like
    // guaranteeing that the guard isn't accidentally dropped sooner, and it
    // also means we don't need to call it `_guard`
    drop(guard);

    Ok(match operation_result {
        Ok(diff) => match diff {
            None => Report::Success,
            Some(diff) => Report::TestFailure {
                diff,
                stdout: out.stdout,
                stderr: out.stderr,
            },
        },
        Err(error) => Report::OperationError {
            command: command_args,
            error,
            stdout: out.stdout,
            stderr: out.stderr,
        },
    })
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let typeshare_binary = args.typeshare_binary();

    // First, do `typeshare --help`. We do this only so that we verify that
    // the given typeshare binary exists, without that error appearing a dozen
    // times.
    let _ = Command::new(typeshare_binary.as_ref())
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| {
            format!(
                "failed to launch '{}'; does it exist?",
                typeshare_binary.display()
            )
        })?
        // We don't actually care about the process finishing successfully
        // or anything, only that it was possible to spawn it
        .wait();

    let base_config_path = args.snapshots.join("typeshare.toml");
    let base_config = read_toml(&base_config_path).with_context(|| {
        format!(
            "failed to load base config file at '{}'",
            base_config_path.display()
        )
    })?;

    let snapshots_dir = fs::read_dir(&args.snapshots).with_context(|| {
        format!(
            "Failed to open snapshots directory '{}'",
            args.snapshots.display()
        )
    })?;

    let include_names = args.include_names();
    let include_names = include_names.as_ref();

    // TODO: iterate the include names, instead of iterating the whole directory
    // and filtering. Probably this involves pre-iterating the files.

    let reports: anyhow::Result<BTreeMap<String, Report>> = thread::scope(|s| {
        let threads: Vec<_> = snapshots_dir
            .map(|snapshot_dir| {
                s.spawn(|| {
                    let entry =
                        snapshot_dir.context("i/o error while iterating snapshots directory")?;

                    let entry_path = entry.path();
                    let entry_name = entry.file_name();
                    let entry_name = entry_name.to_string_lossy();
                    let entry_name = entry_name.into_owned();

                    if let Some(names) = include_names
                        && !names.contains(entry_name.as_str())
                    {
                        return Ok((entry_name, Report::Skip));
                    }

                    let meta = entry_path.metadata().with_context(|| {
                        format!(
                            "error reading snapshot directory '{}'",
                            entry_path.display()
                        )
                    })?;

                    if meta.is_file() {
                        let report = match entry_name.as_str() {
                            "README.md" | ".gitignore" | "typeshare.toml" => Report::Skip,
                            _ => Report::Warning {
                                message: "skipped (all snapshot tests are \
                                    directories; this is a file)",
                            },
                        };

                        return Ok((entry_name, report));
                    }

                    snapshot_test(
                        &entry_path,
                        args.mode,
                        &typeshare_binary,
                        &base_config,
                        &base_config_path,
                        &args.language,
                        args.trimmed_suffix(),
                        &args.additional_args,
                    )
                    .with_context(|| format!("error from snapshot test {entry_name}"))
                    .map(|report| (entry_name, report))
                })
            })
            .collect();

        // Collect all of the threads into a set of reports. Skip threads that
        // returned None.
        threads
            .into_iter()
            .map(|thread| thread.join().unwrap_or_else(|panic| resume_unwind(panic)))
            .collect()
    });

    let reports = reports?;

    let mut stats = Stats::default();
    let mut stderr = stderr();

    for (entry, report) in &reports {
        report
            .print_report(entry, &mut stderr)
            .expect("shouldn't be a problem writing to stderr");

        match report {
            Report::Skip => {}
            Report::Success => stats.success += 1,
            Report::Warning { .. } => stats.warning += 1,
            Report::CommandError { .. }
            | Report::OperationError { .. }
            | Report::TestFailure { .. } => stats.failed += 1,
        }
    }

    if matches!(args.mode, Mode::Test) {
        writeln!(stderr, "{stats}").expect("shouldn't be a problem writing to stderr");
    }

    if reports.iter().any(|(_, report)| report.is_problem()) {
        exit(1);
    }

    Ok(())
}
