# Review RV-155 — design of SL-151

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Lines of attack — this Inquisition probes:**

1. **Design coherence.** Are D1–D3 internally consistent? Does any decision
   contradict another? Does the design claim something that the affected
   surface cannot deliver?

2. **Scope alignment.** Does `slice-151.md` still reference the dropped
   structural scanner or remediation hint? Does the scope's "Non-Goals"
   section match what the design actually excludes?

3. **ADR-001 layering.** Does `parse_entity_toml` live at the right
   altitude (pure leaf, no IO)? Does `scan_kind`'s new parse belong at the
   engine layer? Is the `commands/validate.rs` boundary clean?

4. **Caller survey accuracy.** The design claims ~13 callers for
   `read_meta`/`read_id`. Are there missing callers in the survey table?
   Are the stated prefixes correct for each kind?

5. **Verification coverage.** Do VT-1 through VH-1 actually cover every
   claimed behaviour? Are there gaps — e.g. no test for the `read_id`
   wrapper path, or no test proving `parse_entity_toml` is a pure 1:1
   replacement for `toml::from_str` on valid TOML?

6. **Performance claims.** The design states "no catalog performance
   impact" because `scan_kind` is only called by `validate`. Is this
   actually true at the code level? Does any other code path call
   `scan_kind` or `id_integrity_findings`?

7. **Error quality.** With the remediation hint dropped (F-1), does the
   canonical-id context alone improve the error enough? Is the example
   error shape in the design achievable with `anyhow::Context`?

8. **Governance.** Does the design conflict with ADR-004 (outbound-only
   relations) or ADR-006 (tier merge-safety, detect-half)? Does it
   violate any project convention (POL-001 clankspeak, POL-002 platform
   independence)?

**Held to these invariants:**
- No string-matching on error text (fragile memory)
- No catalog performance regression
- No stale scope references to dropped features
- Every stated caller actually exists with the claimed prefix
- `parse_entity_toml` is a pure leaf (no IO, no config dependency)

## Synthesis

> **The verdict: the design's _decisions_ are sound; its _accounting_ was a
> tissue of false witness. The accused has confessed, the apparitions are
> burned, and the corrected record stands. SL-151 may proceed to `/plan`.**

The skeleton — D1 (shared `parse_entity_toml` wrapper, no structural
pre-scan), D2 (`scan_kind` schema-agnostic `toml::Value` parse for
`validate`), D3 (route the main read paths) — survived cross-examination
whole. No heresy in the decisions:

- **Layering (ADR-001) is clean.** `parse_entity_toml` is a true pure leaf
  (owned text in, no IO) — and `dtoml.rs` already harbours the pure-`parse`
  precedent. `scan_kind` lives at the engine; `commands/validate.rs` only
  composes. The `prefix`-as-parameter mechanism is not a wart but a
  *necessity*: `KINDS` lives in `integrity.rs` and `meta.rs` sits beneath it,
  so the prefix MUST descend from callers — a registry reach-up would invert
  the layer. **Confirmed under the iron.**
- **Performance claim holds.** `scan_kind` is called from exactly one place —
  `id_integrity_findings`, itself called only by `validate`. `scan_entities`
  does not touch it. And `with_context` allocates only on the error path, so
  even `catalog/scan.rs:429` (the forced `read_meta` caller on the hot path)
  pays nothing at runtime.
- **Error quality is achievable.** `anyhow::Context::with_context` over
  `toml::from_str`'s `Result` yields exactly the `SL-007: TOML parse failed` /
  `Caused by:` chain shown. The dropped remediation hint is the *right* mercy —
  a speculative "check for non-contiguous sections" would bear false witness on
  type-mismatch and escape errors. The canonical-id IS the prize.
- **Governance is unviolated.** No relations touched (ADR-004 silent). The
  `validate` augmentation is a pure *detect-half* — it raises a hard error,
  never auto-merges (ADR-006 honoured). No clankspeak (POL-001), no host-coupling
  (POL-002).

**But the surface-accounting was rotten, and rot spreads.** Eight charges
raised; all eight now terminal:

- **F-1 (blocker, fix-now·verified)** — `lazyspec.rs:468` and
  `catalog/scan.rs:429` were confessed out-of-scope yet both call `read_meta`
  in production. A `prefix` param is compile-forcing; the design claimed a
  surface it could not deliver. Both moved into the affected-surface table.
- **F-2 (blocker, fix-now·verified)** — the caller survey was a fabrication:
  rows for `adr.rs`/`policy.rs`/`standard.rs`/`rfc.rs`/`revision.rs`/
  `requirement.rs`/`review.rs`/`rec.rs` as *direct* `read_meta` callers, where
  grep finds **none**. All governance kinds funnel through ONE site
  (`governance.rs:71`, `g.kind.prefix`). The survey both invented callers and
  omitted the real ones; the 3.2–4.8-shot estimate rode the lie. Rewritten
  against grep.
- **F-3 (major, fix-now·verified)** — `read_metas`, the true list funnel, is
  stem-parameterised with no prefix in hand; the "each caller passes its
  prefix" model never accounted for it. Now threaded explicitly.
- **F-4 / F-5 / F-8 (minor, fix-now·verified)** — "Five" that listed six; a
  scope §1 that omitted `read_id`; a `read_id` survey blind to the
  `.ok()`-swallowed site at `integrity.rs:304`; a VT naming `validate_sections`,
  a symbol the design never births. All corrected to truth.
- **F-6 (minor, fix-now·verified)** — no VT pinned the `read_id` wrapper
  error-context path; **VT-7** added.
- **F-7 (nit, tolerated)** — `parse_entity_toml`'s placement in `dtoml.rs`
  (vs the more cohesive `meta.rs`). Consciously tolerated: both are pure
  ADR-001-clean leaves and `dtoml.rs` already hosts a pure parse precedent.
  Left to the author at plan/execute. **The only taint knowingly suffered to
  live.**

**Penance discharged.** `design.md` and `slice-151.md` were corrected in place
— decisions untouched, accounting made honest. The corrected caller topology is
authoritative in the design's **Caller survey**; the plan must take its breadth
from there, never from the burned `~13 modules` claim.

**Standing risks:**
- The `read_metas` prefix-threading (F-3) widens the mechanical surface; the
  plan must enumerate the funnel callers, not trust a module count.
- IMP-109 (single-parse catalog scan) may later touch `catalog/scan.rs`'s
  parse topology; SL-151's forced `read_meta` edit there (F-1) is orthogonal
  but adjacent — sequence with care.

**HERESIS URITOR; DOCTRINA MANET**
