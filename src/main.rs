/// # pathsearch 🕵
///
/// Enumerate all executables in `$PATH`. Unix-only.
///
/// Potential use cases:
///
/// * Pipe to fzf → Launch stuff from your terminal → profit
/// * Faster `dmenu_path` replacement
use anyhow::{anyhow, Result};
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::DirEntry;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::prelude::MetadataExt;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[cfg(unix)]
fn is_executable(entry: &DirEntry) -> bool {
    match entry.metadata() {
        // The access syscall is probably more accurate in edge cases with multiple users but much
        // slower
        Ok(meta) => meta.mode() & 0o111 != 0,
        Err(_) => false,
    }
}

#[cfg(not(unix))]
fn is_executable(entry: &DirEntry) -> bool {
    compile_error!("Only Unix systems are supported");
    unimplemented!();
}

fn executable_file_filter(entry: &DirEntry) -> Option<OsString> {
    if !entry.file_type().ok()?.is_dir() && is_executable(entry) {
        Some(entry.file_name())
    } else {
        None
    }
}

fn find_executables<T: AsRef<Path>>(path: T) -> Result<Vec<OsString>> {
    let entries = fs::read_dir(&path)?
        .flatten()
        .filter_map(|entry| executable_file_filter(&entry))
        .collect();
    Ok(entries)
}

fn main() -> Result<()> {
    let path = env::var_os("PATH").ok_or_else(|| anyhow!("PATH not set"))?;
    let paths: Vec<PathBuf> = env::split_paths(&path).collect();
    // Use BTreeSet for deduping + sorting
    let cmds: BTreeSet<OsString> = paths
        .into_par_iter()
        .flat_map(find_executables)
        .flatten()
        .collect();

    let mut stdout = std::io::stdout();
    for cmd in cmds {
        stdout.write_all(cmd.as_encoded_bytes())?;
        stdout.write_all(b"\n")?;
    }
    stdout.flush()?;

    Ok(())
}
