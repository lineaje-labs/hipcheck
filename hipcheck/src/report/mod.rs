// SPDX-License-Identifier: Apache-2.0

// A report encapsulates the results of a run of Hipcheck, specifically containing:
//
// 1. The successes (which analyses passed, with user-friendly explanations of what's good)
// 2. The concerns (which analyses failed, and _why_)
// 3. The recommendation (pass or investigate)

// The report serves double-duty, because it's both the thing used to print user-friendly
// results on the CLI, and the type that's serialized out to JSON for machine-friendly output.

pub mod report_builder;

use crate::{
	cli::Format,
	error::{Context, Error, Result},
	policy_exprs::Executor,
	version::VersionQuery,
};
use chrono::prelude::*;
use schemars::JsonSchema;
use serde::{Serialize, Serializer};
use std::{
	default::Default,
	fmt,
	fmt::{Display, Formatter},
	iter::Iterator,
	ops::Not as _,
	result::Result as StdResult,
	sync::Arc,
};

/// The report output to the user.
#[derive(Debug, Serialize, JsonSchema)]
#[schemars(crate = "schemars")]
pub struct Report {
	/// The name of the repository being analyzed.
	pub repo_name: Arc<String>,

	/// The HEAD commit hash of the repository during analysis.
	pub repo_head: Arc<String>,

	/// The version of Hipcheck used to analyze the repo.
	pub hipcheck_version: String,

	/// When the analysis was performed.
	pub analyzed_at: Timestamp,

	/// What analyses passed.
	pub passing: Vec<PassingAnalysis>,

	/// What analyses did _not_ pass, and why.
	pub failing: Vec<FailingAnalysis>,

	/// What analyses errored out, and why.
	pub errored: Vec<ErroredAnalysis>,

	/// The final recommendation to the user.
	pub recommendation: Recommendation,
}

impl Report {
	/// Get the repository that was analyzed.
	pub fn analyzed(&self) -> String {
		format!("{} ({})", self.repo_name, self.repo_head)
	}

	/// Get the version of Hipcheck used for the analysis.
	pub fn using(&self) -> String {
		format!("using Hipcheck {}", self.hipcheck_version)
	}

	// Get the time that the analysis occured.
	pub fn at_time(&self) -> String {
		format!("on {}", self.analyzed_at)
	}

	/// Check if there are passing analyses.
	pub fn has_passing_analyses(&self) -> bool {
		self.passing.is_empty().not()
	}

	/// Check if there are failing analyses.
	pub fn has_failing_analyses(&self) -> bool {
		self.failing.is_empty().not()
	}

	/// Check if there are errored analyses.
	pub fn has_errored_analyses(&self) -> bool {
		self.errored.is_empty().not()
	}

	/// Get an iterator over all passing analyses.
	pub fn passing_analyses(&self) -> impl Iterator<Item = &Analysis> {
		self.passing.iter().map(|a| &a.0)
	}

	/// Get an iterator over all failing analyses.
	pub fn failing_analyses(&self) -> impl Iterator<Item = &FailingAnalysis> {
		self.failing.iter()
	}

	/// Get an iterator over all errored analyses.
	pub fn errored_analyses(&self) -> impl Iterator<Item = &ErroredAnalysis> {
		self.errored.iter()
	}

	/// Get the final recommendation.
	pub fn recommendation(&self) -> &Recommendation {
		&self.recommendation
	}
}

/// An analysis which passed.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(crate = "schemars")]
pub struct PassingAnalysis(
	/// The analysis which passed.
	Analysis,
);

impl PassingAnalysis {
	pub fn new(analysis: Analysis) -> PassingAnalysis {
		PassingAnalysis(analysis)
	}
}

/// An analysis which failed, including potential specific concerns.
#[derive(Debug, Serialize, JsonSchema)]
#[schemars(crate = "schemars")]
pub struct FailingAnalysis {
	/// The analysis.
	#[serde(flatten)]
	analysis: Analysis,

	/// Any concerns the analysis identified.
	#[serde(skip_serializing_if = "no_concerns")]
	concerns: Vec<String>,
}

impl FailingAnalysis {
	/// Construct a new failing analysis, verifying that concerns are appropriate.
	pub fn new(analysis: Analysis, concerns: Vec<String>) -> Result<FailingAnalysis> {
		Ok(FailingAnalysis { analysis, concerns })
	}

	pub fn analysis(&self) -> &Analysis {
		&self.analysis
	}

	pub fn concerns(&self) -> impl Iterator<Item = &String> {
		self.concerns.iter()
	}
}

/// Is the concern list empty?
///
/// This is a helper function for serialization of `FailedAnalysis`.
fn no_concerns(concerns: &[String]) -> bool {
	concerns.is_empty()
}

/// An analysis that did _not_ succeed.
#[derive(Debug, Serialize, JsonSchema)]
#[schemars(crate = "schemars")]
pub struct ErroredAnalysis {
	analysis: AnalysisIdent,
	error: ErrorReport,
}

impl ErroredAnalysis {
	/// Construct a new `ErroredAnalysis`.
	pub fn new(analysis: AnalysisIdent, error: &Error) -> Self {
		ErroredAnalysis {
			analysis,
			error: ErrorReport::from(error),
		}
	}

	pub fn top_msg(&self) -> String {
		format!("{} analysis error: {}", self.analysis, self.error.msg)
	}

	pub fn source_msgs(&self) -> Vec<String> {
		let mut msgs = Vec::new();

		try_add_msg(&mut msgs, &self.error.source);

		msgs
	}
}

fn try_add_msg(msgs: &mut Vec<String>, error_report: &Option<Box<ErrorReport>>) {
	if let Some(error_report) = error_report {
		msgs.push(error_report.msg.clone());
		try_add_msg(msgs, &error_report.source);
	}
}

/// The name of the analyses.
#[derive(Debug, Serialize, JsonSchema)]
#[schemars(crate = "schemars")]
pub struct AnalysisIdent(String);

impl Display for AnalysisIdent {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// A simple, serializable version of `Error`.
#[derive(Debug, Serialize, JsonSchema)]
#[schemars(crate = "schemars")]
pub struct ErrorReport {
	msg: String,
	#[serde(skip_serializing_if = "source_is_none")]
	source: Option<Box<ErrorReport>>,
}

fn source_is_none(source: &Option<Box<ErrorReport>>) -> bool {
	source.is_none()
}

impl From<&Error> for ErrorReport {
	fn from(error: &Error) -> ErrorReport {
		log::trace!("detailed error for report [error: {:#?}]", error);

		let mut errors = error
			.chain()
			// This `collect` is needed because `hc_error::Chain` isn't a
			// double-ended iterator, so it can't be reversed without
			// first collecting into an intermediate container.
			.collect::<Vec<_>>()
			.into_iter()
			.rev();

		let mut report = ErrorReport {
			// SAFETY: We're always guaranteed a minimum of one error
			// message, so this is safe.
			msg: errors.next().unwrap().to_string(),
			source: None,
		};

		for error in errors {
			report = ErrorReport {
				msg: error.to_string(),
				source: Some(Box::new(report)),
			};
		}

		report
	}
}

impl From<&(dyn std::error::Error + 'static)> for ErrorReport {
	fn from(error: &(dyn std::error::Error + 'static)) -> ErrorReport {
		let msg = error.to_string();
		let source = error
			.source()
			.map(|error| Box::new(ErrorReport::from(error)));
		ErrorReport { msg, source }
	}
}

/// An analysis, with score and threshold.
#[derive(Debug, Serialize, JsonSchema, Clone)]
#[serde(tag = "analysis")]
#[schemars(crate = "schemars")]
pub struct Analysis {
	/// The name of the plugin.
	name: String,

	/// If the analysis is passing or not.
	///
	/// Same as with the message, this is computed eagerly in the case of
	/// plugin analyses.
	passed: bool,

	/// The policy expression used for the plugin.
	///
	/// We use this when printing the result to help explain to the user
	/// *why* an analysis failed.
	policy_expr: String,

	/// The default query explanation pulled from RPC with the plugin.
	message: String,
}

impl Analysis {
	pub fn plugin(name: String, passed: bool, policy_expr: String, message: String) -> Self {
		Analysis {
			name,
			passed,
			policy_expr,
			message,
		}
	}

	pub fn is_passing(&self) -> bool {
		self.passed
	}

	pub fn statement(&self) -> String {
		if self.is_passing() {
			format!("'{}' passed, {}", self.name, self.policy_expr)
		} else {
			format!("'{}' failed, {}", self.name, self.policy_expr)
		}
	}

	pub fn explanation(&self) -> String {
		self.message.clone()
	}
}

/// Value and threshold for counting-based analyses.
#[derive(Debug, Serialize, JsonSchema, Clone)]
#[schemars(crate = "schemars")]
pub struct Count {
	value: u64,
	policy: String,
}

/// Value for binary-based analyses.
#[derive(Debug, Serialize, JsonSchema, Clone)]
#[schemars(crate = "schemars")]
pub struct Exists {
	value: bool,
	policy: String,
}

/// Value and threshold for percentage-based analyses.
#[derive(Debug, Serialize, JsonSchema, Clone)]
#[schemars(crate = "schemars")]
pub struct Percent {
	value: f64,
	policy: String,
}

/// A final recommendation of whether to use or investigate a piece of software,
/// including the risk threshold associated with that decision.
#[derive(Debug, Serialize, JsonSchema, Clone)]
#[schemars(crate = "schemars")]
pub struct Recommendation {
	pub kind: RecommendationKind,
	risk_score: RiskScore,
	risk_policy: RiskPolicy,
}

impl Recommendation {
	/// Make a recommendation.
	pub fn is(risk_score: RiskScore, risk_policy: RiskPolicy) -> Result<Recommendation> {
		let kind = RecommendationKind::is(risk_score, risk_policy.clone())?;

		Ok(Recommendation {
			kind,
			risk_score,
			risk_policy,
		})
	}

	pub fn statement(&self) -> String {
		format!(
			"risk rated as {:.2}, policy was {}",
			self.risk_score.0, self.risk_policy.0
		)
	}
}

/// The kind of recommendation being made.
#[derive(Debug, Serialize, JsonSchema, Clone, Copy)]
#[schemars(crate = "schemars")]
pub enum RecommendationKind {
	Pass,
	Investigate,
}

impl RecommendationKind {
	fn is(risk_score: RiskScore, risk_policy: RiskPolicy) -> Result<RecommendationKind> {
		let value = serde_json::to_value(risk_score.0).unwrap();
		Ok(
			if Executor::std()
				.run(&risk_policy.0, &value)
				.context("investigate policy expression execution failed")?
			{
				RecommendationKind::Pass
			} else {
				RecommendationKind::Investigate
			},
		)
	}
}

/// The overall final risk score for a repo.
#[derive(Debug, Serialize, JsonSchema, Clone, Copy)]
#[serde(transparent)]
#[schemars(crate = "schemars")]
pub struct RiskScore(pub f64);

/// The risk threshold configured for the Hipcheck session.
#[derive(Debug, Serialize, JsonSchema, Clone)]
#[serde(transparent)]
#[schemars(crate = "schemars")]
pub struct RiskPolicy(pub String);

/// A serializable and printable wrapper around a datetime with the local timezone.
#[derive(Debug, JsonSchema)]
#[schemars(crate = "schemars")]
pub struct Timestamp(DateTime<Local>);

impl From<DateTime<FixedOffset>> for Timestamp {
	fn from(date_time: DateTime<FixedOffset>) -> Timestamp {
		Timestamp(date_time.with_timezone(&Local))
	}
}

impl Display for Timestamp {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		// This is more human-readable than RFC 3339, which is good since this method
		// will be used when outputting to end-users on the CLI.
		write!(f, "{}", self.0.format("%a %B %-d, %Y at %-I:%M%P"))
	}
}

impl Serialize for Timestamp {
	fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
	where
		S: Serializer,
	{
		// The format is "1996-12-19T16:39:57-08:00"
		//
		// This isn't very human readable, but in the case of human output we won't be
		// serializing anyway, so that's fine. The point here is to be machine-readable
		// and use minimal space.
		serializer.serialize_str(&self.0.to_rfc3339())
	}
}

/// Queries for how Hipcheck reports session results
#[salsa::query_group(ReportParamsStorage)]
pub trait ReportParams: VersionQuery {
	/// Returns the time the current Hipcheck session started
	#[salsa::input]
	fn started_at(&self) -> DateTime<FixedOffset>;

	/// Returns the format of the final report
	#[salsa::input]
	fn format(&self) -> Format;
}
