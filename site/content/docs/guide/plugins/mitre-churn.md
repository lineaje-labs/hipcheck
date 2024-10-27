---
title: "mitre/churn"
extra:
  nav_title: "<code>mitre/churn</code>"
---

# `mitre/churn`

Churn analysis attempts to identify the high prevalence of very large commits
which may increase the risk of successful malicious contribution. The notion
here being that it's easier to hide malicious content in a large commit than
in a small one, as malicious contribution relies on getting malicious changes
through a normal submission / review process (assuming review is performed).

Churn analysis works by determining the total number of lines and files
changed across all commits containing changes to code in a repository, and
from that the percentage, per commit, of those totals. For each commit, the
file percentage and line percentage are then combined, as file frequency times
line frequency squared, times 1,000,000, to produce a score. These scores are
then normalized into Z-scores, to produce the final churn value for each commit.
These churn values therefore represent how much the size of a given commit
differs from the average for the repository.

Churn cannot run if a repository contains only one commit (or only one commit
that affects a source file). Churn analysis will always give an error when run
against a repo with a single commit.

## Limitations

* __Whether churn surfaces malicious contributions is an open question__:
  We have ongoing work to confirm that churn does help identify the presence
  of malicious contributions, and therefore is a useful metric for assessing
  supply chain risk against malicious contribution attacks, but at the
  moment this is an assumption made by Hipcheck.
* __Churn's statistical calculations may be insufficient__: There is ongoing
  work to assess the statistical qualities of the churn metric and determine
  whether it needs to be changed.
