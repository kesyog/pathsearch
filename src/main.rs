/// # pathsearch 🕵
///
/// Enumerate all executables in `$PATH`. Unix-only.
///
/// Potential use cases:
///
/// * Pipe to fzf → Launch stuff from your terminal → profit
/// * Faster dmenu_path replacement
use anyhow::{anyhow, Result};
use nix::unistd::{self, AccessFlags};
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn is_executable(entry: DirEntry) -> Option<OsString> {
    if !entry.file_type().ok()?.is_dir() && unistd::access(&entry.path(), AccessFlags::X_OK).is_ok()
    {
        Some(entry.file_name())
    } else {
        None
    }
}

fn find_executables<T: AsRef<Path>>(path: T) -> Result<Vec<OsString>> {
    let entries = fs::read_dir(&path)?
        .flatten()
        .filter_map(is_executable)
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

    for cmd in cmds {
        if let Some(cmd) = cmd.to_str() {
            println!("{}", cmd);
        }
    }

    Ok(())
}
