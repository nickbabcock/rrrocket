use anyhow::{bail, Context};
use boxcars::{CrcCheck, NetworkParse, ParseError, ParserBuilder, Replay};
use clap::Parser;
use glob::glob;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, sync_channel};
use std::thread;

// Avoid musl's default allocator due to terrible performance
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Parses Rocket League replay files and outputs JSON with decoded information
#[derive(Parser, Debug, Clone, PartialEq)]
struct Opt {
    #[arg(
        short = 'c',
        long = "crc-check",
        help = "forces a crc check for corruption even when replay was successfully parsed"
    )]
    crc: bool,

    #[arg(
        short = 'n',
        long = "network-parse",
        help = "parses the network data of a replay instead of skipping it"
    )]
    body: bool,

    #[arg(
        short = 'm',
        long = "multiple",
        help = "parse multiple replays in provided directories. Defaults to writing to a sibling JSON file, but can output to stdout with --json-lines"
    )]
    multiple: bool,

    #[arg(
        short = 'p',
        long = "pretty",
        help = "output replay as pretty-printed JSON"
    )]
    pretty: bool,

    #[arg(
        short = 'j',
        long = "json-lines",
        help = "output multiple files to stdout via json lines"
    )]
    json_lines: bool,

    #[arg(long = "dry-run", help = "parses but does not write JSON output")]
    dry_run: bool,

    #[arg(help = "Rocket League replay files")]
    input: Vec<PathBuf>,
}

#[derive(Serialize, Debug)]
struct RocketReplay<'a> {
    file: &'a PathBuf,
    replay: Replay,
}

fn read_file(opt: &Opt, file_path: PathBuf) -> anyhow::Result<(PathBuf, Replay)> {
    // Try to mmap the file first so we don't have to worry about potentially allocating a large
    // buffer in case there is like a 10GB iso file that ends in .replay
    let f = fs::File::open(&file_path)?;
    let mmap = unsafe { memmap2::MmapOptions::new().map(&f) };
    match mmap {
        Ok(data) => {
            let replay = parse_replay(opt, &data)?;
            Ok((file_path, replay))
        }
        Err(_) => {
            // If the mmap fails, just try reading the file
            let data = fs::read(&file_path)?;
            let replay = parse_replay(opt, &data)?;
            Ok((file_path, replay))
        }
    }
}

fn parse_replay(opt: &Opt, data: &[u8]) -> Result<Replay, ParseError> {
    ParserBuilder::new(data)
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

fn expand_directory(dir: &Path) -> std::vec::IntoIter<anyhow::Result<PathBuf>> {
    let dir_glob_fmt = format!("{}/**/*.replay", dir.display());
    let replays =
        glob(&dir_glob_fmt).with_context(|| format!("unable to form glob in {}", dir.display()));

    match replays {
        Err(e) => vec![Err(e)].into_iter(),
        Ok(replays) => replays
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
                    Some(x.context("glob error"))
                }
            })
            .collect::<Vec<_>>()
            .into_iter(),
    }
}

/// Each file argument that we get could be a directory so we need to expand that directory and
/// find all the *.replay files. File arguments turn into single element vectors.
fn expand_paths(files: &[PathBuf]) -> impl Iterator<Item = anyhow::Result<PathBuf>> + '_ {
    files.iter().flat_map(|arg_file| {
        let p = Path::new(arg_file);
        if p.is_file() {
            vec![Ok(p.to_path_buf())].into_iter()
        } else {
            expand_directory(p)
        }
    })
}

fn parse_multiple_replays(opt: &Opt) -> anyhow::Result<()> {
    let res = expand_paths(&opt.input)
        .inspect(|file| {
            if let Err(ref e) = file {
                eprintln!("Unable to inspect: {}", e)
            }
        })
        .flat_map(Result::ok)
        .par_bridge()
        .map(|file_path| {
            read_file(opt, file_path.clone())
                .with_context(|| format!("Unable to parse replay {}", file_path.display()))
        });

    if opt.dry_run {
        let (send, recv) = std::sync::mpsc::sync_channel(rayon::current_num_threads());
        let thrd = std::thread::spawn(|| {
            let stdout = io::stdout();
            let lock = stdout.lock();
            let mut writer = BufWriter::new(lock);
            for parsed_replay in recv.into_iter() {
                let res: anyhow::Result<(PathBuf, Replay)> = parsed_replay;
                match res {
                    Ok((file, _replay)) => writeln!(&mut writer, "Parsed: {}", file.display())?,
                    Err(e) => eprintln!("Failed {:?}", e),
                }
            }

            Ok(()) as anyhow::Result<()>
        });

        // Ignore the send error as the receiver can hang up when it is no longer supposed to be
        // writing to stdout (eg: broken pipe).
        let _ = res.try_for_each_with(send, |s, x| s.send(x));
        match thrd.join() {
            Err(e) => {
                eprintln!("Unable to join internal thread: {:?}", e);
                Ok(())
            }
            Ok(x) => x.context("Could not write to stdout"),
        }
    } else if opt.json_lines {
        let json_lines = res.map(|parse_result| {
            parse_result.and_then(|(file, replay)| {
                let rep = RocketReplay {
                    file: &file,
                    replay,
                };
                serde_json::to_string(&rep)
                    .with_context(|| format!("Could not serialize replay {}", file.display()))
            })
        });

        let (send, recv) = std::sync::mpsc::sync_channel(rayon::current_num_threads());
        let thrd = std::thread::spawn(|| {
            let stdout = io::stdout();
            let lock = stdout.lock();
            let mut writer = BufWriter::new(lock);
            for json in recv.into_iter().flatten() {
                writeln!(writer, "{}", json)?;
                writer.flush()?;
            }

            Ok(()) as anyhow::Result<()>
        });

        // Ignore the send error as the receiver can hang up when it is no longer supposed to be
        // writing to stdout (eg: broken pipe).
        let _ = json_lines.try_for_each_with(send, |s, x| s.send(x));
        match thrd.join() {
            Err(e) => {
                eprintln!("Unable to join internal thread: {:?}", e);
                Ok(())
            }
            Ok(x) => x.context("Could not write to stdout"),
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
                    .with_context(|| format!("could not open json output file {}", outfile))?;
                let mut writer = BufWriter::new(fout);
                serialize(opt.pretty, &mut writer, &replay)
                    .with_context(|| format!("Could not serialize replay {}", file.display()))
            })
        })
        .collect::<anyhow::Result<()>>()?;
        Ok(())
    }
}

fn serialize<W: Write>(pretty: bool, writer: W, replay: &Replay) -> anyhow::Result<()> {
    let res = if pretty {
        serde_json::to_writer_pretty(writer, &replay)
    } else {
        serde_json::to_writer(writer, replay)
    };

    res.map_err(|e| e.into())
}

fn zip(file_path: &Path, opt: &Opt) -> anyhow::Result<()> {
    let parallelism = std::thread::available_parallelism()
        .map(|x| x.get().max(2))
        .unwrap_or(2);

    let (tx, rx) = sync_channel(parallelism - 1);
    let (return_buf, receive_buf) = channel::<Vec<u8>>();

    let mut buffer = vec![0u8; rawzip::RECOMMENDED_BUFFER_SIZE];
    let f = fs::File::open(file_path)?;
    let archive = rawzip::ZipArchive::from_file(f, &mut buffer)?;

    let entries_len = archive.entries_hint().min(1_000_000);
    let mut entries = archive.entries(&mut buffer);

    // All the names of files concatenated together. Whereas a `Vec<String>`
    // would incur at least one allocation per element, we are able to amortize
    // the cost of storing names.
    let mut names = String::with_capacity(entries_len as usize * 52);
    let mut pos = Vec::with_capacity(entries_len as usize);

    while let Some(entry) = entries.next_entry()? {
        if entry.is_dir() {
            continue;
        }

        let name = entry.file_path().try_normalize()?;
        names.push_str(name.as_ref());
        let name_index = (names.len() - name.len())..names.len();
        pos.push((name_index, entry.wayfinder()));
    }

    thread::scope(|scope| {
        let archive = &archive;
        let names = &names;
        scope.spawn(move || {
            for (name_index, wayfinder) in pos {
                let entry = match archive.get_entry(wayfinder) {
                    Ok(entry) => entry,
                    Err(e) => {
                        if tx.send(Err(e).context("zip entry failed")).is_err() {
                            return;
                        }
                        continue;
                    }
                };

                let name = &names[name_index];
                let max_size = wayfinder
                    .compressed_size_hint()
                    .max(wayfinder.uncompressed_size_hint());
                if max_size > 20 * 1000 * 1000 {
                    let err = anyhow::anyhow!("{}: too large", name);
                    if tx.send(Err(err)).is_err() {
                        return;
                    }
                    continue;
                }

                let mut buf = if let Ok(mut existing_buf) = receive_buf.try_recv() {
                    existing_buf.resize(wayfinder.compressed_size_hint() as usize, 0);
                    existing_buf
                } else {
                    vec![0u8; wayfinder.compressed_size_hint() as usize]
                };

                let mut reader = entry.reader();
                let read_result = reader
                    .read_exact(&mut buf)
                    .with_context(|| format!("{}: read failed", name))
                    .and_then(|_| reader.claim_verifier().context("verifier failed"))
                    .map(|verifier| (name, buf, verifier));

                // If the other end hung up stop processing.
                if tx.send(read_result).is_err() {
                    return;
                }
            }
        });

        let data = rx.into_iter().par_bridge().map_init(
            || {
                // Each worker gets its own inflation buffer and decompressor
                (Vec::<u8>::new(), libdeflater::Decompressor::new())
            },
            |(inflated, decompressor), args| {
                let (name, raw, verification) = args?;
                inflated.resize(verification.size() as usize, 0);
                let inflation = decompressor.deflate_decompress(&raw, inflated)?;
                let _ = return_buf.send(raw);

                let crc = rawzip::crc32(&inflated[..inflation]);
                verification.valid(rawzip::ZipVerification {
                    crc,
                    uncompressed_size: inflation as u64,
                })?;

                let result =
                    parse_replay(opt, inflated).with_context(|| format!("{name}: FAILED"))?;

                Ok((name, result))
            },
        );

        if opt.dry_run {
            data.for_each(|result: anyhow::Result<_>| match result {
                Ok((name, _replay)) => println!("Parsed {}", name),
                Err(e) => eprintln!("Failed {:?}", e),
            })
        } else {
            data.for_each_with(
                Vec::with_capacity(50 * 1000 * 1000),
                |mut out, result| match result {
                    Ok((name, replay)) => {
                        out.clear();
                        let rep = RocketReplay {
                            file: &PathBuf::from(&name),
                            replay,
                        };

                        let replay_json = serde_json::to_writer(&mut out, &rep);
                        if let Err(e) = replay_json {
                            eprintln!("Could not serialize replay: {} {}", name, e);
                            return;
                        }
                        let mut lock = io::stdout().lock();
                        let _ = lock.write_all(out);
                        let _ = lock.write_all(b"\n");
                    }
                    Err(e) => eprintln!("Failed {:?}", e),
                },
            )
        }
    });

    Ok(())
}

fn run() -> anyhow::Result<()> {
    let opt = Opt::parse();

    let enter_zip_mode = opt
        .input
        .first()
        .is_some_and(|x| x.extension().and_then(|ext| ext.to_str()) == Some("zip"));
    if enter_zip_mode {
        zip(&opt.input[0], &opt)
    } else if opt.multiple {
        parse_multiple_replays(&opt)
    } else if opt.input.len() > 1 {
        bail!("Expected one input file when --multiple is not specified");
    } else {
        let replay = if opt.input.is_empty() {
            let mut d = Vec::new();
            io::stdin()
                .read_to_end(&mut d)
                .context("Could not read stdin")?;
            parse_replay(&opt, &d).context("Could not parse replay from stdin")
        } else {
            let file = &opt.input[0];
            let (_, replay) = read_file(&opt, file.clone())
                .with_context(|| format!("Unable to parse replay {}", file.display()))?;
            Ok(replay)
        }?;

        if !opt.dry_run {
            let stdout = io::stdout();
            let lock = stdout.lock();
            serialize(opt.pretty, BufWriter::new(lock), &replay)?;
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

        eprintln!("An error occurred: {:?}", e);
        ::std::process::exit(1);
    }
}
