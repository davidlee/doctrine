# SL-080: Reconcile skill + audit/reconcile seam disentanglement

Backlog origin: **IMP-008** (`reconcile-skill-seam-disentangle`).

## Context

ADR-003 §7 draws a hard edge: `/audit` identifies spec changes and assembles
reconciliation context; `/reconcile` writes them — the **sole explicit writer**
of reconciled spec truth. ADR-009 §1 structures the closure seam topology
(the transition verb refuses `→reconcile` except from `audit`, `→done` except
from `reconcile`).

The seam is **half-built**: the FSM enforces the topology, but the `/reconcile`
skill and verb surface that give it teeth are the deferred machinery ADR-003 §11
names. Today `/audit` still writes spec/governance fixes in place (the "design
was wrong → reconcile `design.md`" disposition) instead of identifying only —
the over-reach ADR-009 §7 amendment records. And `/close` reconciles slice
*status* against the phase rollup, never the specs.

This slice enacts the missing half: the `/reconcile` skill, the reconcile
artefact, and the retuned `/audit` and `/close` skills that together make the
`audit → reconcile → close` seam real.

## Scope

### In scope

1. **`/reconcile` skill** — new skill under `.agents/skills/reconcile/SKILL.md`.
   - The writer half of the seam; sole spec-reconciliation writer.
   - Consumes: the RV ledger findings + the `## Reconciliation Brief` `/audit`
     assembled.
   - Writes via two surfaces: direct edit for per-slice artefacts (design.md);
     the REV kind (`doctrine revision`) for governance/spec truth (ADRs, specs,
     requirements, policies, standards).
   - Records the reconcile outcome: REV rationale (`revision-NNN.md`) for
     REV-backed changes; a `## Reconciliation Outcome` section on the RV for
     direct-edit changes.
   - Handles the `reconcile → design` escalation path when the model itself is
     inadequate (ADR-009 §1 back-edge), and the no-op case (no changes needed).

2. **Retune `/audit`** — strip spec/governance writing:
   - Audit identifies spec changes and assembles the `## Reconciliation Brief`
     (a dedicated section in the RV markdown, separate from `## Synthesis`).
   - Remove the "design was wrong → reconcile design.md" disposition
     that writes in place.
   - Audit's output: a resolved RV ledger + the reconciliation brief.
     Findings stay `verified`; remediation is recorded separately by reconcile.

3. **Retune `/close`** — spec-coherence closure:
   - `/close` must confirm reconciliation is complete before `done`.
   - Every governance/spec item is either covered by a `done` REV, withdrawn
     in the RV with rationale, tolerated with rationale, or escalated to design.
   - The existing structural closure seam is the mechanical backstop; the skill
     adds the substantive check.

4. **Routing wire** — add the `/reconcile` row to `install/routing-process.md`
   (only when the skill lands — shipped-not-reachable rule from ADR-009
   Verification F14). Verify the generated `boot.md` includes the row.

### Out of scope

- **`slice reconcile` CLI verb** — the verb surface the skill drives. Deferred
  per ADR-003 §11; the skill operates manually (like `/audit` does today) until
  the verb lands separately.
- **REV target for `doc/*`** — project-local design notes are not a formal
  doctrine feature category; they don't belong in the REV target set. They are
  edited directly like any project file, not treated as an evergreen category.
- **Tech-spec reconciliation engine** — no tech specs exist yet (ADR-003 §9).
- **Coverage derivation engine** — ADR-009 §3 deferred; reconciliation does
  not derive requirement status from coverage.
- **Conduct enforcement** — `/reconcile` defaults to `gate` per ADR-009 §2
  (advisory only); enforcement is deferred.

## Affected surface

- `.agents/skills/audit/SKILL.md` — retune to identification-only
- `.agents/skills/close/SKILL.md` — add spec-coherence check
- `.agents/skills/reconcile/SKILL.md` — new
- `install/routing-process.md` — add `/reconcile` routing row
- `.doctrine/state/boot.md` — regenerated to pick up routing change

## Risks & Assumptions

- REV discovery (mapping a slice to its REV) is by convention
  (`reconcile-sl-NNN` slug) or recorded in the reconciliation brief; no formal
  slice↔REV relation edge exists yet. Reconcile must guard against slug
  collisions (inspect before reuse).
- No `slice reconcile` CLI verb means the skill is manual discipline; the
  seam is structural at the FSM level but not automated.
- The existing close SKILL.md may have already been partially retuned since
  IMP-008 was filed — verify against the stale-prose list in IMP-008 before
  writing.

## Verification / Closure Intent

- `/reconcile` skill exists and is correctly routed (boot.md row confirmed in
  generated snapshot, not just in source).
- `/audit` skill prose describes identification-only, no in-place writing;
  includes reconciliation brief step.
- `/close` skill includes spec-coherence check (REV done or RV-native
  disposition for every item).
- All three skills form a coherent chain: audit identifies → reconcile writes →
  close confirms.
- Lint: zero warnings on all skill SKILL.md files (YAML frontmatter + prose).
- `doctrine claude install` succeeds and embeds the new skill.
