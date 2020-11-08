use crate::model::{Delta, RepoBranchDeltas};
use crate::styles::{BLUE, GREEN, RED, WHITE, YELLOW};
use crate::views::ListView;
use cursive::theme::ColorStyle;
use cursive::view::ViewWrapper;

pub struct DeltaView {
    list_view: ListView,
    repo_deltas: Option<RepoBranchDeltas>,
}

impl DeltaView {
    pub fn empty() -> Self {
        DeltaView {
            list_view: ListView::new(),
            repo_deltas: None,
        }
    }

    fn reset (self: &mut Self) {
        self.list_view = ListView::new();
    }

    fn append_string(self: &mut Self, s: String) {
        self.list_view.insert_string(s);
    }

    fn append_colorful_string(self: &mut Self, s: String, c: ColorStyle) {
        self.list_view.insert_colorful_string(s, c);
    }

    #[rustfmt::skip]
    pub fn set_repo_deltas(self: &mut Self, repo_deltas: &RepoBranchDeltas) {
        self.repo_deltas = Some(repo_deltas.clone());

        self.reset();

        self.append_colorful_string(format!("{:30} {}", "git repo", repo_deltas.repo.rel_path), *WHITE);
        self.append_string(String::new());

//Summary
        self.append_colorful_string(String::from("Summary:"), *WHITE);
        self.append_string(String::new());
        self.append_colorful_string(format!("{:30}   {}", "Target Branch", "Delta"), *WHITE);
        self.append_colorful_string("===============================|==================================".to_string(), *WHITE);
        for branch_delta in repo_deltas.deltas.iter() {
            self.append_colorful_string(format!("{:30}   {}", branch_delta.branch_name.to_string(), Self::delta_to_string(&branch_delta.delta)), Self::delta_to_color(&branch_delta.delta));
        }
        self.append_string(String::new());

//Details
        self.append_colorful_string(String::from("Details:"), *WHITE);
        self.append_string(String::new());
        for branch_delta in repo_deltas.deltas.iter() {
            if branch_delta.delta == Delta::BranchNotFound{
                continue;
            }

            self.append_colorful_string(format!("{}", branch_delta.branch_name.to_string()), *WHITE);
            self.append_colorful_string(String::from("==============================="), *WHITE);
            self.append_colorful_string(format!("{}", Self::delta_to_string(&branch_delta.delta)), Self::delta_to_color(&branch_delta.delta));
            self.append_string(String::from("Distance from merge-base:"));
            self.append_string(format!("  HEAD: {}", match &branch_delta.distance_head_to_merge_base {
                Ok(v) => {
                    format!("{} commits", v.to_string())
                },
                Err(e) => e.clone()
            }));
            self.append_string(format!("  {}: {}",  branch_delta.branch_name.to_string(), match &branch_delta.distance_target_to_merge_base {
                Ok(v) => {
                    format!("{} commits", v.to_string())
                },
                Err(e) => e.clone()
            }));

            self.append_string(String::new());
        }

    }

    pub fn repo_deltas(self: &Self) -> &Option<RepoBranchDeltas> {
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
