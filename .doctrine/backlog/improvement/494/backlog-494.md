# IMP-494: review show: per-finding detail read (--finding / --full)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`review show` renders the finding index (IMP-490): id, severity, status,
disposition and title. A finding's `detail` and `response` — the reasoning a
human needs to judge it — are reachable only via `--json` or the raw ledger
TOML. So a pointer like "see `doctrine show RV-NNN`" (the SL-268 Reference forms
rule) answers *what* a finding is, not *why*. The user calls the missing read
surface a core issue; quick follow to SL-268.

Proposed:

- `review show RV-NNN --finding F-n` — one finding in full: severity, status,
  disposition, route, title, detail, response, and (post SL-268) its turn
  journal.
- `review show RV-NNN --full` — the same for every finding.
- MCP parity on `review_show`.

Tension: RFC-032 `D5` rejected a `--findings` opt-in because the index should be
the default. `--finding F-n` is a selector, not an opt-in, so it does not
conflict. `--full` is an opt-in for a larger render. Settle whether that
passes `D5` or whether the default stays the index with `--finding` alone.

Sequence after SL-268: v2 adds `route` and the turn journal, which the render
should show. Related: IMP-475 (the D5 read-surface remainder) and IMP-476 (unify
the CLI/MCP read shapes).
