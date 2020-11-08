use crate::model::{Repo, RepoBranchDeltas};
use std::env;
use std::fs;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;

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
    status: &RepoBranchDeltas,
) -> Result<std::process::Child, std::io::Error> {
    Command::new(exec)
        .current_dir(&status.repo.abs_path)
        .args(args.split(' '))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

/// parses a flat list of local repo paths
/// and creates a vector of Repos objects from it
pub fn repos_from(
    project_file: &std::fs::File,
    include_manifest: bool,
) -> Result<Vec<Arc<Repo>>, io::Error> {
    let mut repos = Vec::new();

    let base_folder = find_repo_base_folder()?;
    for project in BufReader::new(project_file).lines() {
        let rel_path = project.expect("project.list read error");
        repos.push(Arc::new(Repo::from(base_folder.join(&rel_path), rel_path)));
    }

    if include_manifest {
        let rel_path = String::from(".repo/manifests");
        repos.push(Arc::new(Repo::from(base_folder.join(&rel_path), rel_path)));
    }

    Ok(repos)
}

/// parses a flat list of local repo paths
/// and creates a vector of Strings of it
pub fn repos_paths_from(project_file: &std::fs::File) -> Result<Vec<String>, io::Error> {
    let mut repos = Vec::new();

    for project in BufReader::new(project_file).lines() {
        let rel_path = project.expect("project.list read error");
        repos.push(rel_path);
    }

    Ok(repos)
}
