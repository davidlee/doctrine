# Review RV-001 — reconciliation of SL-040

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation reconciliation of SL-040 (RV review-ledger kind + verb
family) against ADR-007 (D-C0…D-C11), `design.md`, and `plan.toml` VT criteria.
The first self-hosted audit: this RV reviews the very kind it is built on.

Lines of attack — the seams, where interaction bugs hide between sound parts:

- **Concurrency core (D-C4a/D-C2/D-C3):** entry-CAS + pre-write-CAS cover both
  edit windows; is *every* baton mutation under the per-review lock? Probe the
  post-turn handoff-note write specifically.
- **Close-gate corpus scan (D-C9b):** `unresolved_blockers_for` match/filter
  correctness, and conservative-vs-non-conservative parse of a hand-corrupted
  severity/status (does an unknown severity still gate?).
- **Total status function (D-C8):** totality + named cases — verified by VT-1/2.
- **Warm-cache staleness (D-C10):** content-hash of explored path set, absence⇒
  stale (R1); path-traversal guard on the curated `domain_map`.
- **Coverage parity:** every other numbered kind ships an `e2e_*_cli_golden`;
  does the review CLI surface (and the `/audit` pilot, VA-1) have automated
  end-to-end coverage, or only in-module unit calls?

Evidence base: `just check` green (exit 0); a HIGH-effort code-review subagent
pass over `git diff 4eec11f~1..f9fa5da -- src/ install/ plugins/ .gitignore`
(13 files, +4633). Findings below ingest that pass plus design conformance.

## Synthesis

SL-040 reconciles cleanly. The pure core, the total status function (D-C8,
property-tested over all status pairs), the turn protocol (entry-CAS +
pre-write-CAS over both edit windows, RAII lock, loser-aborts-clean), the
close-gate corpus scan (D-C9b), and the warm-cache content-hash leaf
(absence⇒stale, R1) are all faithful to ADR-007 and the design's hardening.
Every authored VT criterion across PHASE-01…06 is green; `just check` exits 0.
No finding rose to `blocker` — the close-gate has nothing to refuse.

Seven findings, all terminal. The substantive three are owned future work, not
in-slice defects:

- **F-3 (major) → IMP-029.** The review verb family ships no black-box e2e CLI
  golden, the one coverage gap against the kind-parity pattern; the `/audit`
  pilot (VA-1) is agent-mode, so the integration path has no automated guard.
  Behaviour is unit-covered, so this is additive test work, not a regression.
- **F-1 (major) → CHR-001.** The post-turn handoff `--note` is the single baton
  write outside the lock. It self-heals via the next entry-CAS (baton is
  regenerable, D-C2) so impact is cosmetic, but it dents the every-write-under-
  the-lock invariant — worth tidying.
- **F-2 (minor) → CHR-001.** `validate_domain_map` over-rejects on a `..`
  substring; cooperative tooling, low likelihood, easy component-based fix.

The four nits resolved without follow-up: F-5/F-6 are conservative-by-design
behaviours (unknown-status⇒Open; `status` serializes under the lock) disposed
**aligned**; F-7's source-grep bypass guard is the exact mitigation §7 Charge
VIII committed to, disposed **tolerated** with the brittleness consciously
accepted (it fails loud on refactor — the intended tripwire). F-4 (close-gate
not failing safe on a hand-corrupted severity) folds into CHR-001 as a small
fail-safe hardening; reachable only via hand-edit of a closed enum.

Standing risk going into close: the RV kind ships its own first audit with no
e2e CLI regression net (IMP-029). Accepted for closure — unit coverage is
comprehensive and the pilot is exercised by this very audit. Notably, this
audit is itself the dogfood proof of VA-1: `/audit` produced RV-001, not a
hand-made `audit.md`. The pilot integration works.
