# Implementation Plan SL-192: Cascade trait-set selection

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases, split on the ADR-001 layer boundary: **PHASE-01** is the whole pure-
engine change (`src/hymns.rs`, leaf); **PHASE-02** widens the command/loader shell
(`prompt.rs`, `install.rs`) that feeds it. This maps the change onto its natural
cohesion seam — the engine gains the capability, the shell exposes it — and keeps
each phase independently green and verifiable. Implements SPEC-023 FR-004 / FR-005 /
FR-007 (PHASE-01) and the FR-009 CLI-arity delta (PHASE-02).

## Sequencing & Rationale

**Why the engine is one atomic phase, not three.** The delivered `.model` axis is
`Option<String>` on two structs and read at ~6 sites (`matches`, `depth_of`,
`model_matches`, the specificity/precedence chain, plus ≈12 test construction
sites). Flipping the type to `BTreeSet<String>` does not compile until every one of
those sites moves — so membership matching (FR-004), conjunctive selection (FR-005),
and root-wise specificity (FR-007) cannot be staged behind separate green barriers
without shipping throwaway intermediate encodings (e.g. a scalar-sum specificity that
PHASE-02 would immediately rewrite). Landing them together is *less* code and no
rework. TDD still applies within the phase: red the membership/conjunction/
specificity goldens, green them, refactor.

**Why specificity is a raw sorted multiset, not root-collapsed.** The external
adversarial pass (codex, design §8) proposed collapsing duplicate-root pins. That
breaks a real case: `capability/code/high ∧ capability/reasoning/high` — two distinct
sub-trees of one root — is a legitimate intersection that must outrank its factor
`capability/code/high`. As a sorted sequence `[(capability,3),(capability,3)]` the
prefix rule delivers that; collapsing to one `(capability,3)` entry would flatten it
to equal its factor. SPEC-023 D3 mandates "an ordered sequence of `(root, depth)`
pairs" precisely for this. Redundant-nested / contradictory same-root pins are
authoring anti-patterns, deterministically ordered, with the lint deferred to SL-191
(SPEC-023 OQ-3).

**Why the shell is a separate second phase.** After PHASE-01 the engine is set-
valued but the CLI still takes a single `--model` (wrapped into a singleton set at
the `build_ctx` boundary) and the sidecar still parses a single model string
(wrapped likewise) — a fully green, behaviour-preserving state. PHASE-02 then widens
*only* the shell: repeatable `--model`, and `Sidecar.model: Option<Vec<String>>`. The
`Option` wrapper is the load-bearing detail codex caught (design §8 / D4): a bare
`#[serde(default)] Vec` cannot distinguish an omitted field from an explicit empty
list, and would silently clear every path-derived model pin. `None` keeps the path
pin, `Some([])` unpins, `Some(list)` replaces.

**Why the e2e uses shipped snippets, not SL-191 hymns.** PHASE-02's e2e must prove
repeatable `--model` composes a multi-key context end-to-end, but the trait-keyed
hymns (`model/adherence/low.md`, …) are SL-191 (a non-goal here). So the e2e drives
the two *existing* shipped model snippets — `anthropic/claude-sonnet-4` and
`deepseek/_default` — with two `--model` flags: the context set carries both keys,
both snippets match by membership, both compose. This exercises FR-004 + the FR-009
arity with zero dependency on unshipped content. Engine-level composition of *trait*
intersections is proven by PHASE-01's in-memory hymns goldens.

**Behaviour-preservation gate.** The only intended output change across the whole
slice is the `explain` / `Spec` byte-form (the specificity print generalises from
`(u32,u32)` to the pair-vec). Every other delivered golden must stay green with its
context/selector migrated `Option`→singleton-set — asserted by VA-1, reviewed by
intent, never rewritten to fit. (This is the hymns module, not the shared entity
engine, so the strict engine-suite gate is not in play; the discipline still holds.)

## Notes

- Repo lint denies (`clippy::format_push_string`, `expect_used`/`unwrap_used` in
  non-test code, `print_stdout`): build the `explain` line via `Vec<String>` +
  `concat`, never `push_str(&format!)` or `write!(…).expect()` — see the house
  style in `src/retrieve.rs::format_find`. `writeln!(io::stdout(), …)?` at the shell.
- No `install/` asset edits in this slice (trait hymns are SL-191), so the
  rust-embed re-embed footgun does not bite here; the e2e reads existing embedded
  snippets. If a fixture snippet is ever added under `install/`, `touch
  src/install.rs` before trusting `cargo build`.
- `model-keys` and `doctrine_onboard` are deliberately untouched (design §7): the
  former reflects model-band authored labels (its correct contract); the latter's
  single-valued `--model` usage copy is an SL-187 delivery follow-up.
