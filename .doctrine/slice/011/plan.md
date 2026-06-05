# Implementation Plan SL-011: Cache-friendly session boot context

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Six phases build the boot mechanism bottom-up — pure core first, then the data
it projects, then the wiring that installs it, then the sentry that guards it,
and finally the AGENTS.md rewrite that pays off the whole point. The spine is the
design's hard-won corrections from the two hostile passes
([design.md](design.md) §10): the **bounded ≤2-session lag law** (the hook
freshens the *next* session, not the current one — Charge I, codex F1), the
**disk-not-session sentry** (`boot --check` proves the file fresh, never the
inlined prefix — codex F2), the **enum+match harness seam** (no trait/Box/registry
for one impl + one stub — Charge IV), and the **honest reuse split** (memory is an
additive wrapper, adr is a behaviour-preserving extract — Charge V).

The phases are sized so each ends green and is a clean review unit. The pure core
(P1) renders every section as a benign marker, so `doctrine boot` works *before*
any listing reuse or authored asset exists — that `produce()` marker tolerance is
what breaks the old phase-2/phase-6 ordering knot (review fix #3) and lets the
data phases (P2, P3) land in either order against a stable frame. Nothing touches
`install.rs`/`skills.rs`; the new logic lives in `src/boot.rs` (D1), and the only
shared-file touches are the two listing reuses (P2), one additive and one a
guarded extract. The existing entity/slice/state/adr/memory/skills suites are the
behaviour-preservation gate throughout.

## Sequencing & Rationale

- **PHASE-01 (pure core + regenerate verb) first — fully inert, fully testable.**
  Stands up `src/boot.rs` with the pure assembler (`boot_sequence`/`render_boot`)
  and the impure shell (`produce`/`write_if_changed`/exec-path resolution), wiring
  `doctrine boot` to render a marker-only snapshot. This fixes the two
  cache-discipline invariants up front — determinism (no clock/rng) and
  content-diff writing — and orders the build-volatile ExecPath section **last**
  so a path change never busts the governance prefix (codex F4). Everything
  downstream slots sources into this frame.

- **PHASE-02 (adr + memory reuse) — the listings, honestly labelled.** Feeds the
  ADR and Memory sections from the existing list logic. The labels matter and are
  not symmetric (Charge V): `memory::list_rows` is a genuinely *additive* wrapper
  over `select_rows`/`format_list` (the allowed touch on the SL-012-contended
  file), while `adr::list_rows` is a behaviour-preserving *extract* of `run_list`
  — small, since `meta::format_list` already centralises the output, but it edits
  `run_list`, so the adr e2e is the proof obligation. adr is the riskier deed yet
  lands on the file SL-012 does not touch.

- **PHASE-03 (authored assets) — governance surface + routing digest.** Authors
  the two embed assets the snapshot projects, flipping Governance and Routing from
  marker to real. `governance.md` is seeded by the existing install Skip path (no
  `install.rs` change) and is held to its remit boundary — a pointer/digest layer,
  never a fourth source of truth restating CLAUDE.md / `doc/*` / ADRs (Charge VI).
  Independent of P2 thanks to marker tolerance; ordered here so the snapshot is
  fully populated before anything wires it.

- **PHASE-04 (harness seam + `boot install`) — the wiring, once the snapshot is
  worth installing.** The enum+match seam (Claude full, codex import-only), the
  idempotent inode-deduped `@`-import prepend, and the never-clobber/fail-soft
  settings-hook merge with the space-hardened ownership match (Charge VII). Comes
  after the snapshot is real so `boot install` has something complete to point at;
  one harness's failure must not abort the others.

- **PHASE-05 (`boot --check` + skills) — the sentry, narrowly scoped.** The disk
  freshness/health check and its `/route` + `/canon` consumers. The discipline
  here is honesty about what it can see: `--check` proves the *file* fresh and
  populated; it cannot prove the *current session's* inlined prefix is fresh, so
  `/route` pairs it with the in-session lag warning and neither skill over-claims
  (codex F2). No clock, no in-content timestamp — the signal stays out-of-band so
  the cache key is never polluted. The governance-hash in-session detector is left
  as deferred design headroom.

- **PHASE-06 (AGENTS.md rewrite + close-out) last — the payoff and the seal.**
  Only now does the duplication get cut: AGENTS.md sheds the recited CLI, the
  memory nag, and the process prose, because the snapshot finally carries them
  into the same cached prefix. This repo self-wires via `boot install` and
  dogfoods. The live ordeal and the live codex run are explicitly *not* gates on
  this work — they are post-build, user-run closure verification (the only thing
  that can settle the empirical load-order and codex-inline questions), recorded
  with the codex cut-from-v1 fallback.

## Notes

- **Concurrency:** SL-012 remains `in_progress` — the `skills.rs` gate stands, so
  the deferred Charge-IV unification of the boot harness id with `skills::Agent`
  is *not* in this plan; it is named debt for after SL-012 lands.
- **codex is provisional.** Its adapter ships behind the seam, but the unbounded
  staleness (no SessionStart equivalent) means PHASE-06's live codex run may cut
  it from v1, keeping the seam for a later harness. Plan for both outcomes.
- **Empirical gates ride into closure:** the load-order ordeal and the matcher-
  token confirmation cannot be settled analytically — they belong to PHASE-06's
  user-run verification, not to any earlier exit criterion.
