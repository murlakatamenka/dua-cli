extern crate failure;
extern crate failure_tools;
extern crate structopt;

use structopt::StructOpt;

use dua::{ByteFormat, Color};
use failure::Error;
use failure_tools::ok_or_exit;
use std::path::PathBuf;
use std::{fs, io, io::Write, process};

mod options;

fn run() -> Result<(), Error> {
    use options::Command::*;

    let opt: options::Args = options::Args::from_args();
    let stdout = io::stdout();
    let stdout_locked = stdout.lock();
    let walk_options = dua::WalkOptions {
        threads: opt.threads.unwrap_or(0),
        byte_format: opt.format.map(Into::into).unwrap_or(ByteFormat::Metric),
        color: if atty::is(atty::Stream::Stdout) {
            Color::Terminal
        } else {
            Color::None
        },
    };
    let (show_statistics, res) = match opt.command {
        Some(Aggregate {
            input,
            no_total,
            no_sort,
            statistics,
        }) => (
            statistics,
            dua::aggregate(
                stdout_locked,
                walk_options,
                !no_total,
                !no_sort,
                if input.len() == 0 {
                    cwd_dirlist()?
                } else {
                    input
                },
            )?,
        ),
        None => (
            false,
            dua::aggregate(
                stdout_locked,
                walk_options,
                true,
                true,
                if opt.input.len() == 0 {
                    cwd_dirlist()?
                } else {
                    opt.input
                },
            )?,
        ),
    };

    if show_statistics {
        writeln!(io::stderr(), "{:?}", res.stats).ok();
    }
    if res.num_errors > 0 {
        process::exit(1);
    }
    Ok(())
}

fn cwd_dirlist() -> Result<Vec<PathBuf>, io::Error> {
    let mut v: Vec<_> = fs::read_dir(".")?
        .filter_map(|e| {
            e.ok()
                .and_then(|e| e.path().strip_prefix(".").ok().map(ToOwned::to_owned))
        })
        .collect();
    v.sort();
    Ok(v)
}

fn main() {
    ok_or_exit(run())
}