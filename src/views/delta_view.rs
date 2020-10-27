use crate::model::{Delta, RepoDeltas};
use crate::styles::{BLUE, GREEN, RED, WHITE, YELLOW};
use crate::views::ListView;
use cursive::theme::ColorStyle;
use cursive::view::ViewWrapper;

pub struct DeltaView {
    list_view: ListView,
    repo_deltas: Option<RepoDeltas>,
}

impl DeltaView {
    pub fn empty() -> Self {
        DeltaView {
            list_view: ListView::new(),
            repo_deltas: None,
        }
    }

    #[rustfmt::skip]
    pub fn set_repo_deltas(self: &mut Self, repo_deltas: &RepoDeltas) {
        self.repo_deltas = Some(repo_deltas.clone());

        self.list_view = ListView::new();
        self.list_view.insert_colorful_string(format!("{:30} {}", "git repo", repo_deltas.repo.rel_path), *WHITE);
        self.list_view.insert_string(String::new());
        self.list_view.insert_colorful_string(format!("{:30}   {}", "Target Branch", "Delta"), *WHITE);
        self.list_view.insert_colorful_string("===============================|==================================".to_string(), *WHITE);
        for branch_delta in repo_deltas.deltas.iter() {
            self.list_view.insert_colorful_string(format!("{:30}   {}", branch_delta.branch_name.to_string(), Self::delta_to_string(&branch_delta.delta)), Self::delta_to_color(&branch_delta.delta));
        }
    }

    pub fn repo_deltas(self: &Self) -> &Option<RepoDeltas> {
        &self.repo_deltas
    }

    fn delta_to_color(delta: &Delta) -> ColorStyle {
        match delta {
            Delta::ConsolidatedBySameCommit => *GREEN,
            Delta::ConsolidatedByMergeCommit => *GREEN,
            Delta::ConsolidatedByEqualContent => *GREEN,
            Delta::NotConsolidatedButFastForwardable => *YELLOW,
            Delta::NotConsolidated => *RED,
            Delta::BranchNotFound => *BLUE,
        }
    }

    fn delta_to_string(delta: &Delta) -> String {
        match delta {
            Delta::ConsolidatedBySameCommit => {
                "HEAD consolidated: points to the same commit as HEAD"
            }
            Delta::ConsolidatedByMergeCommit => {
                "HEAD consolidated: contains merge commit from HEAD"
            }
            Delta::ConsolidatedByEqualContent => {
                "HEAD consolidated: content same as HEAD (however history differs)"
            }
            Delta::NotConsolidatedButFastForwardable => {
                "HEAD not consolidated: can be fast forwarded to HEAD"
            }
            Delta::NotConsolidated => "HEAD not consolidated: and not fast forwardable",
            Delta::BranchNotFound => "branch not found",
        }
        .to_string()
    }
}

impl ViewWrapper for DeltaView {
    type V = ListView;

    fn with_view<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Self::V) -> R,
    {
        Some(f(&self.list_view))
    }

    fn with_view_mut<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Self::V) -> R,
    {
        Some(f(&mut self.list_view))
    }
}
