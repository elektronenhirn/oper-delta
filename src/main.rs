extern crate app_dirs;
#[macro_use]
extern crate clap;
extern crate cursive;
extern crate indicatif;
extern crate num_cpus;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate spsheet;
extern crate toml;

mod config;
mod manifest;
mod model;
mod report;
mod styles;
mod ui;
mod utils;
mod views;

use anyhow::Result;
use clap::{App, Arg};
use model::{create_model, Filter};
use std::env;
use std::fs::File;
use std::path::Path;
use utils::{find_project_file, repos_from};

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
            Arg::with_name("repo-ignore-list")
                .long("repo-ignore-list")
                .value_name("path")
                .help("path to file which contains list of repos to ignore")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("report")
            .long("report")
            .value_name("file")
            .help("writes a report to a file given by <path> - supported formats: .csv, .ods, .xlsx")
            .takes_value(true)
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
            Arg::with_name("hide-consolidated-by-same-commit")
                .long("hide-consolidated-by-same-commit")
                .help("hide repositories where HEAD is pointing to the tip of the given <branch>"),
        )
        .arg(
            Arg::with_name("hide-consolidated-by-merge-commit")
                .long("hide-consolidated-by-merge-commit")
                .help("hide repositories where the HEAD has been consolidated into given <branch>"),
        )
        .arg(
            Arg::with_name("hide-consolidated-by-equal-content")
                .long("hide-consolidated-by-equal-content")
                .help("hide repositories where the HEAD and <branch> have equal content but are not related by history"),
        )
        .arg(
            Arg::with_name("hide-non-consolidated")
                .long("hide-non-consolidated")
                .help("hide repositories where the HEAD has been not consolidated into given <branch>"),
        )
        .arg(
            Arg::with_name("hide-non-consolidated-but-ff-able")
                .long("hide-non-consolidated-but-ff-able")
                .help("hide repositories where the HEAD has been not consolidated into given <branch> but can be fastforwarded on <branch>"),
        )
        .arg(
            Arg::with_name("hide-branch-not-found")
                .long("hide-branch-not-found")
                .help("hide repositories where the given <branch> couldn't been found"),
        )
        .arg(
            Arg::with_name("manifest")
                .short("m")
                .long("manifest")
                .takes_value(true)
                .help("filter list of repositories by the given manifest"),
        )
        .get_matches();

    let branches = matches.values_of("branch").unwrap().collect::<Vec<_>>();
    let cwd = Path::new(matches.value_of("cwd").unwrap());
    let filter = Filter {
        include_consolidated_by_same_commit: !matches
            .is_present("hide-consolidated-by-same-commit"),
        include_consolidated_by_merge_commit: !matches
            .is_present("hide-consolidated-by-merge-commit"),
        include_consolidated_by_equal_content: !matches
            .is_present("hide-consolidated-by-equal-content"),
        include_non_consolidated: !matches.is_present("hide-non-consolidated"),
        include_non_consolidated_but_ff_able: !matches
            .is_present("hide-non-consolidated-but-ff-able"),
        include_branch_not_found: !matches.is_present("hide-branch-not-found"),
        repo_ignore_list: matches.value_of("repo-ignore-list").map(|x| x.to_string()),
    };
    let report_file_path = matches.value_of("report").map(|x| x.to_string());
    let filter_by_manifest = matches.value_of("manifest");

    do_main(branches, cwd, filter, report_file_path, filter_by_manifest).map_err(|e| e.to_string())
}

fn do_main(
    branches: Vec<&str>,
    cwd: &Path,
    filter: Filter,
    report_file_path: Option<String>,
    filter_by_manifest: Option<&str>
) -> Result<()> {
    let config = config::read();

    env::set_current_dir(cwd)?;
    rayon::ThreadPoolBuilder::new()
        .num_threads(std::cmp::min(num_cpus::get(), MAX_NUMBER_OF_THREADS))
        .build_global()
        .unwrap();

    let project_file = File::open(find_project_file()?)?;
    let mut repos = repos_from(&project_file, false)?;
    if let Some(manifest_file) = filter_by_manifest {
        let manifest = manifest::parse(Path::new(&manifest_file))?;
        repos.retain(|repo| manifest.projects.iter().find(|&p| repo.rel_path == p.path ).is_some());
    }
    let nr_of_total_repos = repos.len();

    let model = create_model(repos, branches, filter)?;

    //TUI or report?
    match report_file_path {
        None => ui::show(model, &config, nr_of_total_repos),
        Some(file) => {
            println!("Skipping UI - generating report...");
            report::generate(model, &file)?
        }
    }

    Ok(())
}
