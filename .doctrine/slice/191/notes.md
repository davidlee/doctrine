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
