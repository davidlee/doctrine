# REV REV-040 — activate SL-228 requirements

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Follow-on to SL-228's close. The slice shipped, `verify-vt` is 25/25, the gate is
green at 4903/0 — and five of the six requirements it claims were still authored
`pending`, with `coverage show` reporting `observed = none`. The behaviour was
live; the ledger had never been told.

**The missing step was the coverage record, not the status flip.** SL-228's design
§11 ("Verification alignment, per requirement") authored the mapping from each
requirement to its verifying tests **as prose**, and nothing ever carried that into
the coverage store. So a status flip alone would have asserted satisfaction with
nothing backing it — which is precisely the unbacked-claim pattern REV-039 had just
finished removing from REQ-385. The order here is therefore: record the evidence,
run it, *then* move the status.

### What was recorded (`.doctrine/slice/228/coverage.toml`)

Five `VT` cells, one per requirement, each a literal check derived from design §11
and **validated by running it before recording**:

| requirement | check | tests |
|---|---|---|
| REQ-384 (`FR-008`) | `cargo test --bin doctrine dispatch::tests::funnel_record` | 18 |
| REQ-385 (`FR-009`) | `cargo test --bin doctrine funnel_machine::` | 29 |
| REQ-386 (`FR-010`) | `cargo test --bin doctrine dispatch::tests::next_oracle` | 13 |
| REQ-388 (`FR-010`, SPEC-022) | `cargo test --bin doctrine git::tests` | 122 |
| REQ-389 (`FR-011`, SPEC-022) | `sh -lc` — `dispatch::tests::hook_check` **and** `commands::guard::tests`, sentinel on both green | 4 + 3 |

`doctrine coverage verify 228` re-derived all five by execution:
`planned→verified` ×5, `0 VT entries lack a check`.

**Two traps met while authoring these, recorded so the next author skips them:**

1. **Multiple libtest filters do not OR.** `cargo test --bin doctrine -- a b`
   returns `0 passed; 0 failed` — it silently matches nothing. Paired with a naive
   `test result: ok` matcher that is a **false green**: a check that runs no tests
   and reports success. Hence the matcher here is
   `[1-9][0-9]* passed; 0 failed` (regex) — it requires at least one test to have
   actually run. Verified against a deliberately non-matching filter: no match.
   REQ-389 needs two disjoint modules and so uses an `sh -lc` sentinel that emits
   only when both pass, rather than a filter that would quietly cover neither.
2. `--command` needs the `=` form for any value that starts with a dash
   (`--command=--bin`), or clap consumes it as its own flag.

### Why `observed` reads `stale` and why that is not a blocker

A cell's `git_anchor` is HEAD at verification time, so committing the coverage
store itself moves HEAD and stales the anchor it just wrote — self-staling by
construction. This is the corpus-wide steady state, not a SL-228 anomaly: every
long-standing `active` requirement in SPEC-021/022 (REQ-287…295, REQ-311…321)
also reads `stale` / `Indeterminate`. Activating on freshly-executed evidence is
consistent with that established practice. The cells' value is that the evidence
is now **named, executable, and re-runnable** — `coverage verify 228` re-derives
it on demand, which was impossible while `observed` was `none`.

### Rows

Five `status` rows, `pending → active`. `status` is the row class
`revision apply` auto-lands, so this revision needs no manual prose landing.

- **REQ-384** (`FR-008`) — funnel position persisted per-phase as authoritative
  run-state, single-writer, crash-safe idempotent recovery.
- **REQ-385** (`FR-009`) — every funnel verb legality-gated on position. Note this
  activates the statement **as split by REV-039**: positional naming, with the
  prescription-completeness over-claim already removed. Activating the pre-REV-039
  wording would have been false; activating this one is not.
- **REQ-386** (`FR-010`) — `dispatch next` emits the single prescribed action.
- **REQ-388** (`FR-010`, SPEC-022) — every funnel git read is a first-class read
  verb; seams reused or relocated, not reimplemented.
- **REQ-389** (`FR-011`, SPEC-022) — working-tree-free funnel bounded by the
  no-pathless-commit / safe-commit guard.

### Deliberately excluded

- **REQ-387** (`FR-011`, SPEC-021) stays `pending`, and this is a decree rather
  than an oversight. Design §11's reconciliation posture (RV-304 F-8) states it
  flips `active` *only if* the subprocess arm projects through the gated funnel
  in-slice; that went to Non-Goals, so the fast-follow slice owns the flip. Its
  own words: *"partial delivery is never presented as satisfaction."*
- **REQ-335** (`FR-007`, SPEC-021) is not SL-228's. It is the confined-orchestrator
  altitude, appears nowhere in design §11, and is absent from the slice's claimed
  set (`slice-228.md`: SPEC-021 FR-008/009/010/011, SPEC-022 FR-010/011). It
  remains `pending` with `observed = none`, correctly `Coherent`.
