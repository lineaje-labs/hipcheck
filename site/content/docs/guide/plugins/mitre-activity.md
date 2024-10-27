---
title: "mitre/activity"
extra:
  nav_title: "<code>mitre/activity</code>"
---

# `mitre/activity`

Activity analysis looks at the date of the most recent commit to the branch
pointed to by `HEAD` in the repository. In the case of a local repository
source, that may be a branch other than the default. In the case of a remote
repository, it will always be the default branch on the remote host.

Hipcheck identifies the committed date of the most recent commit, and
calculates the number of weeks between that commit and the day Hipcheck is
performing this analysis. It then compares that duration against the
configured threshold (default configuration: 71 weeks / one year). If the
duration in the repository is greater than the configured threshold, then
the analysis will be marked as a failure.

## Queries

### `mitre/activity` (default query)

Returns a `Span` representing the time from the most recent commit to now.

## Configuration

- `weeks`: An `Integer` of the permitted number of weeks before a project is
  considered inactive.

## Default Policy Expression

`lte $ P{config.weeks}w`

## Limitations

* __Cases where lack of updates is warranted__: Sometimes work on a piece of
  software stops because it is complete, and there is no longer a need to
  update it. In this case, a repository being flagged as failing this analysis
  may not be truly risky for lack of activity. However, _most of the time_
  we expect that lack of updates ought to be concern, and so considering this
  metric when analyzing software supply chain risk is reasonable. If you
  are in a context where lack of updates is desirable or not concerning, you
  may consider changing the configuration to a different duration, or disabling
  the analysis entirely.
