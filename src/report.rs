use crate::model::{Delta, RepoBranchDeltas};
use std::io;
use std::path::Path;
use std::vec::Vec;

pub fn generate(model: Vec<RepoBranchDeltas>, output_file_path: String) -> Result<(), io::Error> {
    if model.len() == 0 {
        return Ok(());
    }

    let mut wtr = csv::Writer::from_path(Path::new(&output_file_path))?;

    wtr.write_field("Local Path of Repo")?;
    for branch in &model[0].deltas {
        wtr.write_field(format!("{} Branch: Delta", &branch.branch_name))?;
        wtr.write_field(format!(
            "{} Branch: Distance of HEAD to merge-base",
            &branch.branch_name
        ))?;
        wtr.write_field(format!(
            "{} Branch: Distance of {} to merge-base",
            &branch.branch_name, &branch.branch_name
        ))?;
    }
    wtr.write_record(None::<&[u8]>)?;

    for repo in &model {
        wtr.write_field(&repo.repo.rel_path)?;
        for branch in &repo.deltas {
            wtr.write_field(&delta_to_string(&branch.delta))?;
            wtr.write_field(&distance_to_string(&branch.distance_head_to_merge_base))?;
            wtr.write_field(&distance_to_string(&branch.distance_target_to_merge_base))?;
        }
        wtr.write_record(None::<&[u8]>)?;
    }

    wtr.flush()?;

    println!(
        "Wrote {} records as comma-separated-values to {}",
        model.len(),
        output_file_path
    );
    Ok(())
}

fn distance_to_string(distance: &Result<u32, String>) -> String {
    match distance {
        Ok(v) => v.to_string(),
        Err(e) => e.clone(),
    }
}

fn delta_to_string(delta: &Delta) -> String {
    match delta {
        Delta::ConsolidatedBySameCommit => "HEAD consolidated: points to the same commit as HEAD",
        Delta::ConsolidatedByMergeCommit => "HEAD consolidated: contains merge commit from HEAD",
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
