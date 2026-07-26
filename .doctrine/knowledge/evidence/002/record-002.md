# EVD-002: claude -p exposes token usage metrics without usage billing

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

The user reports that Claude Code's non-interactive `claude -p` surface exposes
good token-usage metrics while continuing to run under subscription billing
rather than requiring metered API billing.

This makes `claude -p` the leading candidate for the first registered
measurement-source adapter under SL-231. Before registration, QUE-176 still
needs to verify the exact output fields, invocation boundary, completeness
semantics, stability, and correlation available to the adapter.
