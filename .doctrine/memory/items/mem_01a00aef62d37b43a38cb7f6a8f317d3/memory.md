# `doctor --json`: the envelope is `rows`, and `category` is the display name

Two things bite in sequence when scripting `doctrine doctor --json`:

1. **The payload is not a top-level array.** It is `{"kind": "doctor", "rows":
   [...]}`. A `jq '.[] | select(...)'` errors with *Cannot index string with
   string* — loud, so this one self-reports.
2. **`category` is rendered with `display_name()`, not the variant name.** The
   values carry spaces: `"Relation Integrity"`, `"Raw Label"`, `"Prose
   Citation"`, `"Spec FK"`. A filter written against the Rust variant —
   `select(.category=="RelationIntegrity")` — matches **nothing and says
   nothing**.

The second is the dangerous one. It fails as an *empty result*, which reads
exactly like a clean corpus.

## How it presented

Verifying `SL-238` PHASE-03's `VA-1` — "`doctrine doctor` reports zero findings
from the new check on the live corpus". The first two attempts returned zero.
Both were wrong: the first from the `.rows` error, the second from the variant
spelling. The check was working the whole time.

What caught it was running the **same query against a scratch corpus seeded with
three known-broken refs**. That returned zero as well, which is impossible, so
the query was the defect rather than the corpus. With `"Relation Integrity"` the
control returned its three findings and the live corpus still returned zero — a
zero that now meant something.

## What to do

- Filter on the display name, or filter on `.message` shape and ignore
  `category` entirely.
- **Never accept a zero from an unvalidated query.** Run it against input that
  must produce hits first. See
  [[mem.pattern.harness.grep-negative-needs-positive-control]] and
  [[mem.pattern.verification.suspect-transcription-before-tool]].
- The exit code is an independent cross-check: any Error-severity finding makes
  `doctor` exit non-zero, so `exit 0` corroborates "no error findings" without
  any parsing at all.
