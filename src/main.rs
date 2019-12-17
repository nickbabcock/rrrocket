use boxcars::{CrcCheck, NetworkParse, ParseError, ParserBuilder, Replay};
use glob::glob;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use serde::Serialize;
use snafu::{ErrorCompat, ResultExt, Snafu};
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, Snafu)]
enum SerdeError {
    #[snafu(display("serialization failed from an io error: {}", source))]
    IoError { source: io::Error },

    #[snafu(display("serialization failed: {}", source))]
    OtherError { source: serde_json::Error },
}

#[derive(Debug, Snafu)]
enum RocketError {
    #[snafu(display("Unable to read replay from {}: {}", path.display(), source))]
    ReadReplay { source: io::Error, path: PathBuf },

    #[snafu(display("Unable to read directory for replays {}: {}", path.display(), source))]
    ReadDir { source: io::Error, path: PathBuf },

    #[snafu(display("Unable to parse replay {}: {}", path.display(), source))]
    ParseReplay {
        #[snafu(source(from(ParseError, Box::new)))]
        source: Box<ParseError>,
        path: PathBuf,
    },

    #[snafu(display("Unable to form glob from {}: {}", path.display(), source))]
    InvalidGlob {
        source: glob::PatternError,
        path: PathBuf,
    },

    #[snafu(display("Unable to form glob from {}: {}", source.path().display(), source))]
    InvalidGlobRead { source: glob::GlobError },

    #[snafu(display("Could not open json output file {}: {}", path.display(), source))]
    OpenOutput { source: io::Error, path: PathBuf },

    #[snafu(display("Could not serialize replay {}: {}", path.display(), source))]
    JsonSerialization { source: SerdeError, path: PathBuf },

    #[snafu(display("Expected one input file --multiple is not specified"))]
    InputArguments,

    #[snafu(display("Could not read stdin: {}", source))]
    ReadStdin { source: io::Error },

    #[snafu(display("Could not write to stdout: {}", source))]
    WriteStdout { source: io::Error },
}

#[derive(StructOpt, Debug, Clone, PartialEq)]
#[structopt(
    name = "rrrocket",
    about = "Parses Rocket League replay files and outputs JSON with decoded information"
)]
struct Opt {
    #[structopt(
        short = "c",
        long = "crc-check",
        help = "forces a crc check for corruption even when replay was successfully parsed"
    )]
    crc: bool,

    #[structopt(
        short = "n",
        long = "network-parse",
        help = "parses the network data of a replay instead of skipping it"
    )]
    body: bool,

    #[structopt(
        short = "m",
        long = "multiple",
        help = "parse multiple replays in provided directories. Defaults to writing to a sibling JSON file, but can output to stdout with --json-lines"
    )]
    multiple: bool,

    #[structopt(
        short = "p",
        long = "pretty",
        help = "output replay as pretty-printed JSON"
    )]
    pretty: bool,

    #[structopt(
        short = "j",
        long = "json-lines",
        help = "output multiple files to stdout via json lines"
    )]
    json_lines: bool,

    #[structopt(long = "dry-run", help = "parses but does not write JSON output")]
    dry_run: bool,

    #[structopt(help = "Rocket League replay files")]
    input: Vec<PathBuf>,
}

#[derive(Serialize, Debug)]
struct RocketReplay<'a> {
    file: &'a PathBuf,
    replay: Replay,
}

fn read_file(opt: &Opt, file_path: PathBuf) -> Result<(PathBuf, Replay), RocketError> {
    // Try to mmap the file first so we don't have to worry about potentially allocating a large
    // buffer in case there is like a 10GB iso file that ends in .replay
    let f = fs::File::open(&file_path).context(ReadReplay { path: &file_path })?;
    let mmap = unsafe { memmap::MmapOptions::new().map(&f) };
    match mmap {
        Ok(data) => {
            let replay = parse_replay(opt, &data).context(ParseReplay { path: &file_path })?;
            Ok((file_path, replay))
        }
        Err(_) => {
            // If the mmap fails, just try reading the file
            let data = fs::read(&file_path).context(ReadReplay { path: &file_path })?;
            let replay = parse_replay(opt, &data).context(ParseReplay { path: &file_path })?;
            Ok((file_path, replay))
        }
    }
}

fn parse_replay(opt: &Opt, data: &[u8]) -> Result<Replay, ParseError> {
    ParserBuilder::new(&data[..])
        .with_crc_check(if opt.crc {
            CrcCheck::Always
        } else {
            CrcCheck::OnError
        })
        .with_network_parse(if opt.body {
            NetworkParse::Always
        } else {
            NetworkParse::Never
        })
        .parse()
}

fn expand_directory(dir: &Path) -> impl Iterator<Item = Result<PathBuf, RocketError>> {
    let dir_glob_fmt = format!("{}/**/*.replay", dir.display());
    let replays = glob(&dir_glob_fmt).map_err(|e| RocketError::InvalidGlob {
        source: e,
        path: dir.to_path_buf(),
    });

    match replays {
        Err(e) => either::Either::Left(std::iter::once(Err(e))),
        Ok(replays) => {
            let res = replays
                .inspect(|file| {
                    if let Err(ref e) = file {
                        eprintln!("Unable to inspect: {}", e)
                    }
                })
                .filter_map(|x| {
                    if let Ok(pth) = x {
                        if pth.is_file() {
                            Some(Ok(pth))
                        } else {
                            None
                        }
                    } else {
                        let err = x.map_err(|e| RocketError::InvalidGlobRead { source: e });
                        Some(err)
                    }
                });
            either::Either::Right(res)
        }
    }
}

/// Each file argument that we get could be a directory so we need to expand that directory and
/// find all the *.replay files. File arguments turn into single element vectors.
fn expand_paths<'a>(
    files: &'a [PathBuf],
) -> impl Iterator<Item = Result<PathBuf, RocketError>> + 'a {
    files
        .iter()
        .map(|arg_file| {
            let p = Path::new(arg_file);
            if p.is_file() {
                either::Either::Left(std::iter::once(Ok(p.to_path_buf())))
            } else {
                either::Either::Right(expand_directory(&p))
            }
        })
        .flatten()
}

fn parse_multiple_replays(opt: &Opt) -> Result<(), RocketError> {
    let res = expand_paths(&opt.input)
        .inspect(|file| {
            if let Err(ref e) = file {
                eprintln!("Unable to inspect: {}", e)
            }
        })
        .flat_map(Result::ok)
        .par_bridge()
        .map(|file_path| read_file(opt, file_path));

    if opt.dry_run {
        res.inspect(|parse_result| match parse_result {
            Ok((file, _replay)) => println!("Parsed {}", file.display()),
            Err(ref e) => eprintln!("Failed {}", e),
        })
        .map(|_| ())
        .collect::<()>();
        Ok(())
    } else if opt.json_lines {
        let json_lines = res.map(|parse_result| {
            parse_result.and_then(|(file, replay)| {
                let rep = RocketReplay {
                    file: &file,
                    replay,
                };
                serde_json::to_string(&rep)
                    .map_err(unwrap_io_error)
                    .context(JsonSerialization { path: file })
            })
        });

        let (send, recv) = std::sync::mpsc::sync_channel(rayon::current_num_threads());
        let thrd = std::thread::spawn(|| {
            let stdout = io::stdout();
            let lock = stdout.lock();
            let mut writer = BufWriter::new(lock);
            for line in recv.into_iter() {
                if let Ok(json) = line {
                    writeln!(writer, "{}", json)?;
                    writer.flush()?;
                }
            }

            Ok(())
        });

        // Ignore the send error as the receiver can hang up when it is no longer supposed to be
        // writing to stdout (eg: broken pipe).
        let _ = json_lines.try_for_each_with(send, |s, x| s.send(x));
        match thrd.join() {
            Err(e) => {
                eprintln!("Unable to join internal thread: {:?}", e);
                Ok(())
            }
            Ok(x) => x.context(WriteStdout),
        }
    } else {
        res.map(|parse_result| {
            parse_result.and_then(|(file, replay)| {
                let outfile = format!("{}.json", file.display());
                let fout = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&outfile)
                    .context(OpenOutput { path: outfile })?;
                let mut writer = BufWriter::new(fout);
                serialize(opt.pretty, &mut writer, &replay)
                    .context(JsonSerialization { path: file })
            })
        })
        .collect::<Result<(), RocketError>>()?;
        Ok(())
    }
}

fn unwrap_io_error(e: serde_json::Error) -> SerdeError {
    if e.is_io() {
        let ioe: std::io::Error = e.into();
        SerdeError::IoError { source: ioe }
    } else {
        SerdeError::OtherError { source: e }
    }
}

fn serialize<W: Write>(pretty: bool, writer: W, replay: &Replay) -> Result<(), SerdeError> {
    let res = if pretty {
        serde_json::to_writer_pretty(writer, &replay)
    } else {
        serde_json::to_writer(writer, replay)
    };

    res.map_err(unwrap_io_error)
}

fn run() -> Result<(), RocketError> {
    let opt = Opt::from_args();
    if opt.multiple {
        parse_multiple_replays(&opt)
    } else if opt.input.len() > 1 {
        Err(RocketError::InputArguments)
    } else {
        let replay = if opt.input.is_empty() {
            let mut d = Vec::new();
            io::stdin().read_to_end(&mut d).context(ReadStdin)?;
            parse_replay(&opt, &d).context(ParseReplay { path: "stdin" })
        } else {
            let file = &opt.input[0];
            let (_, replay) = read_file(&opt, file.clone())?;
            Ok(replay)
        }?;

        if !opt.dry_run {
            let stdout = io::stdout();
            let lock = stdout.lock();
            serialize(opt.pretty, BufWriter::new(lock), &replay)
                .context(JsonSerialization { path: "stdout" })?;
        }
        Ok(())
    }
}

fn main() {
    if let Err(e) = run() {
        // Try and detect a broken pipe (piping stdout to head -c 50),
        // and if so exit gracefully
        let mut root = e.source();
        while let Some(source) = root {
            if let Some(io_error) = source.downcast_ref::<std::io::Error>() {
                if io_error.kind() == std::io::ErrorKind::BrokenPipe {
                    ::std::process::exit(0);
                }
            }

            root = source.source();
        }

        eprintln!("An error occurred: {}", e);
        if let Some(backtrace) = ErrorCompat::backtrace(&e) {
            eprintln!("{}", backtrace);
        }

        ::std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use assert_cmd::cmd::Command;
    use predicates::prelude::*;
    use tempfile::tempdir;

    #[test]
    fn test_error_output() {
        Command::cargo_bin("rrrocket")
            .unwrap()
            .args(&[
                "-n",
                "-c",
                "--dry-run",
                "non-exist/assets/fuzz-string-too-long.replay",
            ])
            .assert()
            .failure()
            .code(1)
            .stderr(predicate::str::contains(
                "Unable to read replay from non-exist/assets/fuzz-string-too-long.replay",
            ));
    }

    #[test]
    fn test_error_output2() {
        Command::cargo_bin("rrrocket")
            .unwrap()
            .args(&[
                "-n",
                "-c",
                "--dry-run",
                "-m",
                "assets/fuzz-string-too-long.replay",
            ])
            .assert()
            .success()
            .stderr(predicate::str::contains(
                "Unable to parse replay assets/fuzz-string-too-long.replay",
            ))
            .stderr(predicate::str::contains(
                "Crc mismatch. Expected 3765941959 but received 1825689991",
            ));
    }

    #[test]
    fn test_file_in_stdout() {
        Command::cargo_bin("rrrocket")
            .unwrap()
            .args(&["-n", "assets/replays/1ec9.replay"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                r#"{"header_size":1944,"header_crc":3561912561"#,
            ));
    }

    #[test]
    fn test_stdin_stdout() {
        Command::cargo_bin("rrrocket")
            .unwrap()
            .args(&["-n"])
            .pipe_stdin("assets/replays/1ec9.replay")
            .unwrap()
            .assert()
            .success()
            .stdout(predicate::str::contains(
                r#"{"header_size":1944,"header_crc":3561912561"#,
            ));
    }

    #[test]
    fn test_directory_in() {
        let dir = tempdir().unwrap();
        let options = fs_extra::dir::CopyOptions::new();
        let replays_path = dir.path().join("replays");
        let path = replays_path.to_str().unwrap().to_owned();
        fs_extra::dir::copy("assets/replays", dir.path(), &options).unwrap();

        Command::cargo_bin("rrrocket")
            .unwrap()
            .args(&["-n", "-m", &path])
            .assert()
            .success();

        assert!(replays_path.join("1ec9.replay.json").as_path().exists());
    }

    #[test]
    fn test_directory_in_json_lines() {
        let dir = tempdir().unwrap();
        let options = fs_extra::dir::CopyOptions::new();
        let replays_path = dir.path().join("replays");
        let path = replays_path.to_str().unwrap().to_owned();
        fs_extra::dir::copy("assets/replays", dir.path(), &options).unwrap();

        Command::cargo_bin("rrrocket")
            .unwrap()
            .args(&["-n", "-j", "-m", &path])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                r#"{"header_size":1944,"header_crc":3561912561"#,
            ))
            .stdout(predicate::str::contains("\n").count(1));
    }

    #[test]
    fn test_directory_in_json_lines_nested() {
        let dir = tempdir().unwrap();
        let options = fs_extra::dir::CopyOptions::new();
        let replays_path = dir.path().join("replays");
        let path = replays_path.to_str().unwrap().to_owned();
        fs_extra::dir::copy("assets/replays", dir.path(), &options).unwrap();
        fs_extra::dir::copy("assets/replays", replays_path, &options).unwrap();

        Command::cargo_bin("rrrocket")
            .unwrap()
            .args(&["-n", "-j", "-m", &path])
            .assert()
            .success()
            .stdout(
                predicate::str::contains(r#"{"header_size":1944,"header_crc":3561912561"#).count(2),
            )
            .stdout(predicate::str::contains("\n").count(2));
    }
}
