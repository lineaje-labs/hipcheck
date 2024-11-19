// SPDX-License-Identifier: Apache-2.0

pub mod score;

use crate::{
	config::{AttacksConfigQuery, CommitConfigQuery, PracticesConfigQuery},
	data::git::GitProvider,
	error::Result,
	metric::{affiliation::AffiliatedType, MetricProvider},
	plugin::QueryResult,
	F64,
};
use std::{
	collections::{HashMap, HashSet},
	default::Default,
	ops::Not,
	sync::Arc,
};

/// Queries about analyses
#[salsa::query_group(AnalysisProviderStorage)]
pub trait AnalysisProvider:
	AttacksConfigQuery + CommitConfigQuery + GitProvider + MetricProvider + PracticesConfigQuery
{
	 
	/// Returns result of affiliation analysis
	fn affiliation_analysis(&self) -> Result<QueryResult>;

	/// Returns result of binary analysis
	fn binary_analysis(&self) -> Result<QueryResult>;

	/// Returns result of fuzz analysis
	fn fuzz_analysis(&self) -> Result<QueryResult>;

	/// Returns result of review analysis
	fn review_analysis(&self) -> Result<QueryResult>;
}


pub fn affiliation_analysis(db: &dyn AnalysisProvider) -> Result<QueryResult> {
	let results = db.affiliation_metric()?;

	let affiliated_iter = results
		.affiliations
		.iter()
		.filter(|a| a.affiliated_type.is_affiliated());

	// @Note - policy expr json injection can't handle objs/strings currently
	let value: Vec<bool> = affiliated_iter.clone().map(|_| true).collect();

	let mut contributor_freq_map = HashMap::new();

	for affiliation in affiliated_iter {
		let commit_view = db.contributors_for_commit(Arc::clone(&affiliation.commit))?;

		let contributor = match affiliation.affiliated_type {
			AffiliatedType::Author => String::from(&commit_view.author.name),
			AffiliatedType::Committer => String::from(&commit_view.committer.name),
			AffiliatedType::Neither => String::from("Neither"),
			AffiliatedType::Both => String::from("Both"),
		};

		let count_commits_for = |contributor| {
			db.commits_for_contributor(Arc::clone(contributor))
				.into_iter()
				.count() as i64
		};

		let author_commits = count_commits_for(&commit_view.author);
		let committer_commits = count_commits_for(&commit_view.committer);

		let commit_count = match affiliation.affiliated_type {
			AffiliatedType::Neither => 0,
			AffiliatedType::Both => author_commits + committer_commits,
			AffiliatedType::Author => author_commits,
			AffiliatedType::Committer => committer_commits,
		};

		// Add string representation of affiliated contributor with count of associated commits
		contributor_freq_map.insert(contributor, commit_count);
	}

	let concerns: Vec<String> = contributor_freq_map
		.into_iter()
		.map(|(contributor, count)| format!("Contributor {} has count {}", contributor, count))
		.collect();

	Ok(QueryResult {
		value: serde_json::to_value(value)?,
		concerns,
	})
}

pub fn binary_analysis(db: &dyn AnalysisProvider) -> Result<QueryResult> {
	let results = db.binary_metric()?;
	let concerns: Vec<String> = results
		.binary_files
		.iter()
		.map(|binary_file| format!("Binary file at '{}'", binary_file))
		.collect();
	Ok(QueryResult {
		value: serde_json::to_value(&results.binary_files)?,
		concerns,
	})
}

pub fn fuzz_analysis(db: &dyn AnalysisProvider) -> Result<QueryResult> {
	let results = db.fuzz_metric()?;
	let value = results.fuzz_result.exists;
	Ok(QueryResult {
		value: serde_json::to_value(value)?,
		concerns: vec![],
	})
}

pub fn review_analysis(db: &dyn AnalysisProvider) -> Result<QueryResult> {
	let results = db.review_metric()?;
	let num_flagged = results
		.pull_reviews
		.iter()
		.filter(|p| p.has_review.not())
		.count() as u64;
	let percent_flagged = match (num_flagged, results.pull_reviews.len()) {
		(flagged, total) if flagged != 0 && total != 0 => {
			num_flagged as f64 / results.pull_reviews.len() as f64
		}
		_ => 0.0,
	};
	let value = F64::new(percent_flagged).expect("Percent threshold should never be NaN");
	Ok(QueryResult {
		value: serde_json::to_value(value)?,
		concerns: vec![],
	})
}