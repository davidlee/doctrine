# Implementation Plan SL-018: Shipped orientation memory corpus

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Five phases turn the locked design into shippable work (PHASE-06 is a
post-closure maintenance addition — see its entry below). Two facts shape the
original sequence:

1. **Scripture leads (the gate).** The corpus is spec-violating until sanctioned
   and the `repo=""` admission path is unblessed, so the ADR + memory-spec
   amendment is PHASE-01 — canon before code, canon before a single master.
   Nothing downstream may begin until the class and its evergreen-staleness
   disposition are in scripture (design D10, inquisition Charge II).

2. **Mechanism before content, fixtures before corpus.** The retrieval reach,
   the sync verb, and the authoring path are all testable against *synthetic*
   fixtures and an *empty* embed. So the plumbing (PHASE-02..04) lands and goes
   green without a single real master; the corpus (PHASE-05) is authored last,
   against tooling already proven. This keeps each phase independently green and
   defers the largest hand-authoring effort behind the gates that make it safe.

The split also honours the behaviour-preservation gate: the only shared
machinery touched is the memory indexer, and it is touched by *addition*
(`collect_all` over the unchanged `collect_memories` leaf), so the SL-005/007/008
suites stay green unchanged as the proof.

## Sequencing & Rationale

**PHASE-01 — Sanction the class (GATE).** Scripture only: an ADR blessing the
global/unanchored/path-scoped/derived class *and* its evergreen/reference-grade
staleness exemption, plus the memory-spec amendment that reconciles the §306
contradiction and records the admission path. No master, no code, no test. It
exists so every later phase rides canon rather than improvising past it. Placed
first because the corpus authoring (PHASE-05), the admission golden and the
staleness code (PHASE-02) all depend on the sanction it lands.

**PHASE-02 — Retrieval reach.** The "whole point" of the slice is *scoped
retrieval* of shipped memories, so the read/index surface is established before
the write path that fills it. Preserve the leaf, add the `collect_all`
composite, switch the three callers, fix `read_body`'s cross-root fallback, pin
the required admission golden (the dormant `repo=""` hatch is newly lit — there
is no lived baseline, so it is pinned, not assumed), and implement the evergreen
staleness disposition gated by PHASE-01. All fixture-tested; no embed needed.
The moment the sync verb lands in PHASE-03, its output is already retrievable.

**PHASE-03 — The sync mechanism.** The write path: a new RustEmbed over repo-root
`memory/` (empty for now), the pure idempotent `plan_corpus` diff, the impure
`sync_corpus` shell with a *bounded* prune (only INV-signatured orphans; foreign
and unparseable left untouched), the `memory sync` verb with `--dry-run`/`--yes`,
the separate-entry SessionStart hook (`sync install`, M1 — degrading to a clean
no-op in non-doctrine repos), and the gitignore wiring on both surfaces (the
inverted authored-entity-wiring trap). Modelled on the skills materialize
precedent, deliberately *not* folded into install/boot whose contracts are the
opposite (never-overwrite vs refresh). Empty-embed-tolerant so it goes green
before the corpus exists.

**PHASE-04 — Authoring path + validation.** Before the corpus can be authored it
needs a sanctioned minting path and a gate that keeps it honest: `record
--global` (the declared escape hatch that writes a `repo=""`/anchor=none master
into `memory/`, bypassing the repo-anchor gate by explicit intent without
relaxing the normal path) and master-lint (INV signature, valid `memory_type` ≠
the `reference` literal, the ≥1 path/glob/command scope floor). OQ-C is resolved
here: install prints a hint, it does not orchestrate sync. The lint runs green
against fixtures now and becomes load-bearing over the real corpus next.

**PHASE-05 — Author the corpus (content).** The slice may not go green on
plumbing alone. Triage all 86 spec-driver memories into a disposition table,
author ~12-18 doctrine masters covering every skeleton topic via `record
--global`, lint the whole corpus, ship it through `memory sync`, and confirm the
reach end-to-end (retrieve `--path-scope`, body render, non-decaying `reference`
staleness, boot listing). The audience is the downstream agent *driving*
doctrine — doctrine-repo dev gotchas are explicitly dropped to `items/`, not
shipped. The corpus orients toward boot/skills/`doc/*`, it does not restate them.

**PHASE-06 — Reconcile master-corpus drift (maintenance).** Appended after
closure. The masters are a *living* orientation surface, but they were authored
against doctrine as it stood at PHASE-05; SL-019..023 then grew the surface
(a `backlog` verb, the `spec/` + `backlog/` entity dirs, the spec authoring
skills) and three *enumerating* signpost masters — `cli-command-map`,
`file-map`, `skill-map` — fell behind. This phase re-aligns those three bodies
and re-syncs, riding PHASE-04's authoring path and the design's invariants
(master-lint, the no-restate principle) unchanged — no new mechanism. It also
dispositions the orientation-grade memories authored since closure, promoting to
master only by exception (the corpus stays lean). Placed last and kept narrow:
the non-enumerating masters (`overview`, `conventions`, `storage-model`, …) are
evergreen and stay byte-unchanged, so the behaviour-preservation gate is the
proof — only the three drifted bodies move.

## Notes

**Decisions taken at plan altitude (the design left these open, none
architectural):**

- **OQ-C (sync autorun):** install prints a hint; the verb stays standalone
  (skills parity). Resolved in PHASE-04.
- **OQ-E (M1 hook wiring):** a *separate* SessionStart entry for `memory sync`,
  not chained onto boot's — cohesion (sync and boot are independent surfaces);
  it degrades to a clean no-op in non-doctrine repos. Resolved in PHASE-03.
- **OQ-A (topic skeleton):** content, executed in PHASE-05; the design fixed the
  axes and the provisional skeleton, the plan does not relitigate them.

**Deliberately deferred (do not pull in):** M2 (staleness-reaction hooks) and M3
(override/suppress by `memory_key`; the genuinely-unanchored local-convention
gate-relax) go to a follow-up slice + a behaviour-hooks ADR. The `collect_all`
uid-dedup is v1 behaviour and the documented seam for M3's future key-precedence
pass — it is not itself the override mechanism.

**Do not relitigate** the locked decisions D1–D10 or the 12 dispositioned
charges; the plan refines them into phases, it is not higher authority than the
design or canon.
