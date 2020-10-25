use crate::model::RepoDeltas;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// returns a path pointing to he project.list file in
/// the .repo folder, or an io::Error in case the file
/// couldn't been found.
pub fn find_project_file() -> Result<PathBuf, io::Error> {
    let project_file = find_repo_folder()?.join("project.list");
    if project_file.is_file() {
        Ok(project_file)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "no project.list in .repo found",
        ))
    }
}

/// returns a path pointing to the .repo folder,
/// or io::Error in case the .repo folder couldn't been
/// found in the cwd or any of its parent folders.
pub fn find_repo_folder() -> Result<PathBuf, io::Error> {
    let base_folder = find_repo_base_folder()?;
    Ok(base_folder.join(".repo"))
}

/// returns a path pointing to the folder containing .repo,
/// or io::Error in case the .repo folder couldn't been
/// found in the cwd or any of its parent folders.
pub fn find_repo_base_folder() -> Result<PathBuf, io::Error> {
    let cwd = env::current_dir()?;
    for parent in cwd.ancestors() {
        for entry in fs::read_dir(&parent)? {
            let entry = entry?;
            if entry.path().is_dir() && entry.file_name() == ".repo" {
                return Ok(parent.to_path_buf());
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        "no .repo folder found",
    ))
}

/// executes an external executable with given arguments;
/// if the pattern "{}" is found in the args parameter, it
/// is replaced with the ID of the given commit
pub fn execute_on_repo(
    exec: &str,
    args: &str,
    status: &RepoDeltas,
) -> Result<std::process::Child, std::io::Error> {
    Command::new(exec)
        .current_dir(&status.repo.abs_path)
        .args(args.split(' '))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}
