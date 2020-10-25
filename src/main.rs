extern crate app_dirs;
#[macro_use]
extern crate clap;
extern crate cursive;
extern crate indicatif;
extern crate num_cpus;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate toml;

mod config;
mod model;
mod styles;
mod ui;
mod utils;
mod views;

use clap::{App, Arg};
use model::{create_model, Repo};
use std::convert::Into;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;
use utils::{find_project_file, find_repo_base_folder};

const MAX_NUMBER_OF_THREADS: usize = 18; //tests on a 36 core INTEL Xeon showed that parsing becomes slower again if more than 18 threads are used

fn main() -> Result<(), String> {
    let original_cwd = env::current_dir().expect("cwd not found");
    let matches = App::new("oper-delta")
        .version(crate_version!())
        .author("Florian Bramer <elektronenhirn@gmail.com>")
        .about("git-repo diff tool for branches")
        .arg(
            Arg::with_name("cwd")
                .short("C")
                .long("cwd")
                .value_name("cwd")
                .help("change working directory (mostly useful for testing)")
                .default_value(original_cwd.to_str().unwrap())
                .takes_value(true),
        )
        .arg(
            Arg::with_name("branch")
                .value_name("branch")
                .help("one or multiple branches to diff current HEAD against")
                .takes_value(true)
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("ignore-consolidated")
                .short("c")
                .long("ignore-consolidated")
                .help("ignore repositories where the HEAD has been consolidated into the <branch>"),
        )
        .get_matches();

    let branches = matches.values_of("branch").unwrap().collect::<Vec<_>>();
    let cwd = Path::new(matches.value_of("cwd").unwrap());
    let ignore_consolidated = matches.is_present("ignore-consolidated");

    do_main(branches, cwd, ignore_consolidated).or_else(|e| Err(e.description().into()))
}

fn do_main(branches: Vec<&str>, cwd: &Path, ignore_consolidated: bool) -> Result<(), io::Error> {
    let config = config::read();

    env::set_current_dir(cwd)?;
    rayon::ThreadPoolBuilder::new()
        .num_threads(std::cmp::min(num_cpus::get(), MAX_NUMBER_OF_THREADS))
        .build_global()
        .unwrap();

    let project_file = File::open(find_project_file()?)?;
    let repos = repos_from(&project_file, false)?;

    let diff = create_model(repos, branches, ignore_consolidated)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.description()))?;

    ui::show(diff, &config);

    Ok(())
}

fn repos_from(
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
