# A VA criterion over gitignored runtime state leaves no evidence an audit can re-derive

When authoring a `VA-`/`VT-` criterion, ask **where its evidence will live when
`/audit` reads it** — not just how the phase will discharge it.

## Why

A criterion whose subject is `.doctrine/state/**` has no tree-readable proof by
construction. The subject is gitignored, the harness is usually a scratch test,
and both are gone by audit. What reaches the auditor is a sentence in a phase
sheet, which they can accept or re-run — and re-running is often foreclosed
because a later phase deliberately moved the thing being compared.

SL-244 PHASE-02 `VA-1`: *"an existing snapshot round-trips unchanged, checked
against a real one rather than a constructed fixture."* Discharged by a harness
that parsed and re-serialised two live `design.toml` runs before and after, run
twice and deleted. By audit (`RV-345` `F-7`) PHASE-05 had deliberately moved the
snapshot shape, so the comparison no longer isolated PHASE-02's change. The
window in which the criterion was checkable closed three phases later.

The sharp part: the criterion's insistence on a **real** snapshot rather than a
fixture was correct — a fixture would not have caught what it was hunting — and
is exactly what made the evidence unrepeatable. Rigour and reproducibility pulled
opposite ways and nobody noticed at authoring time.

## How to apply

At `/plan` or `/phase-plan`, for any criterion whose subject is runtime state,
pick one deliberately:

1. **Capture the subject as a test asset.** Commit a real snapshot (redacted if
   need be) as a fixture, and keep the harness as a named test. The "real, not
   constructed" property survives because the bytes came from a real run.
2. **Pin the compat at the tier that breaks**, not at the inner type — a parse
   test over a literal legacy fragment. Precedents:
   `a_snapshot_written_before_the_policy_reads_as_human_only`,
   `a_snapshot_written_before_the_intent_subject_key_still_parses`
   (`src/design_run/snapshot.rs`).
3. **Accept a recorded outcome, and say so in the criterion.** Legitimate, but
   write it into the `expects` text so the auditor is not surprised, and record
   the outcome in the phase notes with enough detail to be believed.

Option 3 is the default that happens by accident. Choose it on purpose or not
at all.

## Related

[[mem.fact.doctrine.storage-tiers]] — the tier framing. Runtime state is
disposable as a *tier*; that is not a licence for any individual file in it (see
`mem_019fcd1727fa7061b771179b113f5726`, which makes the same point about serde
wire forms).
