/// Enumerate all executables on PATH
///
/// Potential use cases:
/// * Pipe output to fzf and have an executable runner
/// * dmenu_path replacement
use anyhow::{anyhow, Result};
use rayon::prelude::*;
use std::collections::HashSet;
use std::ffi::OsString;
use std::os::unix::prelude::*;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn get_executables<T: AsRef<Path>>(path: T) -> Result<Vec<OsString>> {
    // TODO: check if this handles symbolic links properly
    // TODO: use jwalk crate for better parallelism
    let entries = fs::read_dir(&path)?
        .filter_map(|entry| match entry {
            Ok(entry) => {
                if entry.file_type().unwrap().is_file()
                    // Check if the owner has exec permission
                    // TODO: check if current user has exec permission
                    // TODO: check if this is reason for difference vs. dmenu_path
                    && entry.metadata().unwrap().mode() & 0o200 != 0
                {
                    Some(entry.file_name())
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect();
    Ok(entries)
}

fn main() -> Result<()> {
    let path = env::var_os("PATH").ok_or_else(|| anyhow!("PATH not set"))?;
    let paths: Vec<PathBuf> = env::split_paths(&path).collect();
    let mut cmds: Vec<OsString> = paths
        .into_par_iter()
        .flat_map(get_executables)
        .flatten()
        // Dedupe
        .collect::<HashSet<OsString>>()
        .drain()
        .collect::<Vec<OsString>>();
    cmds.par_sort_unstable();

    for cmd in cmds {
        if let Some(cmd) = cmd.to_str() {
            println!("{}", cmd);
        }
    }

    Ok(())
}
