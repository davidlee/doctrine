# doctrine-partner skill subset and route comprehension/posture provision

## Context

Two new collaboration skills were authored this session: `pair` (calibrated
adversarial pair programming) and `walkthrough` (guided, expertise-reversal-aware
code/artifact comprehension + critique). They were scaffolded as a freestanding
`plugins/partner/` plugin (interim, uncommitted — see Assumptions).

Two unresolved tensions drove this slice:

1. **Routing gap.** `/route` is the mandatory first gate but has no slot for
   *pure comprehension* intent ("walk me through this PR" — no change intended).
   A route-obeying agent mis-files it (preflight? canon?). Route also has no way
   to express that pairing/walkthrough are *conduct postures* orthogonal to the
   governance stage, not stages themselves.
2. **Coupling.** Closing the gap means `/route` (the embedded
   `install/routing-process.md` digest) referencing `/pair` and `/walkthrough`.
   Route today references only doctrine-plugin skills. A hard reference to a
   *separate* marketplace plugin risks a dead link when uninstalled.

The chosen resolution mirrors the existing `doctrine-memory` precedent: make the
skills part of the **doctrine plugin** (canonical source under
`plugins/doctrine/skills/`) and ship a `doctrine-partner` *standalone subset*
(plugin.json + README + symlinks back). Core can then safely assume `/pair` and
`/walkthrough` are installed, so route may hard-reference them. ADR-005
(shipped-knowledge tiering: skills route, reference docs explain) governs where
these sit — they are routing skills, correctly placed.

## Scope & Objectives

- **Relocate skills to canonical core.** Move `pair` and `walkthrough` skill
  source to `plugins/doctrine/skills/pair/` and `…/walkthrough/`.
- **Standalone subset.** Create `plugins/doctrine-partner/` mirroring
  `plugins/doctrine-memory/`: `.claude-plugin/plugin.json`, `README.md`, and
  `skills/{pair,walkthrough}` as symlinks into `../../doctrine/skills/…`.
- **Marketplace.** In `.claude-plugin/marketplace.json`, replace the interim
  `partner` entry with `doctrine-partner` ("standalone subset of the doctrine
  plugin; install one or the other, not both" — match the doctrine-memory entry).
- **Discovery exclusion.** Register `doctrine-partner` in
  `MARKETPLACE_ONLY_DOMAINS` (`src/skills.rs`) so the symlink-duplicated embed
  entries are dropped at discovery and do not collide on skill id; correct the
  now-false "(sole) marketplace-only domain" comment. Generalise it cleanly —
  the const is already a `&[&str]`.
- **Route provision** (`install/routing-process.md`, the embedded gate):
  - add a **comprehension-exit row**: *understand / audit an existing artifact,
    no change intended → `/walkthrough` (no slice)*.
  - add a **conduct-posture line**: pairing/walkthrough are postures orthogonal
    to the route stage — layer them on the chosen stage; a walkthrough-discovered
    change re-enters `/route`.
- **Re-embed + snapshot.** Touch `src/skills.rs` to force the RustEmbed recompile,
  `cargo build`, regenerate the boot snapshot (`doctrine boot`), and update boot
  goldens deliberately (behaviour-preservation gate).

## Non-Goals

- No change to the *content* of the pair/walkthrough skills beyond what's needed
  for relocation (their bodies were settled this session).
- No `--only-partner` install flag (the `--only-memory` analog). Out of scope
  unless a need appears; `MARKETPLACE_ONLY_DOMAINS` exclusion is sufficient.
- No new ADR — this rides ADR-005 tiering and the doctrine-memory precedent; no
  project-global decision is being made.
- No fix for the pre-existing boot drift (`Active Policies` unpopulated, stale
  snapshot) beyond the regenerate this slice already performs.
- No `/pair`/`/walkthrough` wiring into specific stage skills (execute, design) —
  posture is applied at the agent's discretion, not hard-wired.

## Affected surface

- `plugins/doctrine/skills/pair/SKILL.md`, `…/walkthrough/SKILL.md` (moved)
- `plugins/doctrine-partner/` (new: plugin.json, README.md, skill symlinks)
- `plugins/partner/` (interim — removed)
- `.claude-plugin/marketplace.json`
- `src/skills.rs` — `MARKETPLACE_ONLY_DOMAINS` + comment + catalog-exclusion test
- `install/routing-process.md` — routing digest (comprehension row + posture line)
- `plugins/doctrine/skills/route/SKILL.md` — second route surface, same additions
  (design §5.2c; kept in sync with the digest)
- `plugins/doctrine-memory/README.md` — OQ-1 (b)+ accuracy fix (folded in)
- boot snapshot + `src/boot.rs` goldens (regenerated)

## Risks / Assumptions / Open questions

- **Assumption (interim state).** `plugins/pair` was already `git mv`'d to
  `plugins/partner` with both `SKILL.md`s authored, and `marketplace.json` edited
  to `partner` — all uncommitted. This slice folds/resets that into the final
  `doctrine-partner` structure; net diff should show no `plugins/partner/`.
- **Risk (embed duplication).** RustEmbed follows symlinks → each subset skill
  emits twice (once canonical, once via the symlink). Mitigated exactly as
  doctrine-memory: discovery drops `MARKETPLACE_ONLY_DOMAINS`. Verify no id
  collision and the install catalog still excludes the subset.
- **Risk (re-embed footgun).** A lone `plugins/` edit does not re-embed on
  `cargo build`; `src/skills.rs` must recompile. Sequence is mandatory.
- **Resolved.** Boot golden asserts the routing digest by *presence only*
  (`contains("Route before you act")` at `boot.rs:945,1291`) — no verbatim
  sentinel; row/line additions are golden-safe (design §2, §10).
- **Resolved (OQ-1, b+).** doctrine-partner README written *accurately*
  (symlink source / resolved-copy distribution); doctrine-memory README corrected
  the same way in-slice — siblings agree on the truth, not a shared falsehood.

## Verification / closure intent

- `cargo build` + `cargo clippy` clean (zero warnings); `just check` green.
- `src/skills.rs` tests green incl. a new assertion that the install catalog
  excludes `doctrine-partner` (mirror the doctrine-memory case ~`skills.rs:887`)
  and no skill-id collision occurs.
- Behaviour-preservation: pre-existing skills/boot suites stay green (goldens
  updated only where the routing digest genuinely changed).
- `doctrine skills install` materialises `pair` + `walkthrough` from the doctrine
  domain; the `doctrine-partner` domain is not independently installed.
- Boot snapshot regenerated; routing table shows the comprehension-exit row.
- `/audit` → `/close` with the rollup reconciled.

## Follow-Ups

- Pre-existing boot drift (`Active Policies` unpopulated) — track separately.
- Possible `--only-partner` flag if standalone-partner installs become common.
