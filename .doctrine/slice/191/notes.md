# SL-191 Notes — durable decisions & findings

Design-stage notes. Durable record of decisions reached and the code seams
located, so a fresh agent does not re-derive them. Slice scope: `slice-191.md`.
Partial design: `design.md`. Originates from IMP-197 (SL-168 postmortem).

## Reframing journey (how the scope arrived at its current shape)

The raw IMP-197 (from SL-168 postmortem §5a/§5c/§5d.1) proposed bolting a
NEGATIVE CONTRACT template onto `dispatch arm-spawn`. Preflight overturned three
premises:

1. **Delivery already exists — ride it (no parallel impl).** The prompt cascade
   (SL-186 resolver + SL-187 delivery) is the home for role/model/stage
   instruction. IMP-197 is `after: IMP-155`; the postmortem predates the cascade.
   → Author content as **hymns**, not an arm-spawn hardcode.

2. **The claude arm is ALREADY wired** — via install-time subagent-def bake, not
   runtime injection. `src/install.rs` replaces the literal marker
   `{{ prompt resolve --role worker }}` (`WORKER_RESOLVE_MARKER`) in each shipped
   `dispatch-worker.md` def with resolved worker hymns (`expand_worker_marker`,
   `install.rs:910`; call site `install.rs:1713-1722`). So static contract
   reaches a claude worker via its **subagent def system prompt**. The gap: the
   marker resolves **role band only** (ContextVector role=Worker, model/stage=None,
   `bands: Only([Role])` — `resolve_worker_role_body`, `install.rs:895-908`).

3. **`cargo fmt` reframed — invert the postmortem.** The harm was never `cargo
   fmt` the verb; it was `cargo fmt` on an **unformatted base**, which reformats
   files the worker never touched → delta spills outside the declared set. Fix the
   base and fmt is idempotent except on the worker's own code — safe AND desirable.
   → Worker contract flips from **forbid fmt** to **MUST run the formatter on its
   own delta**, conditioned on a formatter-clean base.

4. **The formatter seam already exists and is POL-002-clean — `doctrine check`
   (SL-163).** Resolves argv from the owned `[verification]` config
   (`quick`/`commit`/`gate`); its help: *"never carries a host convention as
   correctness (POL-002)."* In this repo `check quick` runs `cargo fmt` via `just
   check`. So NO new formatter config key: workers/orchestrator invoke `doctrine
   check <cadence>`; the shipped hymn never names `cargo fmt`.

## Locked decisions

- **D1 (Q1) — model band is arm-asymmetric in EFFECT, never as a baked rule.**
  Hymns stay **model-keyed**; the model band is selected by the model a def
  actually targets, NOT by an arm→model map ("claude=strong / subprocess=deepseek"
  must appear nowhere in hymns or bake logic). A def with no pinned model resolves
  **no** model band. A claude worker run under pi would just declare pi's model and
  get the right band. The coupling lives in **data** (the def's declared model),
  not logic. (User: "claude arm could run under pi theoretically… that doesn't mean
  you bake that into the hymns.")

## Decisions — ALL LOCKED (design.md is canonical; 2026-07-03)

Design targets the **post-unlock world**: SL-192 (trait-set engine) **done**,
SL-193 (exposed-slot self-replaces) **at audit** (obj #3 overlay + twin
reconciliation gated on its close).

- **D2 (Q2) — trait-keyed model band via dedicated `traits:` frontmatter.** Not
  the harness's `model:` (identity, harness-consumed) — a separate classification
  field. Hymns are trait-keyed (`model/adherence/low.md`), NOT identity-keyed
  (`model/deepseek/_default.md`). Absent `traits:` → empty set → no model band.
  Marker stays a sentinel; bake reads frontmatter (rejected: parametric `--model`
  marker → drift). **No stage band** (folded into role; `Only([Role, Model])`).
- **D3 — trait population: `adherence/low` only.** §5c content is low-adherence
  guidance. `capability/*` deferred (no invented content). pi/universal def
  declares `traits = ["adherence/low"]`.
- **D4 (SPEC-023 OQ-3) — dual-site coverage lint over a shared pure predicate.**
  `traits_covered(declared, corpus)` in `src/hymns.rs`; called at the bake
  (install-time hard error) AND `prompt check` (author-time finding, embedded defs
  via `embedded_agent_defs()`). Def→corpus direction; on-disk def linting deferred.
- **D5 (Q3) — reject-and-halt import gate + base-clean precondition.** Post-import
  `doctrine check`; red → halt+report, never auto-fix (partial, hides compliance
  signal, breaks ADR-012 sole-writer + RFC-011 instrumentation). Base-clean before
  arm-spawn makes the worker's `check quick` delta-scoped.

Selectors updated: `model/deepseek/_default.md` → `model/adherence/low.md`;
+`src/hymns.rs`, `src/commands/prompt.rs`, `install/agents/pi/dispatch-worker.md`,
`src/dispatch.rs` as design-target. Relation `SL-191 references(implements) SPEC-023`.

## External adversarial pass integrated (codex/GPT-5.5, 2026-07-03)

Confirmed internal F1 (layering clean), F2 (contamination scope), and the
`traits:`↔`model:` separation (no hidden second source: `boot.rs:828`
orchestrator-only, `install.rs:900` role-only). Seven findings triaged (full
detail in design.md § External adversarial pass):

- **C1** base-clean is now **non-mutating prove-clean** (mutating the shared base
  has no owner → re-spill); cleanup operator-owned; pre-existing red ≠ worker fault.
- **C2** **no agent-def frontmatter parser exists** — my design claim was wrong;
  added a dedicated parser (traits optional, model cascade-ignored) + negative tests.
- **C3** `prompt check` now runs the **full-context resolver** + asserts `Model`
  band when `traits:` non-empty — proves declared→delivered, not just coverage.
- **C4** reverse dead-hymn lint → **deferred to IMP-242** (no live trigger; needs a
  2nd trait root D3 defers).
- **C5** SL-192 dep satisfied (done); one trait degenerates to cross-band union.
- **C6** hymn READMEs stale → rewrite in scope; marker rename deferred to plan.
- **C7** verification uses `prompt explain` + bake tests, not stateful `prompt
  resolve` (it regenerates boot.md before emit).

New deliverables: agent-def frontmatter parser, both hymn README rewrites, shipped
concept memory `mem.concept.doctrine.hymn-cascade`. Selectors +README/+memory
(design-target); IMP-242 filed (`originates_from SL-191`).

## Plan stage complete (2026-07-03) — see plan.md

7-phase plan locked, `ready`. Hardened over 2 codex/GPT-5.5 passes (all findings in
plan.md § Codex inquisition). Durable plan-stage gotchas:
- **verify-vt `patterns` are line-anchored** — a loop-over-forbidden-list test asserts
  on a different line than the literals, so `assert.*(cargo|target)`-style regex
  false-fails. Use whole-file keyword + unique fn-name mandates, not shape regex.
- **Import belt home = `src/worktree/import.rs`** (`classify_import`/`run_import_from_worktree`),
  NOT `src/dispatch.rs` (design/scope anchors corrected in de65523b).
- **`doctrine check` is a mutating config-cadence proxy** — no non-mutating variant
  exists; PHASE-05 must BUILD the fmt-`--check`+lint base-clean cadence.
- **`resolve_worker_role_body` returns `String`** → PHASE-03 must extract a pure
  `worker_context(traits) -> ContextVector` so PHASE-04 asserts `Band::Model`
  structurally (EX-6), not by body-substring.
- **F1 instance**: `install/hymns/harness/cursor.md` shipped host literals (IMP-245,
  over-eager cursor); user moving it to overlay. PHASE-02 POL-002 gate scoped to the
  SL-191-authored set (role/worker + model/**), NOT the whole tree.

## Content ownership cut (POL-002) — the audit table

Ships in `install/hymns` (doctrine-owned, host-agnostic) vs `.doctrine/hymns`
overlay (this repo's client habits):

| Content | Ships in `install/hymns`? |
|---|---|
| Touch only declared file set (subsumes whole-tree `cargo fmt` as out-of-set) | yes — role |
| No `.doctrine/`/`.claude/` writes; only git verb = final commit | yes — role |
| Hermetic goldens (never byte-assert live-corpus output) | yes — role |
| Component-anchored paths (principle) + owned dirs `.dispatch/`/`.worktrees/` | yes — role |
| State module home + layer/dependency rationale for each new fn (DIRECTIVE only) | yes — role |
| Run `doctrine check quick` per-edit / `check commit` before commit | yes — role |
| deepseek delivery patterns (§5c) | yes — model/deepseek |
| `cargo fmt`, `target/`, `node_modules/`, ADR-001 layer names, Rust module homes, `architecture_layering` gate | NO — `.doctrine/hymns` overlay only |

Rule that dissolves most host specifics: express the contract on the owned
"constrained writer / declared file set" primitive — "no `cargo fmt`" is just a
special case of "no out-of-set edit," so it ships host-agnostic and stronger.

## Key code anchors

- `src/install.rs:895` `resolve_worker_role_body` — the hardwired role-only
  ContextVector (the thing to widen).
- `src/install.rs:910` `expand_worker_marker(def, body)` — literal marker replace.
- `src/install.rs:1713-1722` — bake call site; loops per-harness embedded def;
  harness known here, model in def frontmatter.
- `src/install.rs:44` `WORKER_RESOLVE_MARKER = "{{ prompt resolve --role worker }}"`.
- `src/hymns.rs` — `resolve`, `ContextVector`, `Band`, `BandFilter`, seal/expose.
- Shipped defs: `install/agents/{claude,pi}/dispatch-worker.md`; installed twins
  `.doctrine/agents/{universal,claude,pi,codex}/dispatch-worker.md`
  (universal pins `model: deepseek/deepseek-v4-pro`; claude has none).
- Hymn corpus: `install/hymns/{role/worker,model/deepseek/_default}.md`;
  bands preamble/harness/model/role/stage/project (`install/hymns/README.md`).

## Constraints in force

- **POL-002** — shipped behaviour rests on owned contracts, never host
  conventions/local state. Central to the content split above. `doctrine check`
  and the cascade are the owned seams.
- **ADR-011** (governs) — harness-agnostic spawn interface; per-harness capability.
- **ADR-001** — module layering leaf←engine←command (the bake change lives in the
  command/engine layer; keep no cycles).
- **ADR-005** — tiered shipped knowledge; skills route, hymns/reference explain.
- **Behaviour-preservation gate** — `expand_worker_marker` unit tests + the
  subagent-def drift test (pins `name:` to `DISPATCH_WORKER_AGENT_TYPE`) stay green.
- **crane embed-strip** — any NEW `install/hymns/**` file rides the existing
  `install/` RustEmbed root (already grafted); validate `just nix-build` (host).

## Relevant memory

- `mem.pattern.dispatch.worker-prompt-run-full-suite` — the worker check must run
  the **regression-relevant** suite (`cargo test --bin doctrine <touched-module>`
  + phase e2e), NOT just the phase e2e; shapes the `doctrine check` content in
  `role/worker`.
- `mem.pattern.dispatch.verify-governance-freshness-before-distilling-worker` —
  resolve doc conflicts by edit recency; the postmortem here is deliberately
  superseded by the cascade reframing.

## Split-off / out of scope

- **ISS-206** (filed, committed `acc0af61`) — cascade concatenates same-slot
  Framework+User twins (no `replaces`) → baked def inlines `role/worker` twice.
  Resolver-semantics fix; NOT this slice. Related to SL-186.
- **Postmortem §5b/§5d.2-5** funnel/CI hardening beyond check-cadence wiring
  (`architecture_layering` always-green gate, delta-aware gate diff, golden
  regression harness, `doctor --baseline`) — different surface; follow-ups.

## Instrumentation (RFC-011 case-note filed)

`slice selector add` writes the `[[selector]]` array AFTER `[[relation]]`; a later
`doctrine link` then refuses to append (F1 ordering) → manual TOML re-home needed.
Seed relations first, selectors last — or fix the verb. (In `.doctrine/rfc/011/case-notes.md`.)

## Sequencing fork settled (2026-07-03, RFC-013 outcome)

Option (ii) + (iii)-as-framing: engine capability lands in **SL-192** (cascade
trait-set selection — set-valued context, selector conjunctive pattern-set,
root-wise normalized specificity, repeatable `--model`; implements SPEC-023
FR-004/005/007). `SL-191 after SL-192` recorded. SL-191 stays pure
worker-contract content: trait hymns, def trait-set frontmatter, bake widening.
Q2's model-source question now reads "def frontmatter declares the trait-key
SET" (not a single model id); Q3 unchanged. SPEC-023 OQ-3 (required-trait
lint) placement: decide in SL-192 or SL-191 design.

## Dispatch model plan (2026-07-03) — survives compaction

Arm: **claude, all 7 phases** (operator call). No deepseek/subprocess even where
VT would backstop — at 7 phases landing on main, minimise weird-diff surface;
uniform arm > cheap-arm canary.

Model per phase (opus on the three precision phases; sonnet on bounded/prose):

| Phase | Model  | Rationale (cap/adh) |
|-------|--------|---------------------|
| 01 predicate     | Sonnet | Med/Med. Pure `traits_covered`, fixture-tested, VT forgiving. |
| 02 hymns         | Sonnet | Low/High. POL-002 leak risk is obedience not horsepower; tight "no host literals" prompt. |
| 03 parser+bake   | **Opus** | High/High. Byte-identical behaviour gate + resolver sig + ADR-001 layering. SL-168 defect surface. |
| 04 prompt check  | Sonnet | Med-High. Rides check_corpus precedent; escalate to opus only if phase-plan widens surface. |
| 05 funnel        | **Opus** | Med-High/High. import.rs mutating cadence, ADR-012 C1; silent breakage. |
| 06 READMEs+memo  | Sonnet | Low-Med. Prose + embed ritual (build→sync→validate); checklist prompt. |
| 07 overlay+comp  | **Opus** | Med-High/High. Loader precision, must not break SL-193 projection. |

Decomposition principle: **adherence→arm, capability→model**. Sonnet on the
claude arm is still high-adherence (instruction-following is a claude-family
trait); opus-vs-sonnet buys reasoning, not obedience. So 02 = sonnet despite
High adherence.

Bootstrap paradox: SL-191's contract protects *future* dispatches, not this one.
This drive's only guardrails are model tier + tight per-phase-plans. That's why
03 gets opus + worktree isolation despite being "just a parser" — the contract
that would catch a loose worker doesn't ship until 03 lands.

## Audit harvest (RV-242, 2026-07-04) — closure + durable findings

All 7 phases green: `check gate` clean, clippy zero-warning, `verify-vt 191` all
pass, both live oracles correct (`prompt explain --role worker` = single FW
role/worker winner + additive User project habit, no ISS-206 doubling; `--model
adherence/low` composes the FW model band), VH-1 signed off. Audit = RV-242
(reconciliation facet), two minor findings, both terminal, no blockers.

**F-1 (verified → reconcile): selector-registry drift.** `slice conformance 191` =
11 undeclared / 2 undelivered — all benign (delivered work correct). Undelivered
`src/dispatch.rs` is the stale selector this notes file already flagged (line ~112:
import belt lives in `src/worktree/import.rs`); undelivered `…hymn-cascade/**` is a
glob-vs-symlink mismatch. Reconcile owns the fix (`slice selector rm`/`add` +
design.md §6 mirror) — see RV-242 `## Reconciliation Brief`. No code change.

**F-2 (follow-up → backlog): PHASE-05 EX-4 fork-arm base-clean parity deferred.**
The pi/shared-funnel arm's non-mutating prove cadence + import gate shipped and are
green; subprocess/fork-arm parity is conscious future work. Backlog item minted at
harvest (the "recorded" half of EX-4).

**Durable design insight — band-filter asymmetry (→ memory).** The agent-def bake
(`resolve_worker_role_body` → `worker_context` = `BandFilter::Only([Role,Model])`)
EXCLUDES the `project` band; the session cascade (`prompt resolve/explain`,
`BandFilter::All`) INCLUDES it. The `project`-band home for the client habit
(P07's `doctrine-rust-conventions`) is correct *only because* EX-3/VA-1 verify the
All-bands cascade — a project habit reaches a worker via SessionStart, not the
narrow baked contract. Recorded as `mem.concept.doctrine.worker-resolve-band-filter`.

**Dispatch-tooling gotcha hit at close (→ backlog + RFC-011 case-note).** Phase-
completion sheets are per-worktree runtime state; `registry_completeness` reads the
completed-set from `git::primary_worktree` (state.rs:906 / dispatch.rs:1901) while
an orchestrator-AUTHOR flips `slice phase --status completed` from the COORD cwd →
all completions land on coord, primary stays stale → `prepare-review` bails
("recorded row … not a completed phase"). record-boundary DOES write primary, so
only the completion flags diverge. Fix was `slice phase … --status completed -p
<primary>` ×7. Bites orchestrator-author dispatch only, never worker-driven.

**Standing risks accepted:** (1) overlay reconciliation is this-repo-only; the
projection default re-writes the self-`replaces` starter on `doctrine install`
(ISS-210, out of scope by design). (2) fork-arm base-clean parity deferred (F-2).
