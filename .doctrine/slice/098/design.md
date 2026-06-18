# Design — SL-098: Requirements discovery and home-finding (redesign)

*Redesign after RV-078 inquisition. Eight findings resolved: F-1 (wrong baseline
— plugins not .dirge), F-2 (structured prose), F-3/F-5 (dead plan.toml fields),
F-4 (altitude gap), F-6 (brief position), F-7 (non-incremental walkthroughs),
F-8 (rotting anchors).*

## 1. Approach

No new code. The entity machinery — `spec req add`, `spec new`, `doctrine link`,
REV `introduce`/`create`/`move` — already supports the operations needed. What's
missing is skill guidance that makes implied-requirement discovery and orphan
placement a natural part of every pass through the reconcile loop.

The design works through each affected skill in dependency order. The key insight:
requirements travel as **metadata** (TOML sidecar + prose references) through
design→plan→audit, and only materialize as canonical `REQ-NNN` entities in
reconcile, when their spec home is known. This sidesteps the "`spec req add`
requires a SPEC_REF" constraint without any code change.

**Skill file baseline.** The authoritative source is `plugins/doctrine/skills/*/SKILL.md`.
The `.doctrine/skills/` and `.dirge/skills/` directories are transient install
targets — they must never be edited directly. The current plugins design skill has
**no** requirements pass, **no** "collect decisions" state, and asks questions one
at a time. Both are new additions to the authoritative file. (Prior SL-098 work
modified `.dirge/skills/` — a transient copy — which is why RV-078 saw a
duplicate; the plugins version is the truth.)

The `/plan` skill (plugins) says `specs`/`requirements` stay empty in v1. This
design respects that constraint and routes through `plan.md` prose instead.

## 2. The `design-requirements.toml` sidecar

The entity-model storage rule (doc/entity-model.md) requires structured data in
TOML, never prose. Implied requirements are discrete, countable items with typed
fields — they are structured data, not narrative.

**New file:** `slice/NNN/design-requirements.toml` — sister to `design.md`:

```toml
# design-requirements.toml — implied requirements surfaced during design.
# Handles are local to this design (REQ-D01, REQ-D02, …). No canonical entity
# exists yet — reconcile creates REQ-NNN when the spec home is known.

[[implied]]
handle = "REQ-D01"
statement = "Every state mutation through the reconcile writer must produce an immutable record with timestamp and actor identity."
kind = "quality"
home_hint = "new tech spec at container level"
descends_from = "Decision §4 — the write-seam must be auditable"

[[implied]]
handle = "REQ-D02"
statement = "Close must refuse when plan targets an orphan with no REQ-NNN mapping."
kind = "constraint"
home_hint = "existing tech spec at component level (the reconcile engine)"
descends_from = "Decision §7 — the orphan deadlock gate"
```

In `design.md`, the `## Implied Requirements` section becomes a narrative
reference — it names each handle and gives a one-line summary, pointing to the
TOML for the authoritative record. It does **not** repeat the structured fields:

```markdown
## Implied Requirements

This design surfaces two implied requirements (see `design-requirements.toml`):
- REQ-D01: Audit trail retention — immutable record for every write
- REQ-D02: Orphan detection gate — close must refuse unplaced orphans
```

The TOML is author-facing today (no tooling consumes it), but it is correctly
formatted for future tooling. This satisfies the storage rule: structured data
in TOML, prose reference in MD. The LLM reads both files — the TOML for
authoritative field values, the MD for narrative context.

## 3. `/design` skill — add collect-decisions and requirements-pass states

The plugins design skill (authoritative) has this state machine:

1. Explore (read design.md, survey, understand)
2. Ask clarifying questions (one at a time)
3. Propose 2-3 approaches
4. Present design
5. Write design.md
6. Adversarial Review
7. Integrate Review Feedback
8. Transition to planning

This design inserts two new states between Explore (1) and Propose (3):

### New state 2: Collect decisions

> 2. **Collect decisions.** Read the current state of the design doc. Identify
>    every open design question — choices not yet made, tradeoffs not yet
>    resolved, constraints not yet accepted. Survey implications: what entity
>    constraints, existing REQs, or ADR clauses bear on each question.
>
>    Batch all decisions together. Present them as a structured list with
>    options, tradeoffs, and your recommendation for each. Ask the user to
>    confirm or correct each one.
>
>    **Guardrail:** Batch ALL decisions before presenting. Do not badger the
>    user with each question — collect first, then present.

### New state 3: Requirements pass

> 3. **Requirements pass.** Before proposing approaches, inventory the
>    requirements picture:
>    - Which existing canonical REQs (REQ-NNN) are impacted?
>    - Which implied requirements does this design's decisions demand that no
>      existing REQ captures? These are **orphan requirements** (REQ-DNN).
>    - Ask the user: are there requirements they see that aren't captured?
>
>    Record in `design-requirements.toml` (authoritative) and reference from
>    `design.md` `## Implied Requirements` with one-line summaries.
>
>    **Guardrail:** Do not record implied requirements as structured fields in
>    prose — use `design-requirements.toml` (§2).
>
>    **Guardrail:** If an implied requirement crosses the altitude boundary
>    (product vs C4 level, or crosses spec kinds), note the ambiguity in the
>    TOML `home_hint` field — don't force a decision here. `/reconcile` handles
>    placement.

### Change to state 6 (adversarial review)

Add one attack vector:

> - "What requirements are implied by these decisions but not captured in
>   `design-requirements.toml`?"

### Resulting state numbering

After insertion: 1 Explore → 2 Collect decisions (new) → 3 Requirements pass
(new) → 4 Propose 2-3 approaches → 5 Present design → 6 Write design.md → 7
Adversarial Review → 8 Integrate Review Feedback → 9 Transition to planning.

The original "Ask clarifying questions" state is absorbed into "Collect
decisions" — the user still gets asked, but as a batched confirmation rather
than serial one-at-a-time badgering.

## 4. `/plan` skill — verification mapping in plan.md prose

The current plan skill (line 40) states: `specs` / `requirements` stay empty in v1
(no registry yet). This design **respects that constraint**: `plan.toml`
`[requirements]` stays empty. The close gate reads `plan.md` instead.

**Changes to plan skill:**

After step 1 (read design), add a new sub-step before scaffolding (step 3):

> 2a. **Requirements mapping.** Read `design-requirements.toml` — collect every
>     `REQ-DNN` handle. For each, determine which phase(s) will verify it. Read
>     any canonical REQs cited in the design. Present the mapping to the user
>     for confirmation.
>
>     Record the mapping in `plan.md` under a `## Requirements verification` section:
>
>     ```markdown
>     ## Requirements verification
>
>     | Handle | Verifying phase(s) | Note |
>     |---|---|---|
>     | REQ-D01 | PHASE-01, PHASE-02 | Audit trail retention — write-seam + record format |
>     | REQ-D02 | PHASE-03 | Orphan detection gate — close gate implementation |
>     | REQ-077 | PHASE-01 | Canonical REQ cited by design |
>     ```
>
>     This section is prose (the table is presentational, not machine-read). The
>     close gate reads it to confirm every `REQ-DNN` has a `→ REQ-NNN` mapping.
>
>     **Guardrail:** If a `REQ-DNN` has no phase assigned, that's a design gap —
>     surface as a `/consult` trigger. Do not drop the requirement silently.

**Note on enforcement:** `plan.md` is prose — the table is presentational, not a
queryable registry. The close gate (§7) is the single enforcement point. No
intermediate stage validates consistency between `design-requirements.toml`,
`plan.md`, and the reconciliation brief — the close gate catches gaps before
`done`.

## 5. `/audit` skill — orphan awareness in the brief

**Changes to audit skill:**

Add a sub-step in the audit process (between evidence gathering and synthesis):

> **4a. Orphan survey.** After raising findings, before synthesis:
>
> 1. Read `design-requirements.toml` — collect every `REQ-DNN` handle.
> 2. Cross-reference: has a canonical `REQ-NNN` already been created for it?
>    (Unlikely in first pass; possible in later loop iterations.)
> 3. For each still-orphaned requirement, add an entry to the reconciliation
>    brief.

**Orphan entries in the reconciliation brief nest under `Governance/spec (REV)`:**

The audit skill already defines two reconciliation brief groups: `Per-slice
(direct edit)` and `Governance/spec (REV)`. Orphan requirements require REV
`introduce`/`create` rows — they belong under `Governance/spec (REV)`, as a
sub-section:

```markdown
## Reconciliation Brief

### Per-slice (direct edit)
- …

### Governance/spec (REV)

- ADR-006 §D5: …

#### Orphaned requirements (REV introduce)
- REQ-D01: "Audit trail retention" — quality, likely tech spec at container level.
  Descends from Decision §4. No canonical REQ yet. See `design-requirements.toml`.
- REQ-D02: "Orphan detection gate" — constraint, likely existing tech spec.
  Descends from Decision §7. No canonical REQ yet. See `design-requirements.toml`.
```

**Edge cases:**

- **Legacy design with no `design-requirements.toml`:** Not an error — the orphan
  survey yields nothing. If the auditor spots an implied requirement in the prose,
  it becomes a regular finding, dispositioned normally.
- **Orphan in TOML but not in plan.md:** A `REQ-DNN` defined in
  `design-requirements.toml` but absent from `plan.md`'s verification table is a
  gap. Flag as a finding — audit identifies it; reconcile decides.

## 6. `/reconcile` skill — orphan placement workflow

**New step, between no-op gate (current step 2) and per-slice edits (current step 3):**

> **2b. Orphan placement.** For each orphaned requirement in the brief:
>
> ### Determine spec home(s)
>
> Survey existing specs. For each candidate:
> - Does the spec's scope cover this requirement's subject?
> - Is the spec's C4 level / product altitude right for this requirement?
> - Would the requirement fit coherently alongside existing requirements?
>
> If no existing spec fits, propose creating a new one. If a spec almost fits
> but is too broad, consider splitting.
>
> **Altitude assessment** is reconcile's responsibility. The audit brief's home
> hint is a starting point, not binding.
>
> **⚠️ Altitude framework gap.** No altitude decision framework exists (tracked
> as IMP-097). Until it does, rely on existing spec `c4_level`/`descends_from` as
> reference points and on domain reasoning. When the altitude question is
> genuinely ambiguous, trigger `/consult` rather than guessing. Do not force
> placement when no natural home exists — a stuck orphan is a design smell, not
> a reconcile failure.
>
> ### Work with existing requirements
>
> When the design cited existing `REQ-NNN` entries:
> - Verify they're still current. If a cited REQ has drifted, propose REV `modify`.
> - If an implied `REQ-DNN` duplicates an existing canonical REQ, map the orphan to it.
> - If an existing REQ is in the wrong spec, REV `move`.
>
> ### Handle multi-spec placement
>
> A requirement may belong in multiple specs (PRD + component spec). This is
> legal — the composition seam supports multi-membership. Each placement is a
> separate REV `introduce` row.
>
> ### Negotiate wording
>
> The requirement's statement may need rewording for its new home. Present
> proposed wording. If rewording materially changes the obligation, `/consult`.
>
> ### Author REV changes
>
> Each placement becomes a REV `introduce` row. If a new spec is needed, add a
> `create` row. Multiple `introduce` rows for the same requirement across specs
> share the same `REQ-NNN`.
>
> ### Record the mapping
>
> In the REV's reconcile narrative (`revision-NNN.md`):
>
> ```markdown
> ### Orphan placements
> - REQ-D01 → REQ-201 (FR-004 in SPEC-018): audit trail retention
> - REQ-D02 → REQ-202 (FR-005 in SPEC-003): orphan detection gate
> ```

## 7. `/close` skill — orphan deadlock gate

**Addition to spec-coherence gate (close skill step 2):**

Before allowing `done`, add a sub-check:

> **Orphan deadlock check.** Read `plan.md` `## Requirements verification` —
> collect every `REQ-DNN` handle listed. For each:
> - If the reconciliation outcome records a `REQ-DNN → REQ-NNN` mapping → pass.
> - If the reconciliation outcome records "withdrawn" with rationale → pass.
> - Otherwise → **refuse close.** The orphan is unplaced. Return to `/reconcile`.

This is the sole enforcement point. `plan.md` is prose (the table is
presentational), and the close gate reads it as the LLM reads any prose — it does
not depend on structural parsing.

## 8. Skill file changes

| Skill | Change |
|---|---|
| `/design` | Refine state 3 (Requirements pass): `design-requirements.toml` instead of prose fields. Add adversarial review vector. Add guardrail. |
| `/plan` | New sub-step 2a (Requirements verification table in `plan.md`). Acknowledge `plan.toml` `[requirements]` stays empty. |
| `/audit` | New sub-step 4a (orphan survey). Orphan section under `Governance/spec (REV)` in brief. Legacy design handling. |
| `/reconcile` | New step 2b (orphan placement workflow: home, existing-REQ, multi-spec, wording, REV, mapping). `/consult` guardrail for altitude gap. |
| `/close` | Orphan deadlock check reading `plan.md` verification table. |

## 9. Memories

After skill edits are complete, record:

- **Pattern:** The full `REQ-D → REQ-NNN` lifecycle — design discovery (TOML sidecar), plan verification mapping (plan.md prose), audit surfacing (brief under Governance/spec), reconcile placement (REV), close deadlock check.
- **Concept:** `design-requirements.toml` — format, relationship to `design.md`, handoff points.
- **Concept:** `REQ-DNN` as a local design-scoped handle — what it is, what it isn't, when it materializes.
- **Signpost:** IMP-096 (requirements-capture skills) and IMP-097 (altitude framework).

## 10. Follow-up

- **IMP-096:** Requirements capture and refinement skills. Their integration touchpoints are the `design-requirements.toml` file, the `plan.md` verification table, the audit brief orphan section, and the REV `introduce` path in reconcile.
- **IMP-097:** Altitude assessment framework. Blocking dependency for reliable orphan placement — proceed with `/consult` guardrail until resolved.

## 11. Walkthrough — incremental per-skill scenarios

Each scenario shows what the skill produces in isolation, building toward the
full end-to-end flow. Use these as verification targets when editing each skill.

### 11a. After `/design` edit only

**Given:** A slice needing an audit trail for the reconcile writer. No existing
REQs cover this.

**When:** `/design` runs through its state machine — explore, collect decisions,
requirements pass (state 3), propose approaches, present design, write.

**Then:**
- `design-requirements.toml` exists with two `[[implied]]` rows (REQ-D01, REQ-D02).
- `design.md` `## Implied Requirements` references both handles with one-line
  summaries — no repeated structured fields.
- Adversarial review (state 7) includes the requirements attack vector.

**Verify:** `design-requirements.toml` is valid TOML. `design.md` `## Implied
Requirements` section contains no `**Statement:**` / `**Kind:**` / `**Home:**`
field blocks.

### 11b. After `/plan` edit only

**Given:** The artefact state from 11a (design-requirements.toml + design.md).

**When:** `/plan` reads design, runs sub-step 2a (requirements mapping),
scaffolds plan, writes plan.md.

**Then:**
- `plan.md` contains a `## Requirements verification` table mapping REQ-D01 and
  REQ-D02 to phases.
- `plan.toml` `[requirements].targets` is **empty** (respects current constraint).
- No `REQ-DNN` is unassigned.

**Verify:** `plan.md` table has one row per `design-requirements.toml` `[[implied]]`
row. `plan.toml` `[requirements]` is either absent or empty.

### 11c. After `/audit` edit only

**Given:** Artefact state from 11b, plus a completed implementation.

**When:** `/audit` runs — evidence gathering, findings, orphan survey (new sub-step
4a), synthesis, brief.

**Then:**
- Reconciliation brief `### Governance/spec (REV)` contains a
  `#### Orphaned requirements (REV introduce)` sub-section.
- Each orphan entry cites its `REQ-DNN` handle and the `design-requirements.toml`
  source.
- A legacy design with no `design-requirements.toml` produces no orphan section.

**Verify:** Orphan section is nested under `Governance/spec (REV)`, not a
top-level section.

### 11d. After `/reconcile` edit only

**Given:** Artefact state from 11c (brief with orphan section).

**When:** `/reconcile` runs — reads brief, no-op gate passes (brief has items),
orphan placement (new step 2b), per-slice edits, REV authoring.

**Then:**
- For greenfield (no existing specs): reconcile proposes a new spec, places both
  orphans via REV `create` + `introduce` rows.
- For retrofit (existing spec as input): reconcile cites existing REQs, places
  new orphans, maps duplicates to existing REQs.
- REV narrative records `REQ-DNN → REQ-NNN` mappings.
- Altitude decision is documented; `/consult` fires if ambiguous.

**Verify:** Every orphan in the brief has either a mapping or a "stuck —
/consult" note. No orphan is silently dropped.

### 11e. After `/close` edit only — full end-to-end

**Given:** Artefact state from 11d (all orphans placed, REV done).

**When:** `/close` runs — pre-check, spec-coherence gate (with new orphan
deadlock check), commit, transition.

**Then:**
- Close reads `plan.md` `## Requirements verification`, collects `REQ-D01`,
  `REQ-D02`.
- Both have `→ REQ-NNN` mappings in the reconciliation outcome → gate passes.
- If an orphan were unplaced, close refuses with "unplaced orphan REQ-D03".

**Verify (negative case):** Remove one `REQ-D → REQ-NNN` mapping from the
reconciliation outcome. Close must refuse. Restore it — close must accept.
