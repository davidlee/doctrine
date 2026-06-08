# SL-023 Audit — Ship knowledge tiers (ADR-005)

Conformance audit, post-implementation. Reconciled against `design.md`, ADR-005
(+ its `inquisition.md` resolutions), and the phase `VT-`/`VA-` criteria. All four
phases `completed`; rollup 4/4.

## Evidence

| # | Check | Command / artifact | Result |
|---|---|---|---|
| E1 | PHASE-01 VT-1 | `install::tests::glossary_is_shipped` | ok |
| E2 | PHASE-02 VT-1 | `install::tests::using_doctrine_is_shipped` | ok |
| E3 | PHASE-03 VT-2 | `boot::tests::regenerate_projects_routing_digest_and_governance_body` | ok |
| E4 | PHASE-04 VT-1 | `skills::tests::dedup_skills_route_not_restate` | ok |
| E5 | Gate | `just check` (lint + test + fmt) | GREEN |
| E6 | ADR-005 VT (embed/ship) | real `install --yes` into a temp git repo | `.doctrine/{glossary,using-doctrine}.md` present |
| E7 | ADR-005 VT (push) | installed `.doctrine/routing-process.md` | reference-forms + reference-docs block present |
| E8 | ADR-005 VA (reachability) | `routing-process.md:40-41` | both pull-refs named in the push digest |
| E9 | ADR-005 VA (no --help dup) | grep flag-syntax in `glossary.md` + `using-doctrine.md` | CLEAN |
| E10 | Breadcrumbs | `slice-020.md`, `spec-012.md` | SL-023 breadcrumb present, originals intact |

## Findings

- **F1 — ADR-005 VT embed/ship. ALIGNED.** A real client install (E6) ships both
  the glossary and the operator's guide. The asset-level guards (E1, E2) lock the
  embed set against regression.
- **F2 — ADR-005 VT push presence. ALIGNED.** The reference-forms rules + pointer
  ride a regenerated boot snapshot (E3, E7); storage-rule + use-CLI were already
  resident and were not re-added (design D5 / R-C3).
- **F3 — ADR-005 VA reachability. ALIGNED.** Both pull-references are pointed-at
  from the resident push tier (E8) and each de-dup'd skill keeps a tier-1/2 pointer
  (E4 asserts it). No shipped-but-unreachable doc (the AGENTS.md failure averted).
- **F4 — ADR-005 VA no --help reproduction. ALIGNED.** Neither shipped doc carries
  a flag table (E9); the restate-line guard (E4) keeps the named skills clean.
- **F5 — De-dup evidence-bound. ALIGNED.** Exactly the six named sites (eight files)
  were touched; the MAY-permitted one-line pointers (`slice:37`, `plan:40`) and the
  `slice new "<title>"` incantation were left intact, per design D6 / R-C1.
- **F6 — entity-model dangling link (R-A3). ALIGNED.** The link was dropped; the
  glossary opening is self-contained — no dangling reference in the client copy.
- **F7 — pre-existing dead template path. ALIGNED + repaired.** The `doc/glossary.md`
  path the five shipped templates + `governance.md` carried pointed at a location
  absent from every client install; corrected to `.doctrine/glossary.md`. A latent
  defect fixed as a relocation ripple (design D3).

## Dispositions requiring action / disclosure

- **F8 — PUSH/glossary reference-form duplication (R-A5). TOLERATED DRIFT (conscious).**
  The reference-form *rules* now live in two homes: compact in the push digest, full
  (tables + examples) in the glossary. This is sanctioned tiering, not parallel
  implementation — PUSH cannot point-at a rule needed before any skill invoke. Risk:
  the two could drift. Mitigation in place: PUSH carries rule statements only (verified
  no table rows in the block). **Residual:** no automated test asserts the two stay
  congruent. Rationale for tolerating: the PUSH block is ~6 lines and rule-only; a
  congruence test would be brittle for little gain. Harvested as a follow-up candidate
  (F11).
- **F9 — retained flag *names* in kept prose. TOLERATED (within the ratified line).**
  `retrieve-memory` still names `--limit`/`--min-trust` and `record-memory §5` keeps a
  flagless `doctrine memory verify <UID|KEY>` block; `record-memory §4` names
  `trust_level`/`severity`. These were **not** in the named-offender set and are
  concept/verb references, not flag-syntax templates — permitted by R-OQ-4 (MAY cite
  by name). Disclosed for honesty; no action.
- **F10 — slice status divergence (`⚠`, SL-009). FIX NOW at /close.** `slice list`
  flags `proposed` (hand-edited) vs the 4/4 rollup. No lifecycle-transition CLI exists;
  reconcile `slice-023.toml status` to a terminal value at close.

## Follow-ups (harvested)

- **F11 (candidate, optional).** A congruence guard for the PUSH reference-forms block
  vs `glossary.md` — defer unless drift is observed. Not owned; low priority.
- **OQ-00N → bare-OQ-N corpus normalisation** — separate chore/slice, out of SL-023
  scope (design Follow-Ups). Not started.

## Observations (not findings)

- **Foreign WIP in the working tree.** `src/backlog.rs` (+114 lines, SL-022 backlog
  status⟺resolution transition) and `src/main.rs` carry uncommitted changes that
  predate this session. Left untouched and **excluded from every SL-023 commit** —
  each commit was staged by explicit path, not `git add -A`. Flagged for the owner.
- **Durable gotcha recorded:** `mem.pattern.build.jail-target-redirect` — `cargo build`
  writes to `~/.cargo/doctrine-target-jail/debug`; the in-repo `./target/debug/doctrine`
  is a stale separate file; embed-dependent runs must use the jail binary.

## Judgement

Audit-ready. Every ADR-005 Verification item is satisfied with traceable evidence;
all findings are dispositioned. One fix-now (F10, the status reconcile) is owned by
`/close`. Hand off to `/close`.
