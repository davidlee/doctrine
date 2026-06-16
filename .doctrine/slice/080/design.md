# SL-080 Design: Reconcile skill + audit/reconcile seam disentanglement

## Decisions

### D1 — `/reconcile` owns writing; `/audit` owns identification only

ADR-003 §7 draws the hard edge; this design enacts it.

- `/audit` identifies what changed and records findings on the RV ledger. It
  builds a **reconciliation brief** — a structured map of findings→target
  artefacts — so `/reconcile` can act without re-performing audit. Audit does
  **not** write spec/governance changes.
- `/reconcile` is the **sole explicit writer** of reconciled truth. It consumes
  the RV ledger + reconciliation brief, writes the changes, and records what was
  done.
- `/close` verifies reconciliation is complete (every governance/spec item
  covered by a `done` REV or an RV-native disposition) before the terminal
  transition.

### D2 — Two write surfaces, one skill

`/reconcile` writes to two surfaces with different mechanisms:

| Surface | Mechanism | Rationale |
|---|---|---|
| **Per-slice artefacts** (design.md, slice-NNN.md) | Direct edit with user agreement | Per-slice, not evergreen governance. Changing them as understanding evolves is fair game; already happens during `/consult` and `/design` escalation. |
| **Governance/spec truth** (ADRs, specs, requirements, policies, standards) | REV kind (`doctrine revision`) | Evergreen, normative truth. REV provides typed `[[change]]` rows, approval checkpoint, apply path for `status` rows, surfaced-for-manual for prose. The REV rationale (`revision-NNN.md`) carries the reconcile narrative. |

A single reconcile pass may use both surfaces — e.g. update `design.md` directly
*and* author a REV for an ADR amendment.

**Project-local documentation (e.g. `doc/*` in the doctrine repo itself) is not
a doctrine feature category.** It is edited directly like any project file, not
treated as an evergreen doctrinal surface. The design does not reference it as a
formal category.

### D3 — Reconciliation brief bridges audit→reconcile

The reconciliation brief is the structured handoff from `/audit` to
`/reconcile`. It lives in a **dedicated `## Reconciliation Brief` section**
in the RV markdown (`review-NNN.md`) — separate from `## Synthesis` (the
audit's closure story). The brief is audit's last act before handing off.
Shape:

```markdown
## Reconciliation Brief

### Per-slice (direct edit)
- design.md §3: the eviction model changed from edge-at-a-time to per-SCC —
  update prose to match implementation.

### Governance/spec (REV)
- ADR-006 §D5: branch-point staleness description is wrong → REV modify
- REQ-077: cordage scale target verified at 50k nodes → REV status active
```

### D4 — `/reconcile` skill process

1. **Read inputs:** RV ledger (`RV-NNN`) + `## Reconciliation Brief`.
   Confirm every finding is terminal (`verified`/`withdrawn`/`tolerated`).
   Findings stay `verified` — remediation is recorded separately by reconcile,
   not by mutating the finding disposition.
2. **No-op gate:** If the reconciliation brief is empty (all findings
   withdrawn/tolerated with no writes needed), append a `## Reconciliation
   Outcome` to the RV confirming nothing to reconcile, then hand off to
   `/close`.
3. **Per-slice edits:** For each direct-edit item in the brief, present the
   edit to the user, confirm, then write to `design.md` (and/or `slice-NNN.md`
   if scope changed). Record what was edited in `## Reconciliation Outcome`.
4. **REV authoring:** For each governance/spec item in the brief:
   - Discover or create a REV. If the brief records a REV id, use it. Otherwise
     create one by convention (`reconcile-sl-NNN` slug). **Collision guard:**
     if a REV with that slug already exists, inspect its title/rationale before
     reuse; if unrelated, create a distinct slug or ask for user decision.
   - Add `[[change]]` rows: `doctrine revision change add REV-N --action <a> ...`
   - Record the reconcile narrative in `revision-NNN.md` (what changed, why,
     link to RV finding).
   - **Split rule:** if any row is known to require separate debate or will not
     land in this pass, split it into its own REV *before* approval/apply. Do
     not create a half-applied omnibus REV that blocks close.
5. **Approve & apply:** `doctrine revision approve REV-N` then
   `doctrine revision apply REV-N` — auto-lands `status` rows; surfaces
   `modify`/`create`/etc. for manual prose landing.
6. **Manual prose landing:** For surfaced-for-manual rows (ADR/Spec prose edits,
   new entity creation), perform the edits by hand under the authored-truth
   honour model. Then transition the REV to `done`:
   `doctrine revision status REV-N done`.
7. **Escalation gate:** If reconcile discovers the model itself is inadequate
   (not mere instance drift), transition the slice back to `design`:
   `doctrine slice status <id> design` (the ADR-009 back-edge).

### D5 — `/audit` retune: strip writing

Current audit SKILL.md already directs findings to the RV ledger, not in-place
edits. The retune tightens the prose:

- **Remove** any guidance that implies writing `design.md` or governance during
  audit (the "design was wrong → reconcile design.md" disposition pattern).
- **Add** the reconciliation brief as audit's final output: a dedicated
  `## Reconciliation Brief` section in the RV markdown, separate from
  `## Synthesis`, mapping findings to target artefacts.
- Findings that require a change stay `verified` (the finding is confirmed) —
  the *remediation* belongs to reconcile and is recorded separately. Do not
  mutate finding disposition to `fixed`; the RV may not support that state,
  and keeping finding truth separate from remediation truth is correct.
- Audit's last step becomes: "write the reconciliation brief, then hand off to
  `/reconcile`."

### D6 — `/close` retune: spec-coherence check

Current close SKILL.md is mechanically sound (uses `doctrine slice status`,
references RV, checks closure seam). Add:

- **Spec-coherence gate:** Before `done`, verify every governance/spec item
  in the reconciliation brief is resolved:
  - Covered by a `done` REV, **or**
  - Withdrawn in the RV with rationale, **or**
  - Tolerated in the RV with rationale, **or**
  - Escalated back to design.
  - Every per-slice direct-edit item has been applied to `design.md`/
    `slice-NNN.md`.
  - The RV ledger is resolved (`done · await=none`).
  - The reconcile outcome is recorded (REV rationale and/or RV
    `## Reconciliation Outcome`).
- No free-floating "rejected" disposition — every item lands in one of the
  RV-native or REV states above.
- If any item is unresolved, refuse close and return to `/reconcile`.

### D7 — Routing wire

Add the `/reconcile` row to `install/routing-process.md` (the boot snapshot
source). The row reads:

```
| Slice exists, audit complete, reconciliation brief ready | `/reconcile` |
```

This lands **only when the skill ships** — never point routing at a deferred
skill (ADR-009 F2/F14). Until then, reconcile-entry is manual discipline.

Verification: after `doctrine boot`, the generated `.doctrine/state/boot.md`
must include the `/reconcile` row in the routing table, not just the install
source.

### D8 — No CLI verb surface in this slice

`doctrine slice reconcile` is not built here. The skill drives existing verbs
(`doctrine revision *`, direct file edits). The `slice reconcile` CLI verb is a
separate follow-on (ADR-003 §11). The skill operates as manual discipline with
CLI-assisted steps — same posture as `/audit` today.

### D9 — Reconcile inspects targets, does not re-audit

Reconcile does not perform new issue discovery. It may inspect target artefacts
only to validate applicability, locate edit points, detect drift since audit,
and perform the authored write. This keeps the seam intact (audit owns
discovery) without making reconcile mechanically blind to post-audit changes.

## Open Questions

### OQ-1 (resolved) — REV discovery

How does reconcile discover whether a REV already exists for this slice?
Today there is no `doctrine revision list --slice SL-NNN` or slice↔REV
relation edge. **Resolution:** the reconciliation brief records the REV id(s)
if audit created a stub REV; otherwise reconcile creates one by convention
(`reconcile-sl-NNN` slug). Reconcile must inspect before reuse (collision guard,
D4 step 4). A REV→slice relation is follow-on.

### OQ-2 — One REV per slice or per finding?

A slice may need multiple governance changes. Options:
- **One REV per slice** — collate all governance findings into a single
  `reconcile SL-NNN` REV with multiple `[[change]]` rows. Simpler; one artefact
  to track. But a REV stays `started` (not `done`) until *all* rows land,
  including surfaced-for-manual prose edits — one stuck row blocks the whole REV.
- **One REV per independent change** — finer granularity, but more artefacts to
  track and approve.

**Recommendation: one REV per slice** by default, with the REV rationale
carrying the reconcile narrative. The REV's `done` gate is the close-gate
precondition. Split rule (D4 step 4): if a surfaced-for-manual row is known to
require separate debate, split it into its own REV *before* approval/apply.

## Verification

- `/reconcile` skill exists at `.agents/skills/reconcile/SKILL.md` with valid
  YAML frontmatter.
- `/audit` skill prose describes identification-only; no in-place writing
  guidance; includes reconciliation brief step.
- `/close` skill includes spec-coherence check (REV done or RV-native
  disposition for every item; no free-floating "rejected").
- All three skills form a coherent chain: audit identifies → reconcile writes →
  close confirms.
- `doctrine claude install` succeeds and embeds the new skill.
- Routing table in `install/routing-process.md` adds `/reconcile` row.
- Generated `boot.md` includes the `/reconcile` routing row (verify after
  `doctrine boot`).
- Lint: zero warnings on all SKILL.md files.

## Affected Files

- `.agents/skills/reconcile/SKILL.md` — **new**
- `.agents/skills/audit/SKILL.md` — retune
- `.agents/skills/close/SKILL.md` — retune
- `install/routing-process.md` — add `/reconcile` row
- `.doctrine/state/boot.md` — regenerated

## Non-Goals

- `doctrine slice reconcile` CLI verb (deferred, ADR-003 §11)
- REV target for project-local documentation (not a doctrine feature category)
- Coverage derivation engine (ADR-009 §3 deferred)
- Conduct enforcement (advisory only, ADR-009 §2)
