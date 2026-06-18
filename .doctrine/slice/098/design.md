# Design — SL-098: Requirements discovery and home-finding (redesign)

*Redesign after RV-078 inquisition. Eight findings resolved: F-1 (wrong baseline
— plugins not .dirge), F-2 (structured prose), F-3/F-5 (dead plan.toml fields),
F-4 (altitude gap), F-6 (brief position), F-7 (non-incremental walkthroughs),
F-8 (rotting anchors).*

*Second-pass amendment after a post-plan technical review (no ledger raised).
Seven findings reconciled: B1 (multi-spec CLI semantics — §6), B2 (collect-decisions
vs clarifying-questions are distinct — §3), C1 (orphan placement relocated to
REV step 4f — §6), C2 (no-op gate amended for orphans — §6), D1 (plan.md
verification narrative is a trade, not a resolution — §4), E2 (stuck is
non-terminal; orphan withdrawal mechanism defined — §6/§7), F2 (close gate is
advisory, not enforced — §7). The plan mirrors this amendment.*

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

This design inserts two new states between Explore (1) and Propose (4).
"Collect decisions" and "Ask clarifying questions" are **distinct activities**
and both are kept (B2): clarification is an exploratory intent loop — questions
arise from earlier answers, so they stay one-at-a-time; collect-decisions is a
*decision-confirmation* loop — open decisions are already enumerable, so they
batch. The two do not merge.

### New state 2: Collect decisions

> 2. **Collect decisions.** Survey the design surface for every open decision
>    — choices not yet made, tradeoffs not yet resolved, constraints not yet
>    accepted. Where a design doc already exists (refinement, not greenfield),
>    read it; otherwise work from the slice scope and the Explore output.
>    Survey implications: what entity constraints, existing REQs, or ADR clauses
>    bear on each question.
>
>    Batch all decisions together. Present them as a structured list with
>    options, tradeoffs, and your recommendation for each. Ask the user to
>    confirm or correct each one.
>
>    **Guardrail:** Batch ALL decisions before presenting. Do not badger the
>    user with each decision — collect first, then present.
>
>    This state is a decision-confirmation loop, not a replacement for the
>    exploratory "Ask clarifying questions" loop (state 3). If surfacing a
>    decision reveals an unknown the user must resolve, hand to state 3 rather
>    than forcing a premature choice.

### Renumbered state 3: Ask clarifying questions (unchanged content)

The existing exploratory intent loop stays — questions one at a time, choosing
the most impactful, because later questions depend on earlier answers. It is
**not** absorbed into collect-decisions (B2). Renumber only.

### New state 4: Requirements pass

> 4. **Requirements pass.** Before proposing approaches, inventory the
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

### Change to the adversarial-review state

Add one attack vector:

> - "What requirements are implied by these decisions but not captured in
>   `design-requirements.toml`?"

### Resulting state numbering and the XML state machine

After insertion: 1 Explore → 2 Collect decisions (new) → 3 Ask clarifying
questions (renumbered, content unchanged) → 4 Requirements pass (new) → 5
Propose 2-3 approaches → 6 Present design → 7 Write design.md → 8 Adversarial
Review → 9 Integrate Review Feedback → 10 Transition to planning.

The skill carries **two** state representations that must stay in sync: the
numbered list above and the `<Process State Machine>` XML block (with `<state>`/
`<transition>` elements). The XML block must be edited in the same pass — its
states and transitions must reference "Collect decisions", "Ask clarifying
questions", and "Requirements pass", and the `Adversarial review → Ask
clarifying questions` back-edge must survive. An edit that touches only the
numbered list leaves a stale XML block (E1).

## 4. `/plan` skill — verification narrative in plan.md (a trade, not a resolution)

The current plan skill states: `specs` / `requirements` stay empty in v1 (no
registry yet). This design **respects that constraint**: `plan.toml`
`[requirements]` stays empty. The close gate reads `plan.md` instead.

**Storage-rule posture (D1, F2).** The plan skill step 5 prohibits *queried or
derived* data in `plan.md`. The REQ-D → phase mapping is neither: it is
**authored** (the planner writes it) and **agent-read** (no tool queries it; the
close gate reads it as the LLM reads any prose). It is, however, structured and
enforced-upon — the category RV-078 F-3 flagged. This design does **not** claim
F-3 is resolved; it **trades** it: the mapping lives as authored prose in
`plan.md` because (a) `plan.toml [requirements]` is the v1-empty registry, not a
verification map, and (b) `design-requirements.toml` is owned by `/design` and
exists before `/plan` runs, so the phase mapping cannot live there without
crossing the design/plan ownership boundary. When the requirement registry
lands, the mapping graduates to `plan.toml` and the prose is retired.

To keep the trade honest, the plan skill step 5 prohibition is amended to name
this as a permitted exception: *authored, agent-read verification narrative*
is distinct from *tool-queried/derived data*; the former is allowed in `plan.md`
prose, the latter is not. The mapping is written as **narrative prose with a
list** — not a pipe-table — to avoid implying machine-parseable structure.

**Changes to plan skill:**

After step 1 (read design), add a new sub-step before scaffolding (step 3):

> 2a. **Requirements mapping.** Read `design-requirements.toml` — collect every
>     `REQ-DNN` handle. For each, determine which phase(s) will verify it. Read
>     any canonical REQs cited in the design. Present the mapping to the user
>     for confirmation.
>
>     Record the mapping in `plan.md` under a `## Requirements verification`
>     section as **narrative prose with a list** (not a pipe-table — D1: a table
>     implies machine-parseable structure this design explicitly disclaims):
>
>     ```markdown
>     ## Requirements verification
>
>     This slice verifies the following implied requirements (handles from
>     `design-requirements.toml`) and cited canonical REQs:
>
>     - REQ-D01 (audit trail retention) — verified by PHASE-01 and PHASE-02
>       (write-seam + record format).
>     - REQ-D02 (orphan detection gate) — verified by PHASE-03 (close gate
>       implementation).
>     - REQ-077 (canonical, cited by design) — verified by PHASE-01.
>     ```
>
>     This section is authored, agent-read narrative — not tool-queried, not
>     derived. The close gate (§7) reads it as the LLM reads any prose to
>     confirm every `REQ-DNN` has a `→ REQ-NNN` mapping. See §4 storage-rule
>     posture for why this is a trade, not a resolution.
>
>     **Guardrail:** If a `REQ-DNN` has no phase assigned, that's a design gap —
>     surface as a `/consult` trigger. Do not drop the requirement silently.

**Note on enforcement:** `plan.md` is prose — the list is narrative, not a
queryable registry. The close gate (§7) is an **advisory agent-discipline
check** (F2), not a binary-enforced gate: nothing in the CLI refuses close on an
unplaced orphan; the §11e walkthrough is the backstop. No intermediate stage
validates consistency between `design-requirements.toml`, `plan.md`, and the
reconciliation brief — the close check catches gaps before `done`, when an agent
runs it. Filing the stronger enforcement (orphan status riding the RV ledger,
where the existing close-gate binary already enforces) as a follow-up IMP is the
long-term fix; SL-098 ships the advisory version.

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

## 6. `/reconcile` skill — orphan placement as a REV-authoring sub-step

**Placement: step 4f, not step 2b (C1).** Orphan placement produces REV
`introduce`/`create`/`move` rows — it *is* REV authoring. The reconcile skill
already has a clean two-surface split (step 3 per-slice direct edits, step 4 REV
authoring with 4a–4e). Inserting orphan placement at 2b would duplicate step 4
with no rule for how the two REV-authoring sites interact. Placing it as **4f**
(append to the existing 4a–4e sequence) keeps the two-surface split intact and
reuses the discover/collision/narrative/split machinery already there.

**No-op gate amendment (C2).** Step 2 fires when "the reconciliation brief is
empty — every finding was withdrawn or tolerated with no writes needed." Once
orphans exist, the brief's emptiness and the every-finding-terminal gloss
diverge: a slice with all findings tolerated but unplaced orphans has a
non-empty brief (orphan entries) yet satisfies the gloss. Read by gloss, an agent
could short-circuit to close and skip orphan placement — the exact failure this
slice exists to prevent. The skill's step 2 is amended with an explicit
guardrail:

> **Orphan content counts.** `#### Orphaned requirements (REV introduce)`
> entries are brief content. The no-op gate does **not** fire while any orphan
> is unplaced — proceed to step 4 (and 4f) even if every RV finding is terminal.

**New sub-step 4f — Orphan placement** (after 4e split rule, before step 5
approve & apply):

> **4f. Orphan placement.** For each orphaned requirement in the brief's
> `#### Orphaned requirements (REV introduce)` sub-section:
>
> ### Determine spec home(s)
>
> Survey existing specs. For each candidate:
> - Does the spec's scope cover this requirement's subject?
> - Is the spec's C4 level / product altitude right for this requirement?
> - Would the requirement fit coherently alongside existing requirements?
>
> If no existing spec fits, propose creating a new one (a `create` row in the
> same REV). If a spec almost fits but is too broad, consider splitting.
>
> **Altitude assessment** is reconcile's responsibility. The audit brief's home
> hint is a starting point, not binding.
>
> **⚠️ Altitude framework gap.** No altitude decision framework exists (tracked
> as IMP-097). Until it does, rely on existing spec `c4_level`/`descends_from`
> as reference points and on domain reasoning. When the altitude question is
> genuinely ambiguous, trigger `/consult` rather than guessing. Do not force
> placement when no natural home exists — a stuck orphan is a design smell, not
> a reconcile failure.
>
> ### Work with existing requirements
>
> When the design cited existing `REQ-NNN` entries:
> - Verify they're still current. If a cited REQ has drifted, propose REV `modify`.
> - If an implied `REQ-DNN` duplicates an existing canonical REQ, record the
>   `REQ-DNN → REQ-NNN` mapping in the narrative; no new row needed.
> - If an existing REQ is in the wrong spec, REV `move`.
>
> ### Handle multi-spec placement (B1)
>
> A requirement may belong in multiple specs (PRD + component spec). The CLI
> does **not** support attaching one `REQ-NNN` to a second spec: each
> `revision change add --action introduce` mints a new labelled requirement in
> one `--member-of` spec, and no re-member verb exists (verified empirically —
> no `REQ-NNN` appears in >1 spec's `members.toml`). Multi-spec placement is
> therefore expressed as **sibling requirements with traced lineage**: one
> `introduce` row per destination spec, each minting its own `REQ-NNN`, all
> traced back to the same `REQ-DNN` in the narrative. The genuine
> multi-membership question (one REQ in two rosters) is deferred to IMP-096.
>
> ### Negotiate wording
>
> The requirement's statement may need rewording for its new home. Present
> proposed wording. If rewording materially changes the obligation, `/consult`.
>
> ### Author REV rows
>
> Each placement becomes a REV `introduce` row in this REV (discovered at 4a,
> guarded at 4b). If a new spec is needed, add a `create` row. Multi-spec
> placement = multiple `introduce` rows, one per spec, each with its own
> `--new-label` and `--member-of` (B1).
>
> ### Record the mapping
>
> In the REV's reconcile narrative (`revision-NNN.md`) under `### Orphan
> placements`, recording the `REQ-DNN → REQ-NNN` lineage (one line per
> placement; multi-spec placements get one line per sibling REQ):
>
> ```markdown
> ### Orphan placements
> - REQ-D01 → REQ-201 (FR-004 in SPEC-018): audit trail retention
> - REQ-D02 → REQ-202 (FR-005 in SPEC-003): orphan detection gate
> - REQ-D03 → REQ-210 (FR-001 in SPEC-002), REQ-211 (FR-001 in SPEC-007):
>   traceability across product + component (multi-spec siblings, same REQ-D03)
> ```
>
> ### Stuck or withdrawn orphans (E2)
>
> An orphan that cannot be placed is **non-terminal** for close. Two outcomes:
>
> - **Stuck — `/consult`.** The altitude question is genuinely ambiguous. Record
>   the stuck state in the narrative. Close **refuses** a stuck orphan (it has
>   no `→ REQ-NNN` mapping and no withdrawal) and returns to reconcile. Stuck is
>   not a resting state; it routes through `/consult` to a placement or a
>   withdrawal.
> - **Withdrawn.** The orphan is retracted at the design level — edit
>   `design-requirements.toml` to remove or annotate the `[[implied]]` row with
>   rationale, and record the withdrawal in the narrative. Orphans are **not**
>   RV findings, so "withdrawn" here is a design-level retraction recorded in the
>   reconcile narrative, not the RV finding disposition. Close passes a
>   withdrawn orphan on the narrative record.
>
> The 4e split rule applies: a stuck orphan that will not land in this pass is
> split into its own REV before approve/apply, so it doesn't block the omnibus.

## 7. `/close` skill — orphan advisory check (F2)

**Addition to spec-coherence gate (close skill step 2):**

Before allowing `done`, add a sub-check:

> **Orphan check (advisory).** Read `plan.md` `## Requirements verification`
> and collect every `REQ-DNN` handle listed. For each, follow the read-path
> below to determine its outcome:
> - **Placed** — the reconciliation outcome records a `REQ-DNN → REQ-NNN`
>   mapping → pass.
> - **Withdrawn** — the reconciliation outcome records a withdrawal with
>   rationale → pass.
> - **Stuck** — the reconciliation outcome records "stuck — `/consult`" with no
>   mapping → **refuse close.** Return to `/reconcile`.
> - **Absent** — no outcome recorded → **refuse close.** Return to `/reconcile`.
>
> **Read-path (E3).** The reconciliation outcome lives in `review-NNN.md`
> `## Reconciliation Outcome` (written by reconcile step 6), which *points to*
> the REV narrative in `revision-NNN.md` `### Orphan placements` for the
> `REQ-DNN → REQ-NNN` mappings. Close reads `review-NNN.md` first; if the
> outcome references a REV for orphan placements, follow the pointer into
> `revision-NNN.md` to read the mappings. Do not read `revision-NNN.md`
> directly without the `review-NNN.md` pointer — the RV is the reconciled-truth
> surface.
>
> **Advisory, not enforced (F2).** This check is agent discipline, not a binary
> gate: nothing in the CLI refuses close on an unplaced orphan (the existing
> close-gate binary enforces only unresolved RV blockers). The §11e walkthrough
> is the backstop. The long-term fix — orphan status riding the RV ledger, where
> the existing close-gate binary already enforces — is filed as a follow-up IMP;
> SL-098 ships the advisory version and names it honestly.

## 8. Skill file changes

| Skill | Change |
|---|---|
| `/design` | New state 2 (Collect decisions) and new state 4 (Requirements pass); state 3 (Ask clarifying questions) renumbered, content unchanged (B2). `design-requirements.toml` sidecar as authoritative home; `## Implied Requirements` prose reference only. Adversarial-review attack vector. Guardrails (no structured prose; altitude ambiguity in `home_hint`). **Both** the numbered list and the `<Process State Machine>` XML block updated in sync (E1). |
| `/plan` | New sub-step 2a (Requirements verification narrative in `plan.md` as a prose list, not a table — D1). Acknowledge `plan.toml` `[requirements]` stays empty. Amend step 5 prohibition to permit authored, agent-read verification narrative as distinct from tool-queried/derived data. |
| `/audit` | New sub-step 4a (orphan survey; output held for brief-writing at step 5 — G4). Orphan section under `Governance/spec (REV)` as `#### Orphaned requirements (REV introduce)` (F-6). Legacy design handling. |
| `/reconcile` | No-op gate (step 2) amended: orphan content counts, gate does not fire while any orphan unplaced (C2). New sub-step 4f (orphan placement: home, existing-REQ, multi-spec as sibling REQs with traced lineage — B1, wording, REV rows, mapping, stuck/withdrawn — E2). `/consult` guardrail for altitude gap (IMP-097). |
| `/close` | Orphan advisory check in spec-coherence gate (step 2): placed/withdrawn pass, stuck/absent refuse (E2). Read-path `review-NNN.md` → `revision-NNN.md` (E3). Named honestly as advisory, not binary-enforced (F2). |

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

**When:** `/design` runs through its state machine — Explore → Collect decisions
(new state 2) → Ask clarifying questions (renumbered state 3) → Requirements pass
(new state 4) → Propose approaches → Present design → Write.

**Then:**
- `design-requirements.toml` exists with two `[[implied]]` rows (REQ-D01, REQ-D02).
- `design.md` `## Implied Requirements` references both handles with one-line
  summaries — no repeated structured fields.
- Adversarial review includes the requirements attack vector.
- **Both** the numbered state list and the `<Process State Machine>` XML block
  list "Collect decisions", "Ask clarifying questions", and "Requirements pass";
  the `Adversarial review → Ask clarifying questions` back-edge survives (E1).

**Verify:** `design-requirements.toml` is valid TOML. `design.md` `## Implied
Requirements` section contains no `**Statement:**` / `**Kind:**` / `**Home:**`
field blocks. The XML state block and the numbered list name the same states.

### 11b. After `/plan` edit only

**Given:** The artefact state from 11a (design-requirements.toml + design.md).

**When:** `/plan` reads design, runs sub-step 2a (requirements mapping),
scaffolds plan, writes plan.md.

**Then:**
- `plan.md` contains a `## Requirements verification` **prose list** (not a
  pipe-table — D1) mapping REQ-D01 and REQ-D02 to phases.
- `plan.toml` `[requirements].targets` is **empty** (respects current constraint).
- No `REQ-DNN` is unassigned.
- Step 5's storage-rule prohibition names the authored-agent-read-narrative
  exception.

**Verify:** `plan.md` list has one entry per `design-requirements.toml`
`[[implied]]` row. `plan.toml` `[requirements]` is either absent or empty. No
pipe-table in the verification section.

### 11c. After `/audit` edit only

**Given:** Artefact state from 11b, plus a completed implementation.

**When:** `/audit` runs — evidence gathering, findings, orphan survey (new sub-step
4a, output held for brief-writing at step 5 — G4), synthesis, brief.

**Then:**
- Reconciliation brief `### Governance/spec (REV)` contains a
  `#### Orphaned requirements (REV introduce)` sub-section.
- Each orphan entry cites its `REQ-DNN` handle and the `design-requirements.toml`
  source.
- A legacy design with no `design-requirements.toml` produces no orphan section.

**Verify:** Orphan section is nested under `Governance/spec (REV)`, not a
top-level section. The survey output is held until step 5 writes the brief (not
added to a brief that doesn't exist yet — G4).

### 11d. After `/reconcile` edit only

**Given:** Artefact state from 11c (brief with orphan section). All RV findings
terminal, but the brief is non-empty because of orphan entries.

**When:** `/reconcile` runs — reads brief, no-op gate does **not** fire (orphan
content counts — C2), per-slice edits, REV authoring (4a–4e), orphan placement
(new sub-step 4f), approve & apply.

**Then:**
- For greenfield (no existing specs): reconcile proposes a new spec, places both
  orphans via REV `create` + `introduce` rows.
- For retrofit (existing spec as input): reconcile cites existing REQs, places
  new orphans, maps duplicates to existing REQs.
- Multi-spec placement (B1): one `introduce` row per destination spec, each
  minting its own `REQ-NNN`, all traced to the same `REQ-DNN` in the narrative.
- REV narrative records `REQ-DNN → REQ-NNN` mappings under `### Orphan placements`.
- Altitude decision is documented; `/consult` fires if ambiguous.
- A stuck orphan is recorded as "stuck — `/consult`" in the narrative and is
  **non-terminal** for close (E2); a withdrawn orphan is a design-level
  retraction (edit `design-requirements.toml` + narrative record).

**Verify:** Every orphan in the brief has one of: a mapping, a multi-spec
sibling set, a withdrawal record, or a stuck note. No orphan is silently
dropped. No `REQ-NNN` is claimed to be shared across two `introduce` rows.

### 11e. After `/close` edit only — full end-to-end

**Given:** Artefact state from 11d (all orphans placed or withdrawn, REV done).

**When:** `/close` runs — pre-check, spec-coherence gate (with new orphan
advisory check), commit, transition.

**Then:**
- Close reads `plan.md` `## Requirements verification`, collects `REQ-D01`,
  `REQ-D02`.
- Close follows the read-path (E3): `review-NNN.md` `## Reconciliation Outcome`
  → pointer to `revision-NNN.md` `### Orphan placements` → reads the
  `REQ-DNN → REQ-NNN` mappings.
- Both have mappings → check passes (advisory — F2; no binary refuses, the
  walkthrough is the backstop).
- A stuck orphan → close refuses and returns to `/reconcile`.

**Verify (negative case):** Remove one `REQ-D → REQ-NNN` mapping from the
reconciliation outcome (leave the orphan with no mapping and no withdrawal).
Close must refuse. Restore it — close must accept. A withdrawn orphan (design-
level retraction recorded in the narrative) passes.
