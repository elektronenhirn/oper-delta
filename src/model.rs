use console::style;
use git2::{Branch, BranchType, Repository};
use indicatif::{MultiProgress, ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

/// representation of a local git repository
pub struct Repo {
    pub abs_path: PathBuf,
    pub rel_path: String,
    pub description: String,
}

// a qualitative difference between two branches
#[derive(Clone, Debug, PartialEq)]
pub enum Delta {
    ConsolidatedBySameCommit,
    ConsolidatedByMergeCommit,
    ConsolidatedByEqualContent,
    NotConsolidatedButFastForwardable,
    NotConsolidated,
    BranchNotFound,
}

#[derive(Clone, Debug)]
pub struct BranchDelta {
    pub branch_name: String,
    pub delta: Delta,
}

#[derive(Clone)]
pub struct RepoBranchDeltas {
    pub repo: Arc<Repo>,
    pub deltas: Vec<BranchDelta>,
}

pub struct Filter {
    pub include_consolidated_by_same_commit: bool,
    pub include_consolidated_by_merge_commit: bool,
    pub include_consolidated_by_equal_content: bool,
    pub include_non_consolidated: bool,
    pub include_non_consolidated_but_ff_able: bool,
    pub include_branch_not_found: bool,
}

impl fmt::Display for Delta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn create_model(
    repos: Vec<Arc<Repo>>,
    branches: Vec<&str>,
    filter: Filter,
) -> Result<Vec<RepoBranchDeltas>, git2::Error> {
    // setup progress bar
    let progress = MultiProgress::new();
    let progress_bars = (0..rayon::current_num_threads())
        .enumerate()
        .map(|(n, _)| {
            let pb = ProgressBar::hidden();
            pb.set_prefix(&n.to_string());
            pb.set_style(
                ProgressStyle::default_spinner().template("[{prefix}] {wide_msg:.bold.dim}"),
            );
            progress.add(pb)
        })
        .collect::<Vec<ProgressBar>>();
    let overall_progress = ProgressBar::new(repos.len() as u64);
    overall_progress.set_style(
        ProgressStyle::default_bar()
            .template(" {spinner:.bold.cyan}  Scanned {pos} of {len} repositories"),
    );
    let overall_progress = progress.add(overall_progress);
    thread::spawn(move || {
        progress.join_and_clear().unwrap();
    });

    //now create the model
    let repo_branch_deltas: Vec<RepoBranchDeltas> = repos
        .par_iter()
        .map(move |repo| {
            let progress_bar = &progress_bars[rayon::current_thread_index()?];
            progress_bar.set_message(&format!("Scanning {}", repo.rel_path));

            let progress_error = |msg: &str, error: &dyn std::error::Error| {
                progress_bar.println(format!(
                    "{}: {}: {}",
                    style(&msg).red(),
                    style(&repo.rel_path).blue(),
                    error
                ));
                progress_bar.inc(1);
                progress_bar.set_message("Idle");
            };

            calc_branch_deltas_for_a_single_repo(repo, &branches, &filter).map_or_else(
                |e| {
                    progress_error("Failed to open", &e);
                    None
                },
                |x| {
                    progress_bar.set_message("Idle");
                    x
                },
            )
        })
        .progress_with(overall_progress)
        .filter_map(|x| x)
        .collect();

    Ok(repo_branch_deltas)
}

fn calc_branch_deltas_for_a_single_repo(
    repo: &std::sync::Arc<Repo>,
    branches: &Vec<&str>,
    filter: &Filter,
) -> Result<Option<RepoBranchDeltas>, git2::Error> {
    let git_repo = Repository::open(&repo.abs_path)?;

    let deltas = branches
        .iter()
        .filter_map(|branch_name| {
            let git_repo_ref = &git_repo;
            let local_branch = git_repo_ref.find_branch(branch_name, BranchType::Local);
            let branch = if local_branch.is_err() {
                git_repo_ref.find_branch(branch_name, BranchType::Remote)
            } else {
                local_branch
            };

            let delta = match branch {
                Ok(branch) => {
                    let mut delta = Delta::NotConsolidated;
                    if consolidated_by_same_commit(git_repo_ref, &branch) {
                        delta = Delta::ConsolidatedBySameCommit;
                    } else if consolidated_by_merge(git_repo_ref, &branch) {
                        delta = Delta::ConsolidatedByMergeCommit;
                    } else if consolidated_by_equal_content(git_repo_ref, &branch) {
                        delta = Delta::ConsolidatedByEqualContent;
                    } else if fast_forwardable(git_repo_ref, &branch) {
                        delta = Delta::NotConsolidatedButFastForwardable;
                    }
                    delta
                }
                Err(_err) => Delta::BranchNotFound,
            };

            Some(BranchDelta {
                branch_name: String::from(*branch_name),
                delta,
            })
        })
        .collect::<Vec<_>>();

    //apply filter from the command line
    let include_repo = if filter.include_consolidated_by_same_commit
        && deltas
            .iter()
            .any(|x| x.delta == Delta::ConsolidatedBySameCommit)
    {
        true
    } else if filter.include_consolidated_by_merge_commit
        && deltas
            .iter()
            .any(|x| x.delta == Delta::ConsolidatedByMergeCommit)
    {
        true
    } else if filter.include_consolidated_by_equal_content
        && deltas
            .iter()
            .any(|x| x.delta == Delta::ConsolidatedByEqualContent)
    {
        true
    } else if filter.include_non_consolidated_but_ff_able
        && deltas
            .iter()
            .any(|x| x.delta == Delta::NotConsolidatedButFastForwardable)
    {
        true
    } else if filter.include_non_consolidated
        && deltas.iter().any(|x| x.delta == Delta::NotConsolidated)
    {
        true
    } else if filter.include_branch_not_found
        && deltas.iter().all(|x| x.delta == Delta::BranchNotFound)
    {
        true
    } else {
        false
    };

    if include_repo {
        Ok(Some(RepoBranchDeltas {
            repo: repo.clone(),
            deltas,
        }))
    } else {
        Ok(None)
    }
}

fn consolidated_by_same_commit(git_repo: &Repository, branch: &Branch) -> bool {
    let head_as_obj = git_repo
        .head()
        .expect("No HEAD for git repo")
        .peel(git2::ObjectType::Commit)
        .unwrap();

    let branch_as_obj = branch.get().peel(git2::ObjectType::Commit).unwrap();
    head_as_obj.id() == branch_as_obj.id()
}

fn consolidated_by_merge(git_repo: &Repository, branch: &Branch) -> bool {
    //walk down the history of "branch" and probe for a commit which has HEAD as a parent
    let branch_as_obj = branch.get().peel(git2::ObjectType::Commit).unwrap();
    let mut revwalk = git_repo.revwalk().expect("Failed to create revwalk");

    revwalk
        .push(branch_as_obj.id())
        .expect("branch not found in revwalk");
    revwalk.simplify_first_parent();
    revwalk.set_sorting(git2::Sort::TIME);

    let head_as_obj = git_repo
        .head()
        .expect("No HEAD for git repo")
        .peel(git2::ObjectType::Commit)
        .unwrap();

    for commit_id in revwalk {
        let commit = commit_id
            .and_then(|commit_id| git_repo.find_commit(commit_id))
            .expect("Failed to find commit");
        if commit.parent_ids().any(|x| x == head_as_obj.id()) {
            return true;
        }
    }

    false
}

fn consolidated_by_equal_content(git_repo: &Repository, branch: &Branch) -> bool {
    let head_as_obj = git_repo
        .head()
        .expect("No HEAD for git repo")
        .peel(git2::ObjectType::Tree)
        .unwrap();
    let branch_as_obj = branch.get().peel(git2::ObjectType::Tree).unwrap();

    git_repo
        .diff_tree_to_tree(head_as_obj.as_tree(), branch_as_obj.as_tree(), None)
        .unwrap()
        .deltas()
        .count()
        == 0
}

fn fast_forwardable(git_repo: &Repository, branch: &Branch) -> bool {
    //our "branch" can be ff if the history of HEAD contains the tip of "branch"

    let mut revwalk = git_repo.revwalk().expect("Failed to create revwalk");

    revwalk.push_head().expect("No head found");
    revwalk.simplify_first_parent();
    revwalk.set_sorting(git2::Sort::TIME);

    let branch_as_obj = branch.get().peel(git2::ObjectType::Tree).unwrap();

    for commit_id in revwalk {
        if commit_id.unwrap() == branch_as_obj.id() {
            return true;
        }
    }

    false
}

impl Repo {
    pub fn from(abs_path: PathBuf, rel_path: String) -> Repo {
        let description = abs_path.file_name().unwrap().to_str().unwrap().into();
        Repo {
            abs_path,
            rel_path,
            description,
        }
    }
}
