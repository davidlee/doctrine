# Implementation Plan SL-191: Dispatch worker contract: cascade hymns + check-cadence + role/model/stage bake

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Seven phases turning the locked design (`design.md`, D1–D5 + adversarial C1–C7)
into delivery. The spine is the worker-contract **delivery path** — predicate →
content → bake → check — followed by the enforcement **belt** (funnel) and the
knowledge **refresh** (READMEs + shipped memory). The one hard external gate
(SL-193 close) is isolated to the final phase.

- **PHASE-01** — `traits_covered` pure predicate (engine DRY core).
- **PHASE-02** — the two Framework hymns (role/worker + model/adherence/low), content.
- **PHASE-03** — agent-def frontmatter parser + bake widening (delivers traits to the baked def).
- **PHASE-04** — `prompt check`: coverage finding + delivered-`Model`-band assertion.
- **PHASE-05** — funnel check-cadence wiring (non-mutating base-clean + reject-and-halt import gate).
- **PHASE-06** — corpus knowledge refresh (README rewrites + shipped concept memory).
- **PHASE-07** — overlay reconciliation + live role-resolve verification (SL-193 gate **now cleared** — 193 done+merged; runs in-order).

## Sequencing & Rationale

**Dependency spine (01 → 02 → 03 → 04).** The predicate is the shared core both
the bake and `prompt check` call down to, so it lands first (PHASE-01), tested
against inline fixture corpora — no content or bake needed. Content (PHASE-02) is
independent authoring; it and the predicate have no ordering constraint, but the
bake (PHASE-03) depends on **both** — it must resolve non-empty (needs the
`adherence/low` content) and run the coverage predicate (needs PHASE-01). PHASE-04
extends the bake's own widened resolver into `prompt check`, so it follows
PHASE-03.

**Why the bake is the load-bearing phase.** Re-grep at plan time confirmed the
design's premises against the current tree, with two favourable shifts:
- The `prompt check` **def-enumeration seam already exists** (`check_corpus`,
  prompt.rs:322, already loops `embedded_agent_defs()` and calls
  `resolve_worker_role_body`). PHASE-04 *extends* that loop rather than building
  new enumeration — a smaller phase than the design assumed.
- **No agent-def frontmatter parser exists** (adversarial C2 confirmed): the bake
  does a byte-level marker replace only (install.rs:1796-1803). The parser is
  genuinely new surface and is the real weight of PHASE-03, alongside widening
  `resolve_worker_role_body` (SL-192 already made the context set-valued, so the
  resolver change is populate-set + widen-band, not a struct migration).

**The belt (PHASE-05) floats.** The funnel wiring touches the import belt
`src/worktree/import.rs` (codex F3 — not `src/dispatch.rs`) and the dispatch
skills, invoking the already-owned `doctrine check` seam (SL-163). It
depends on nothing in 01–04 and could sequence anywhere; it is placed after the
delivery spine to keep the bake/lint story contiguous and to land the enforcement
belt once the contract it enforces exists.

**The refresh (PHASE-06) trails the content it describes.** The README rewrites
and shipped `mem.concept.doctrine.hymn-cascade` memory teach the trait-space +
self-`replaces` authoring model — so they land after PHASE-02/03 make that model
the actual shipped behaviour, avoiding documenting an intent that could still shift.

**GATE CLEARED (2026-07-03): SL-193 is `done` and merged to main+edge.** The
stray hand-authored twin is now a git-tracked authored overlay carrying a
projected `replaces = "role/worker"` sidecar. PHASE-07 EN-1 is satisfied; the F2
sequencing constraint is fully lifted and PHASE-07 runs in-order after PHASE-06
with no external wait. The reconciliation work itself remains substantive — the
overlay currently *fully* replaces `role/worker` (its thin generic content wins
over Framework), so PHASE-07 must re-home the client habits (likely to the
`project` band / a non-replacing slot) so the enriched Framework contract
actually composes rather than being suppressed. The paragraph below records the
original gate rationale for provenance.

**The (now-cleared) hard gate — SL-193 close — is isolated to PHASE-07 (F2).** SL-191
enriches the **Framework** `install/hymns/role/worker.md`. In *this* repo the
overlay `.doctrine/hymns/role/worker.md` currently `replaces` that slot (a stray
hand-authored twin, `replaces = "role/worker"`), so a live-repo
`prompt explain --role worker` reflects the overlay, not the Framework enrichment
— and until SL-193 reconciles that twin, any live role-resolve assertion is
contaminated. Plan-time re-grep found this gate **softening**: SL-193 has advanced
`audit → reconcile` and its self-`replaces` sidecar is already live on disk (the
`explain` trace shows the overlay winning and the Framework twin suppressed — the
ISS-206 doubling is fixed). So the design's original "SL-191 execution waits on
SL-193" collapses to a single tail: **PHASE-01…06 all verify against hermetic
fixtures / the embedded corpus / `prompt explain` and are SL-193-independent**;
only the overlay reconciliation (obj #3) and the live-repo role-resolve oracle
sit behind SL-193 close, in PHASE-07. SL-193 is at `reconcile` and likely closes
before we reach PHASE-07 — but PHASE-07's EN-1 hard-gates on it regardless.

**Verification posture (adversarial C7).** `prompt resolve` has a side effect — it
regenerates `.doctrine/state/boot.md` before emitting — so it is not a clean
read-only oracle. Every phase verifies via `prompt explain` (pure precedence
trace) + pure bake unit/e2e tests; `prompt resolve` is exercised only in
live/dry dispatch, never as the correctness oracle.

## Notes

- **Marker rename (F4/C6) — decided: keep the literal.** The sentinel
  `{{ prompt resolve --role worker }}` reads as role-only but now resolves role +
  declared traits. Renaming `WORKER_RESOLVE_MARKER` would churn both shipped defs'
  marker strings (user-visible in installed defs) plus the constant. Chosen:
  **keep the literal + a loud comment** at the constant and the bake call site
  documenting the sentinel contract (bake reads frontmatter, not marker args).
  Lower blast radius, no installed-def churn. Folded into PHASE-03.
- **Subprocess-arm base-clean (open code-detail)** — whether the subprocess fork
  path needs its own base-clean beat or the shared funnel covers it is decided
  inside PHASE-05 (EX-3), not left open past it.
- **POL-002 content gate** — PHASE-02 asserts no host literal (`cargo`/`target/`/
  `just`/`node_modules`) in the **SL-191-authored** hymns (`install/hymns/role/
  worker.md` + `install/hymns/model/**`), NOT the whole `install/hymns` tree: the
  `harness` band legitimately names host tooling (codex F1) and is out of scope.
  Host literals live in the `.doctrine/hymns` overlay (PHASE-07).
- **crane embed-strip** — the new `install/hymns/model/adherence/low.md` and the
  new `memory/mem.concept.doctrine.hymn-cascade/` ride the existing `install/`
  and `memory/` RustEmbed roots (already grafted); validate `just nix-build`
  (host-only release check, not per-commit).
- **Behaviour-preservation gate** — `expand_worker_marker` unit tests + the
  subagent-def drift test stay green (PHASE-03 VT-2); the trait-less resolve is
  byte-identical to today.
- **Graceful degradation on the SL-193 gate.** PHASE-01…06 deliver the whole
  shipped payload — Framework contract, bake, coverage lint, funnel belt, corpus
  knowledge — with **no** dependency on SL-193 close. If SL-193 has not closed
  when PHASE-06 completes, the slice is *code-complete minus the overlay
  reconciliation*: PHASE-07 waits on the gate, it does not block the rest. The
  slice should not read as blocked while only PHASE-07 is outstanding.
- **PHASE-03 is the heaviest phase** — new agent-def frontmatter parser + widened
  `resolve_worker_role_body` + call-site wiring + install-time hard-error + def
  declarations. Kept as one unit (objective-cohesive: "make the bake deliver
  traits"); the parser is the de-riskable sub-unit and should land red/green
  first within the phase, before the resolver widens onto it. Split into 4 VT rows
  (parser negatives / trait-less byte-identity / covered-bake+uncovered-hard-error
  / behaviour-preservation) so a failed phase localises.

## Codex/GPT-5.5 inquisition — integrated (2026-07-03)

An adversarial pass on the plan (not the locked design) surfaced 8 findings, all
verified against the tree and integrated:

- **F1 (blocker) — whole-tree POL-002 gate was impossible.** `install/hymns/harness/
  cursor.md` (IMP-245, just merged) legitimately names `cargo`/`clippy`/`nix develop`
  — the harness band's job. PHASE-02's gate is now **scoped to the SL-191-authored
  hymns** (`role/worker.md` + `model/**`), not the whole `install/hymns` tree; the
  harness band is explicitly out of scope.
- **F2 (blocker) — no non-mutating `doctrine check` variant exists.** `check` is a
  config-cadence proxy (`src/commands/check.rs`); `check quick` runs mutating
  `cargo fmt`. PHASE-05 now **establishes** a non-mutating base-clean cadence
  (fmt `--check` + lint) rather than assuming one (EX-1).
- **F3 (major) — import belt home was wrong.** The reject-and-halt gate lives in
  `src/worktree/import.rs` (`classify_import`, `run_import_from_worktree`), not
  `src/dispatch.rs`. PHASE-05 VT-1 retargeted; a stray-substring false-pass avoided.
- **F5 (major) — PHASE-04 had no structural seam.** `resolve_worker_role_body`
  returns `String`, so a band assertion could only substring-grep the body. PHASE-03
  now extracts a pure `worker_context(traits) -> ContextVector` builder (EX-6) that
  both bake and `prompt check` use; PHASE-04 asserts `bands ∋ Band::Model` directly.
- **F6 (major) — PHASE-07 had no automated VT** and was the *only* phase proving
  live composition, while the full-replace overlay makes PHASE-02's in-repo
  verification meaningless. PHASE-02 EX-4 now verifies the model band live but the
  role band via hermetic fixture; PHASE-07 gains a hermetic composition VT (VT-1).
- **F4/F7/F8 (sizing + weak mandates) — hardened.** PHASE-03 split into 4 VTs;
  PHASE-02/06 VT rows given unique fn-name keywords + `patterns` regex so
  `verify-vt` asserts real shape, not incidental substrings (`hymns`/`target`
  already pervade `src/install.rs`).
