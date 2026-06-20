# Review RV-114 — reconciliation of SL-104

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Review surface (F-2).** This audit reviews the impl-bundle **`review/104`**
(`35d16875`) against base `844fe25b` — **not `main`**. SL-104 code is unintegrated
(integration is `/close`'s job, post-audit); no dispatch candidate interaction
branch was created (handover treats `review/104` as the reviewable bundle
directly). Net delta = **4 files, +181/-5**, test + comment-string only, **zero
production behavior change**.

**What this audit probes** — SL-104 is a deliberately narrow hardening pass
(design §11 cut-set confirmed). Two deliverables + one test:

1. **NF-001 (REQ-275) structural non-blocking** — two-tier, dependency-free:
   Tier-1 allowlist source-scan (`tests/e2e_estimate_non_blocking.rs`, bundle-only)
   confining facet symbols to the exposure surface; Tier-2 compile-time exhaustive
   `Gate` destructure (`slice.rs`) proving the closure-gate input is facet-free.
2. **Confidence legitimization** — correct the 5 stale `expect(dead_code)` reason
   strings in `estimate.rs` (off SL-102/103, onto IMP-112 + the confidence-REQ
   landing at reconcile); `expect`s stay armed; no behavior change.
3. **FR-008 value asymmetry** — pin the untested `value = -5` valid contract.

**Lines of attack / invariants held** (per primed `domain_map`):
- Does Tier-1 actually fail on a new gating read, and is the allowlist honest?
  → **R1**: allowlist is **9 files** (incl. `main.rs`), not the 8 in design §4 /
  EX-1. Is `main.rs` a legitimate exposure site or a smuggled gating read?
- Does Tier-2's `Gate` destructure remain exhaustive (no `..`) and facet-free?
- **R2**: the residual gap — a hand-written close fn in allowlisted `slice.rs`
  reading `SliceDoc.estimate` evades both tiers. Honestly disclosed?
- **R3**: the confidence reason strings cite a REQ that does not yet exist —
  placeholder discipline, REQ minted at reconcile via Revision (D2 / ADR-013).
- **R4**: NF-002 / NF-003 cited green (graph VT-3; `e19`/`v7`/
  `custom_deserialize_unknown_keys`), **not** rebuilt — Non-Goals respected.

**Evidence gathered (this audit, against `review/104` in an isolated detached
worktree, torn down after):**
- Tier-1 `no_facet_symbol_outside_allowlist` — **pass** (0.08s).
- Tier-2 `gate_destructure_is_exhaustive_and_facet_free` — **pass**; `Gate` has a
  single field `extra_reqs: Vec<String>` (statically confirmed facet-free).
- `value::tests::v5a_negative_finite` — **pass**.
- `cargo clippy` (bins/lib) — **zero warnings**.
- VA-1: `grep -nE 'SL-102|SL-103' src/estimate.rs` on the bundle — **empty**.
- Verify deliberately **scoped** (clippy + targeted tests), not `just gate`/`just
  check`: both run `cargo fmt` which would reformat the whole crate and pull in
  pre-existing `status.rs` fmt drift that is not SL-104's (handover gotcha).

## Synthesis

**Closure story.** SL-104 delivers exactly its narrowed scope (design §11 cut-set)
and nothing inert. Every authored EX/VT criterion is satisfied on the `review/104`
bundle, independently re-verified this audit: Tier-1 allowlist scan green, Tier-2
`Gate` destructure green and provably exhaustive (`Gate` carries a single field,
`extra_reqs: Vec<String>` — facet-free by construction), the FR-008 value-asymmetry
test green, clippy zero-warn, and the stale SL-102/SL-103 dead-code citations fully
purged from `estimate.rs` (VA-1 empty). The change is test + comment-string only;
no production code path moved. The slice leaves the estimate/value facets
production-ready with no unspec'd dead code.

**The five findings split cleanly.** Two are deferred-to-reconcile confirmations
(`verified`), three are accepted-as-is (`aligned`):

- **F-1 (allowlist 9≠8)** — the only true code↔authored-doc divergence. The 9-file
  allowlist is *correct* (`main.rs` is a genuine CLI-write-handler exposure site;
  dropping it reddens the tripwire on `main`); the locked design §4 / plan EX-1 are
  *stale* at 8. Accepted by user ruling (2026-06-20). Remediation is a per-slice
  doc edit at reconcile, not a code change.
- **F-3 (confidence REQ placeholder)** — the SL-101-stranded confidence governance.
  The reason strings are honest descriptive placeholders (design F5 sequencing);
  reconcile discharges them by minting the percentile-model REQ via a Revision
  (D2 / ADR-013) and rewriting the citations. Governance, routed through a REV.
- **F-2 (residual gap)** — the disclosed, consciously-accepted boundary of the
  structural proof; a hand-written `SliceDoc.estimate` read in allowlisted
  `slice.rs` evades both tiers. No change; this Synthesis carries the disclosure
  the retired `audit.md` once would have.
- **F-4 (NF-002/NF-003)** — Non-Goal respected; green via existing coverage, cited
  not rebuilt.
- **F-5 (collision-exclusion breadth)** — load-bearing impl detail, correct, design
  prose was illustrative.

**Standing risks consciously accepted.**
1. The Tier-2/allowlist structural proof does **not** cover a hand-written facet
   read inside an already-allowlisted file (F-2). Mitigated by review; a future
   syn-based fitness gate (SL-112) is the consolidation candidate, out of scope.
2. The confidence percentile model ships as armed-but-dead code (F-3) until
   IMP-112 wires the display path; the `expect(dead_code)` tripwires self-clear on
   consumption. No gating/aggregation effect exists in v1 by design.

**No blocker; the close-gate is clear.** Both `verified` findings route their
remediation to reconcile, which is the next stage — neither blocks the
`audit→reconcile` transition (reconcile is precisely where they are discharged).

## Reconciliation Brief

Built from every non-aligned finding (F-1, F-3). Grouped by write surface.

### Per-slice (direct edit)
- **design.md §4 + plan.toml EX-1 (F-1)**: amend the Tier-1 allowlist from 8 to
  **9 files**, adding `src/main.rs`, and update the prose that enumerates it
  (design §4 allowlist block, the "8-file" mentions in §4 and EN-1/EX-1). Rationale:
  `main.rs` is a legitimate facet exposure site via the estimate/value CLI write
  handlers (`main.rs:4501/4541/6358/6369`); the locked design under-enumerated the
  surface. The shipped allowlist is correct; the authored docs are stale.

### Governance/spec (REV)
- **SPEC-020 + new confidence REQ (F-3)**: mint a concrete `REQ-NNN` via a Revision
  (ADR-013, REV-002 precedent) homing the percentile model — `lower`/`upper` read as
  project-wide P-low/P-high band; defaults `0.1`/`0.9` from `doctrine.toml
  [estimation]`; each bound finite, in `[0,1]`, `low < high`; **display-framing only,
  no gating/aggregation/normalization in v1**; no entity-local field; estimate-only.
  Amend SPEC-020 (one responsibility bullet + a `### Confidence band resolution`
  subsection under "Project-wide unit resolution" + the REQ in `members.toml`).
  Classification drafted *functional* (OQ-1) — reclassify to quality at reconcile if
  the spec author prefers; low stakes.
- **estimate.rs reason-string follow-through (F-3, code touch at reconcile)**: after
  the REQ is minted, rewrite the 3 `expect(dead_code, reason=…)` strings on
  `DEFAULT_LOWER_CONFIDENCE`, `DEFAULT_UPPER_CONFIDENCE`, and `resolve_confidence`
  (and the `mod display` framing) to cite the concrete `REQ-NNN` in place of the
  "confidence requirement landing at SL-104 reconcile" placeholder. Tripwires stay
  armed.
