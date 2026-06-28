# SL-165: Post-Implementation Review — Systemic Improvements

**Slice:** SL-165 — Close-projection path for audit fix-now repairs
**Outcome:** `done` — gate conforms to REQ-317; REQ-316 narrowed via REV-014
**Review ledgers:** RV-175 (design inquisition, 2 blockers), RV-177 (reconciliation audit, 2 findings)
**Scope:** 3 files, 5 invariants, 1 REV, ~12 agent round-trips over audit→reconcile→close

---

## 1. What Went Well

- **Design inquisition caught the two hardest holes before code was written.**
  RV-175 (codex-inquisitor) found the name-trust-vs-content-trust gap (INV-6/moved-ref)
  and the over-broad scope exception (INV-2/scratch ride-through). Both were semantic
  holes in a *provenance* gate — shipping either would have defeated the slice's purpose.
  The adversarial design review (ADR-007) earned its keep.

- **The landed gate is genuinely faithful.** Every §5.5 invariant (1–6) traced to source;
  conformance algebra clean (3/3); the lifecycle anchor test exercises the full
  repair→close→integrate→`status done` path without manual fold dances. The gate's
  predicates survive adversarial re-inspection.

- **The REV external review caught prose-gate drift.** Codex passed REQ-316 prose on
  pass 1 (REQUEST-CHANGES: 4 under-described teeth — INV-3 Created-only missing,
  INV-5 count-exact not stated, INV-1/F3 full-journaled-gate implicit, kind=audit
  constraint omitted) then APPROVE on pass 2 after correction against source. Without
  this, the normative spec would have described a weaker gate than the one actually
  enforced.

---

## 2. Systemic Friction Points

### 2.1 Close complexity cascade (HIGH)

The close sequence required:

```
promote edge→main → build gate-bearing binary from candidate worktree (not on edge) →
dogfood close_target create with new gate → admit → integrate --trunk → ISS-030 verify →
coverage record → slice status done ✗ (undischarged drift) →
diagnose rec_discharges 3-clause predicate → author 2 accept RECs (status_delta +
evidence_ref ⊇ ALL residual coverage keys) → retry done → lifecycle flip →
harvest memory → close backlog
```

**Cost:** ~9 round-trips just for close integration. Three pain points compound:

1. **The gate binary wasn't on edge.** The implementation lived only in a gc'd dispatch
   worktree. The closer had to rebuild from the candidate tip. For a slice that *modifies
   the close machinery itself*, the close machinery can't dogfood itself without a
   candidate build.

2. **The closure drift-discharge recipe is opaque.** `slice status done` refused with
   `undischarged residual drift on requirement(s): …` — a one-line error with zero
   guidance. The REC authoring pattern (3-clause predicate in `src/slice.rs:1268`:
   move=accept + status_delta matching *current authored* status + evidence_ref ⊇ ALL
   residual coverage keys including OTHER slices' cells) had to be reverse-engineered
   from source code, then hand-authored into REC TOML files.

3. **The accept REC shape is undocumented.** The close skill mentions the `done` flip
   but gives no recipe for the discharge RECs that gate it. The memory corpus has the
   recipe (`mem_019f075f`), added *after* this close, but it's not surfaced by the
   close skill or the `drift` error message.

### 2.2 Spec internal contradiction was latent (MEDIUM)

REQ-316 ("no non-journaled source") and REQ-317 ("source must be candidate ref") were
both `active` with `unsettled=false` while mutually contradictory. No spec-internal
consistency check caught this. SL-165 was the first slice that needed both REQs
simultaneously — it surfaced a contradiction that had been latent since SPEC-022 was
authored.

### 2.3 Selector discipline drifts (PATTERN)

SL-165 had clean conformance (3/3) — but only because the scope was surgically tight
(one src file, two test files). The pattern visible in SL-169 (4 undeclared) is the
norm for slices with broader scope. The conformance algebra catches under-declaration
only at audit time — late, when the cost of back-filling selectors is a reconcile
detour rather than a design-time check.

### 2.4 External review as a bottleneck (STRUCTURAL)

The ADR-013/D4 requirement for external (codex) review of normative spec edits added
2 round-trips: initial REQUEST-CHANGES + revised APPROVE. The first pass found the
prose described a weaker gate than the code enforced — exactly the right scrutiny. But
the manual prose-editing loop (author REV narrative → codex review → revise wording →
codex re-review → approve → apply → manual prose landing) is labor-intensive for what
should be a mechanical fidelity check. The prose-gate gap (4 under-described teeth)
would have been prevented by a tool that diffs the spec's claimed invariants against
the gate's actual predicates.

---

## 3. Systemic Improvements

### S1: Close skill — add closure drift-discharge recipe

**What:** The `/close` skill (§4, lifecycle flip) should document the `rec_discharges`
predicate and the accept REC recipe. When `slice status done` refuses, the skill should
surface the diagnostic path rather than leaving it to a source-dive.

**Where:** `/close` SKILL.md, after the lifecycle transition step, or as a dedicated
troubleshooting sub-section.

**Specifics:**
```
If `slice status done <id>` refuses with "undischarged residual drift":
1. For each flagged requirement, author an accept REC:
   `doctrine rec new --move accept --owning-slice SL-NNN --title "accept REQ-NNN"`
2. Hand-author into rec-NNN.toml:
   - [[status_delta]]: requirement = "REQ-NNN", from = "<current>", to = "<current>"
     (same-value delta if status didn't change — `to` must equal the authored status)
   - [[evidence_ref]]: one entry per distinct coverage key (include OTHER slices' cells,
     not just your own) — find keys: `grep -rl REQ-NNN .doctrine/slice/*/coverage.toml`
   See mem_019f075f for the full recipe.
```

### S2: `slice status done` — richer error message

**What:** When `done` refuses on drift, the error should name the flagged requirements,
explain that an accept REC is needed, and point to the pattern. The current one-line
`undischarged residual drift` is a diagnostic dead end.

**Where:** `src/slice.rs` — `rec_discharges` or the caller that constructs the error.

**Specifics:** The error should emit something like:
```
slice status SL-NNN done refused: undischarged residual drift on requirement(s): REQ-316, REQ-317
→ Each needs an accept REC (move=accept) with a status_delta affirming its
  current authored status and evidence_ref covering all residual coverage keys.
  Pattern: doctrine rec new --move accept --owning-slice SL-NNN --title "accept REQ-NNN"
  See: .doctrine/memory/items/mem_019f075f/
```

### S3: Spec consistency gate — detect REQ contradictions

**What:** Build (or mandate a manual check at) a consistency lint that detects
contradictory requirement pairs within a spec. The REQ-316 ⊥ REQ-317 pattern
("A forbids X" ∧ "B mandates X") is mechanically detectable: parse the mandatory/
prohibitive modality, resolve shared nouns, flag conflicts.

**Where:** `doctrine spec validate` or a new `doctrine check spec-consistency` gate.
Alternatively: a lightweight manual gate in the `/spec-product` skill — "before
locking a spec, enumerate its requirements and check for forbids/mandates
contradictions on shared subjects."

**Specifics:** At minimum, when both REQ-316 and REQ-317 carry `active` status, the
spec should carry a `⚠ contradictory` marker or refuse to be marked `settled`. The
detector doesn't need NLP — a keyword heuristic (forbids/refuses/only vs mandates/
requires/sources) on shared nouns (source/provenance/candidate) would catch this class.

### S4: Close gate — verify the gate-bearing binary is available

**What:** When a slice's code modifies `src/dispatch.rs` (or any path reachable from
the `doctrine` binary), the close skill should detect that the slice's implementation
isn't on edge and build a gate-bearing binary from the candidate/admitted OID before
attempting dogfood operations. The current pattern (build from candidate worktree) is
manual and fragile.

**Where:** `/close` skill — step 3a (dispatched slice integration) should check:
does `main` (or the delivery trunk) carry the slice's `src/` edits? If not, build
from the admitted candidate OID before running `candidate create`/`admit`/`integrate`.

**Alternative (better):** The `dispatch candidate create --worktree` path already
builds in the candidate worktree; a `dispatch candidate build` command that produces
the gate binary from a candidate ref would give the closer a one-command path.

### S5: REV prose-gate fidelity lint

**What:** When a REV's `modify` row targets a requirement whose normative prose
uses invocatory language (forbids/refuses/only/requires/mandates), lint the proposed
prose against the landed gate's actual predicates. The codex review in SL-165 found
4 under-described teeth — a mechanical diff (what invariants does the code enforce
vs what the prose claims) would have caught them without an external review round-trip.

**Where:** A new `doctrine revision check-fidelity REV-N` command, or a hook in
`revision approve` that warns when a `modify` row targets a requirement and the
proposed prose mentions fewer constraints than the code's predicate set.

**Pragmatic v1:** The `/reconcile` skill should, for REV modify rows targeting
requirements, mandate a read of the exact gate predicates in source and a
side-by-side comparison with the proposed prose — rather than assuming the designer's
prose is correct.

### S6: Memory — the accept REC pattern should be surfaced in `/close`

**What:** `mem_019f075f` (closure drift discharge via accept REC) was harvested
*after* SL-165 close. Every subsequent close that touches a spec requirement will
need this pattern. The close skill should reference the memory or inline the recipe.

**Where:** `/close` SKILL.md §4 — add a cross-reference or short recipe.

---

## 4. Recommendations by Effort

| ID | Effort | Impact |
|----|--------|--------|
| S1 | Small (skill edit) | Eliminates ~4 agent round-trips per governed close |
| S2 | Small (error string) | Eliminates source-dive every time `done` refuses |
| S6 | Trivial (1 line in skill) | Surfaces the existing memory |
| S3 | Medium (new check) | Prevents latent spec contradictions from shipping |
| S4 | Medium (skill + maybe new command) | Eliminates manual candidate-build detour |
| S5 | Large (new tool) | Reduces REV external-review round-trips |

**Do now:** S1, S2, S6 (low effort, high recurrence).
**Do next:** S3 (the class of spec contradictions will recur).
**Defer:** S4 (rare — only bites when the close mechanism is the slice's own subject); S5 (requires new tooling).
