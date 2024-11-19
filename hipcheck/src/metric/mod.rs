// SPDX-License-Identifier: Apache-2.0

pub mod affiliation;
pub mod binary;
pub mod binary_detector;
pub mod commit_trust;
pub mod contributor_trust;
pub mod fuzz;
pub mod identity;
pub mod linguist;
mod math;
pub mod review;

use crate::{
	config::{AttacksConfigQuery, CommitConfigQuery},
	data::{git::GitProvider, DependenciesProvider, FuzzProvider, PullRequestReviewProvider},
	error::Result,
	metric::{
		affiliation::AffiliationOutput, binary::BinaryOutput,
		binary_detector::BinaryFile, commit_trust::CommitTrustOutput,
		contributor_trust::ContributorTrustOutput, fuzz::FuzzOutput,
		identity::IdentityOutput, linguist::Linguist, review::ReviewOutput,
	},
};
use std::sync::Arc;

/// Queries about metrics
#[salsa::query_group(MetricProviderStorage)]
pub trait MetricProvider:
	AttacksConfigQuery
	+ BinaryFile
	+ CommitConfigQuery
	+ DependenciesProvider
	+ GitProvider
	+ Linguist
	+ FuzzProvider
	+ PullRequestReviewProvider
{
	
	/// Returns result of affiliation metric
	#[salsa::invoke(affiliation::affiliation_metric)]
	fn affiliation_metric(&self) -> Result<Arc<AffiliationOutput>>;

	/// Returns result of binary metric
	#[salsa::invoke(binary::binary_metric)]
	fn binary_metric(&self) -> Result<Arc<BinaryOutput>>;

	/// Returns result of contributor trust metric
	#[salsa::invoke(commit_trust::commit_trust_metric)]
	fn commit_trust_metric(&self) -> Result<Arc<CommitTrustOutput>>;

	/// Returns result of contributor trust metric
	#[salsa::invoke(contributor_trust::contributor_trust_metric)]
	fn contributor_trust_metric(&self) -> Result<Arc<ContributorTrustOutput>>;

	/// Returns result of identity metric
	#[salsa::invoke(identity::identity_metric)]
	fn identity_metric(&self) -> Result<Arc<IdentityOutput>>;

	/// Returns result of fuzz metric
	#[salsa::invoke(fuzz::fuzz_metric)]
	fn fuzz_metric(&self) -> Result<Arc<FuzzOutput>>;

	/// Returns result of review metric
	#[salsa::invoke(review::review_metric)]
	fn review_metric(&self) -> Result<Arc<ReviewOutput>>;

}