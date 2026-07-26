# DEC-052: Default observations to authored storage with an explicit ignore tradeoff

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

The default storage tier for authoritative observation records is authored
collection data: committed, diffable TOML under
`.doctrine/observations/records/**`. The capture command creates the record but
never stages, commits, or pushes it. Until an operator or coordinator commits
it, a new observation remains visibly untracked.

This default has an intentional review-cost tradeoff. A busy observation
ledger can add many low-value files to a pull request and crowd the code or
governance changes a reviewer is trying to inspect. Projects may opt out by
ignoring `.doctrine/observations/records/` in the repository `.gitignore`, or
individual developers may use `.git/info/exclude` for a local-only choice.
Doctrine should document both escape hatches rather than treating committed
instrumentation as mandatory.

Ignoring authoritative records changes their operational properties: they are
local-only, do not survive a fresh clone or ordinary machine loss, cannot be
correlated or aggregated by collaborators, and provide no shared audit trail.
Projects choosing that mode own another transport or accept those losses.
Capture and local query continue to work.

Optional `.doctrine/observations/by-month/**` convenience symlinks and reserved
publication temporary files are derived or transient regardless of the
authoritative-store choice. They are gitignored by default, never queried as
authority, and may be rebuilt or removed.
