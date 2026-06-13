# SL-055 Design — Holistic skills review & token-efficient improvements

This is an **orchestration design**, not an algorithm design. The slice produces
a critical review of the whole skill corpus and homes the improvements, run as a
paired Fable session. The design's job is to fix *how the work flows* so Fable's
context stays lean: expensive reading is front-loaded onto cheap external
research + Opus synthesis, and Fable arrives to a pre-digested, ranked evidence
base it only has to adjudicate.

## §1 Disposition model (the spine)

Every finding carries `{dimension, evidence, severity, disposition, confidence,
affected-skills}`. **Disposition is a routing tag, not a fix.** Findings are
**problem-only** — the synthesiser names and ranks; Fable authors the fix live.

Three dispositions, Fable adjudicates each (research pre-tags a guess):
- `prose` — a small skill-text edit. The default; this is mostly a prose pass.
- `dispatch` — needs a real code/CLI change. Fable MAY spawn a subagent to land
  it *within this slice*, or push it to backlog if too involved.
- `backlog` — too big for this pass → `backlog new` (IMP/ISS/CHR), referenced
  from the skill prose where relevant.

This softens the slice's prose-only non-goal: prose by default, structural fixes
are Fable's call, backlog is the escape hatch.

## §2 Evidence base — gitignored research dir

`.doctrine/slice/055/research/` (handover tier, gitignored via
`.doctrine/slice/*/research/`). Disposable; durable findings graduate to backlog
items / `notes.md` at close.

```
research/
  BRIEF.md      authored task spec for the research wave (committed? no — gitignored
                with the dir; it is the disposable work order)
  CORPUS.md     concatenated 26 skills + reference docs — the research-wave input
  <deepseek>.md research-wave output: R1–R5 sections (problem-only, tagged)
  00-index.md   Opus synthesiser output — ranked, deduped findings. THE lean read
                for Fable. Everything else is backing detail.
```

## §3 Subagent fan-out — two waves

Matches the cost split: cheap+fast for mechanical extraction, Opus for judgement.

**Research wave (external — DeepSeek, user-operated).** Smarter/faster/cheaper
than an in-session Haiku subagent, and runs async off the agent's context
entirely. Axes (see `BRIEF.md`):
- R1 inventory (census) · R2 vocab/term drift · R3 staleness (vs accepted ADRs +
  CLI) · R4 boot/reference-doc duplication · R5 lifecycle/cross-ref mention
  extraction (mechanical — feeds S2).

**Synthesis wave (in-session — Opus subagents).** Depends on the research wave:
- S1 overlap/boundary — routing-table coherence, skill responsibility overlap &
  gaps (reasons over R1).
- S2 **lifecycle seam matrix** (§5) — the hygiene workstream (reasons over R5).
- S3 synthesiser — folds R1–R4 + S1–S2 into `00-index.md`: dedupe, rank by
  severity×confidence, attach disposition guesses.

## §4 Review dimensions

structural consistency · vocabulary drift · boundary overlap · boot/reference
duplication · staleness · token bloat · **lifecycle hygiene (primary)**.

## §5 Lifecycle seam matrix (S2 detail)

For each skill: `transitions/cross-refs it owns × the obligation it should remind
about × what it currently says`. Gaps are the highest-value findings. Seeds:
- `/close` → close the originating backlog item (the IMP/ISS that spawned the
  slice); confirm rollup; reconcile lifecycle status.
- `/design` entry → `slice status <n> design`; `/plan` → `plan`/`ready`;
  `/execute` → `started`; `/audit` → `audit`; `/close` → `reconcile`→`done`.
  Skills should *drive* the ADR-009 transitions, not leave them hand-edited
  (which is what produces the SL-009 `⚠` rollup divergence).
- post-merge → branch / dispatch-worktree cleanup (`/dispatch`, `/worktree`).

## §6 Apply & re-embed

Edit the real dirs under `plugins/doctrine/skills/` (partner/memory symlinks
inherit automatically). Re-embed = touch `src/skills.rs` + `doctrine skills
install` (`mem.pattern.distribution.skill-refresh-command`). The `description:`
frontmatter is the auto-trigger surface — edits there are behavioural.

## §7 Phases (light — OQ-1 resolved: minimal ceremony)

- **PHASE-01 Research base** — deliver `00-index.md`. EN: BRIEF+CORPUS exist.
  EX: research-wave output present, S1–S3 run, `00-index.md` ranked & tagged.
  VT: none (no code) — VA: index covers all 26 skills, every finding tagged.
- **PHASE-02 Paired review & improvements** — `/pair` with Fable: walk
  `00-index.md` top-down, triage each finding's disposition, apply `prose`
  edits, `dispatch`/`backlog` the rest. EX: every hi/med finding dispositioned
  and either applied, dispatched, or backlogged. VA: human-approved in session.
- **PHASE-03 Re-embed & close** — re-embed ritual, sanity-check descriptions
  still trigger, `/audit` → `/close`. VT: `doctrine skills install` clean,
  `just check` green (if any code touched via dispatch).

## §8 Risks

- R1 (carried) — editing `description:` changes routing; treat as behavioural.
- R2 — research wave is external/unverifiable; S3 synthesiser must not trust R3
  staleness flags blindly — confirm against `doctrine --help` + ADR list before
  tagging hi.
- R3 — token bloat fix vs information loss: trimming a skill that recites boot
  governance is safe *only because* boot is always injected; verify the snapshot
  actually carries the recited content before cutting (`mem.pattern.distribution
  .shipped-not-reachable` — the inverse risk: don't cut something nothing else
  carries).

## §9 Adversarial self-review — findings integrated

- **A1 — external research wave is a single point of failure.** If DeepSeek's
  output is malformed or thin, PHASE-01 stalls. *Mitigation:* the research wave
  is a *convenience*, not a dependency — the in-session Opus synthesis subagents
  can read `CORPUS.md` directly and do R1–R5 themselves as fallback. DeepSeek
  just makes the cheap part cheaper. S3 must not blindly trust R3 staleness
  flags (R2 risk) — re-confirm against `doctrine --help` + the ADR list.
- **A2 — `00-index.md` could itself become a token-bloat hog** (7 dimensions ×
  26 skills). The "one lean read" claim fails if it's a flat dump. *Mitigation:*
  the index is **severity-tiered** — hi/med findings inline with evidence; `lo`
  collapsed to a one-line-per-skill appendix. Fable walks hi→med; lo is opt-in.
- **A3 — PHASE-02 prose pass has no mechanical gate.** "Human-approved" is soft
  and skill cross-refs can rot silently. *Mitigation:* PHASE-03 adds a mechanical
  check beyond `skills install` clean — every skill named in the routing table
  and every `/skill` cross-reference still resolves to an existing skill dir
  (a grep-level integrity sweep), and every `description:` is non-empty.
