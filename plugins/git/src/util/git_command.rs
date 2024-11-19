// SPDX-License-Identifier: Apache-2.0

use crate::data::*;
use anyhow::{Context, Result};
use git2::DiffFormat;
use git2::DiffLineType;
use git2::Repository;
use git2::Revwalk;
use git2::Sort;
use jiff::Timestamp;
use std::{convert::AsRef, path::Path};

fn get_repo_head<'a>(repo: &'a Repository) -> Result<Revwalk<'a>> {
	let mut revwalk = repo.revwalk().context("Unable to determine HEAD")?;
	// as of right now, we always want the commits sorted from newest to oldest
	revwalk
		.set_sorting(Sort::TIME)
		.context("Unable to set commit sorting")?;
	revwalk.push_head().context("Unable to push repo HEAD")?;
	Ok(revwalk)
}

/// Base function for getting all of the commits in a repo
fn get_commits_raw<P: AsRef<Path>>(
	// where the repo is
	repo: P,
	// only look for commits new than this timestamp, if not None
	since: Option<Timestamp>,
) -> Result<Vec<RawCommit>> {
	let repo = Repository::open(repo).context("Could not open repository")?;
	let revwalk = get_repo_head(&repo)?;

	// 5_000 seemed like a decent baseline to limit allocations needed for small/medium repos
	let mut raw_commits = Vec::with_capacity(5_000);
	for oid in revwalk {
		let oid = oid?;
		let commit = repo.find_commit(oid)?;

		let raw_commit = RawCommit::from(commit);

		match (&raw_commit.committed_on, &since) {
			(Ok(commit_time), Some(since)) => {
				if commit_time < since {
					break;
				}
			}
			_ => {}
		}

		raw_commits.push(raw_commit);
	}
	Ok(raw_commits)
}

/// retrieve all of the commits in a repos history
pub fn get_commits<P: AsRef<Path>>(repo: P) -> Result<Vec<RawCommit>> {
	get_commits_raw(repo, None)
}

pub fn get_commits_from_date<P>(repo: P, since: Timestamp) -> Result<Vec<RawCommit>>
where
	P: AsRef<Path>,
{
	get_commits_raw(repo, Some(since))
}

pub fn get_diffs<P: AsRef<Path>>(repo: P) -> Result<Vec<Diff>> {
	let repo = Repository::open(repo).context("Could not open repository")?;
	let revwalk = get_repo_head(&repo)?;

	// 10_000 seemed like a decent baseline to limit allocations for small/medium repos
	let mut diffs = Vec::with_capacity(10_000);

	// let mut previous_commit: Option<git2::Commit<'_>> = None;
	for oid in revwalk {
		let oid = oid?;
		let commit = repo.find_commit(oid)?;

		if let Some(previous_commit) = commit.parents().next() {
			let current_tree = commit
				.tree()
				.context("Could not determine tree for the current commit")?;
			let previous_tree = previous_commit
				.tree()
				.context("Could not determine tree for the previous commit")?;
			let diff = repo
				.diff_tree_to_tree(Some(&previous_tree), Some(&current_tree), None)
				.context("Could not diff current commit to previous commit")?;

			let stats = diff.stats().context("Could not determine stats for diff")?;

			let total_insertions_in_commit = stats.insertions();
			let total_deletions_in_commit = stats.deletions();

			// arbitrary pre-allocation to hold FileDiffs for this commit to reduce number of needed allocations
			let mut file_diffs: Vec<FileDiff> = Vec::with_capacity(128);
			// iterate over all of the patches in this commit to generate all of the FileDiffs for this commit
			diff.print(DiffFormat::Patch, |delta, _hunk, line| {
				if let Some(file_name) = delta.new_file().path() {
					let file_name = file_name.to_string_lossy();

					let file_diff: &mut FileDiff = match file_diffs
						.iter_mut()
						.find(|fd| fd.file_name.as_str() == file_name)
					{
						Some(file_diff) => file_diff,
						None => {
							file_diffs.push(FileDiff {
								file_name: file_name.to_string().into(),
								additions: None,
								deletions: None,
								patch: String::new(),
							});
							// unwrap is safe because we just pushed
							file_diffs.last_mut().unwrap()
						}
					};

					match line.origin_value() {
						DiffLineType::Addition => file_diff.increment_additions(1),
						DiffLineType::Deletion => file_diff.increment_deletions(1),
						_ => {}
					}
				}
				true
			})
			.context("Could not generate FileDiff for commit")?;

			let diff = Diff {
				additions: Some(total_insertions_in_commit as i64),
				deletions: Some(total_deletions_in_commit as i64),
				file_diffs,
			};
			diffs.push(diff);
		}
	}
	Ok(diffs)
}
