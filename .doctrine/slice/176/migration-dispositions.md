# SL-176 PHASE-04 — Migration Dispositions

> **STATUS: APPROVED (VH-1, 2026-06-29).** This artifact is the review
> subject (design §B.2; plan PHASE-04 VH-1). The orchestrator drafted the
> classification; the User adjudicated the prov-vs-fulfil + degree judgement.
> Sign-off + OQ resolutions recorded below; the migration applies this exact
> classification.

Authored 2026-06-29. Live re-census taken at draft time (design EN-2 / SL-149 AR-1:
the IMP-207 list is *reference, not input*).

## Method

- **Census via structured queries** (`doctrine relation census` / `relation list
  --label slices --json`), not raw grep — grep over `.doctrine` catches prose
  mentions (e.g. `scoped_from` appears in 100+ design/notes files; the real edge
  count is 19). Authored-toml ground truth confirmed: **82 `slices`**, **19
  `references(scoped_from)`**, **7 `drift`** rows. No dangling targets.
- **Per-edge classification by intent** (SR-3 criterion: creation-order + intent —
  did the item scope the slice, or did the slice spawn the item?). Each of the 82
  `slices` edges was classified by reading BOTH the backlog item and the slice
  body. Day-granular `created` dates alone are insufficient (47/82 share a date),
  so intent from the bodies is load-bearing.
- **The IMP-207 doctor flag is a PRIOR, not ground truth.** It flagged 19 items
  where *all linked slices are terminal* — a narrow proxy that both false-positives
  and misses. Independent body-intent classification is recorded below; every
  disagreement with the doctor flag is called out for the reviewer.

## Headline for the reviewer — the split moved

| | design baseline (assumed) | this live classification |
|---|---|---|
| provenance (→ `originates_from`) | ~19 (IMP-207 set) | **41** |
| fulfilment (→ `fulfils`) | ~63 | **41** |

The design's table keyed class-2 off the 19-item IMP-207 doctor set; live
body-intent classification finds **41 provenance** edges. The extra 22 are
"missed provenance" the doctor's all-slices-terminal heuristic could not catch
(items *surfaced during* a slice's review/audit/design and deferred), minus 2
the doctor over-called (IMP-112, IMP-138 — slices built to *do* the item). This
is **within the design contract** (counts are reference; re-census live), but it
materially changes the migration shape: **41 two-file flips** (not 63) and **41
in-place relabels** (not 19). **This re-proportioning is the primary thing VH-1
should confirm.**

Confidence: 79/82 high, 3 med (IMP-052, IMP-162, IDE-021 — flagged below).

---

## Class 1 — `references(scoped_from)` → `references(originates_from)` (19, deterministic)

In-place wire rename of the `role=` string on a `references` row; same file
(slice toml), same direction (slice→item), same target. **No triage** — purely
mechanical. Sources (all slices):

SL-117→IMP-106, SL-118→IDE-013, SL-119→IMP-116, SL-120→IMP-117, SL-122→IDE-007,
SL-123→{ISS-034, IMP-052, IMP-072}, SL-125→IMP-046, SL-128→IMP-124,
SL-129→IMP-067, SL-131→{IDE-012, IMP-111}, SL-145→CHR-024, SL-152→IMP-072,
SL-165→IMP-188, SL-166→ISS-056, SL-169→IMP-133, SL-172→ISS-057.

(19 rows; some slices carry several.)

---

## Class 2 — `slices` provenance → `references(originates_from)` (41)

**Relabel in place** in the backlog item's toml; same direction (born-end =
backlog item), same target slice. The item was *born from* the slice (discovered,
deferred, split-out, or surfaced as a side-effect of the slice's work).

| item | slice | conf | rationale |
|---|---|---|---|
| CHR-010 | SL-094 | high | Item title "SL-094 residual"; discovered during zoom implementation |
| CHR-022 | SL-140 | high | Surfaced as finding R2 during SL-140 cordage traversal unification |
| IDE-004 | SL-056 | high | Surfaced during SL-056 Charge XIII consult as optional channels-backend enhancement |
| IDE-009 | SL-059 | high | Code-review findings (C4,C6,C3) deferred from SL-059 knowledge implementation |
| IDE-021 | SL-148 | med | Deferred coordination half of PRD-005; SL-148 implements reservation half only |
| IMP-005 | SL-024 | high | Surfaced by SL-024 code review; deferred as design OQ-1 |
| IMP-008 | SL-043 | high | Stale-prose gap discovered at SL-043 close; triggered deferred item |
| IMP-019 | SL-036 | high | Found during SL-036 post-close code review; determinism-proof gap discovered in audit |
| IMP-020 | SL-036 | high | Found during SL-036 post-close code review; code-smell refactor deferred |
| IMP-025 | SL-040 | high | Surfaced during SL-040 design as deferred follow-up (warm-cache contentset) |
| IMP-036 | SL-046 | high | Surfaced by SL-046 PHASE-04 real-corpus smoke testing; fragility discovered |
| IMP-037 | SL-046 | high | Surfaced by SL-046 code-review; 4-place re-parse duplication detected |
| IMP-039 | SL-053 | high | IMP-039 body: "Follow-up deferred by SL-053"; item born from slice |
| IMP-040 | SL-053 | high | IMP-040 body: "Follow-up deferred by SL-053"; item born from slice |
| IMP-045 | SL-056 | high | Surfaced from SL-056 dispatch worker sandboxing; macOS platform need discovered |
| IMP-051 | SL-096 | high | FR-006 surfaced during SPEC-019 design; SL-096 explicitly defers (gated on IMP-006) |
| IMP-052 | SL-056 | med | Originated from SL-056 PHASE-05 design; partially delivered by SL-123, remainder open |
| IMP-053 | SL-096 | high | Surfaced during SL-059 design; collapsed into shapes label, not separately implemented |
| IMP-065 | SL-064 | high | Explicitly deferred from SL-064 as OQ-D in design |
| IMP-068 | SL-040 | high | Surfaced by RV-026 post-ship code-review of SL-040; cleanups discovered post-delivery |
| IMP-082 | SL-080 | high | Discovered during SL-080 design phase |
| IMP-094 | SL-094 | high | Item title "SL-094 residual"; SL-094 Follow-Ups lists reset-to-fit affordance |
| IMP-095 | SL-097 | high | Title "(SL-097 follow-up)"; deferred from slice work |
| IMP-099 | SL-105 | high | Discovered at SL-105 reconcile (RV-084); cross-kind extension deferred as follow-up |
| IMP-102 | SL-121 | high | SL-121 Non-Goals lists IMP-102 as deferred sequenced follow-up |
| IMP-103 | SL-121 | high | SL-121 Non-Goals lists IMP-103 as gated deferred follow-up |
| IMP-105 | SL-026 | high | IMP-105 body: "Split out of SL-026 during 2026-06-19 design re-validation" |
| IMP-122 | SL-121 | high | Item: "Surfaced by the SL-121 audit (RV-107 F-1, F-2)" |
| IMP-143 | SL-110 | high | Follow-up merge task from SL-110 unmerged Rev 2 candidate-branch work |
| IMP-162 | SL-147 | med | "SL-147 F-7 follow-up"; deferred from slice discover-path phase |
| IMP-163 | SL-143 | high | Self-correction gate follow-up discovered during SL-143 memory-corpus work |
| ISS-001 | SL-020 | high | SL-020 audit finding F5 during backlog.rs review; test-only refactor need |
| ISS-003 | SL-036 | high | Found during SL-036 post-close code review; doc/behaviour mismatch discovered in audit |
| ISS-019 | SL-085 | high | Discovered during SL-085 dispatch setup; git-checkout issue found in slice work |
| RSK-001 | SL-036 | high | Surfaced via SL-036 audit; testing gap deferred to first consumer |
| RSK-002 | SL-036 | high | SL-036 design known-open, re-confirmed post-close perf review; deferred |
| RSK-003 | SL-036 | high | Discovered SL-036 post-close perf review; recursion overflow deferred to consumer |
| RSK-004 | SL-036 | high | Discovered SL-036 post-close perf review; query-time complexity deferred to consumer |
| RSK-005 | SL-039 | high | Surfaced by SL-039 PHASE-02 codex adversarial review; deferred at close |
| RSK-007 | SL-046 | high | Surfaced by SL-046 code-review; lexical-vs-numeric sort bug noted |
| RSK-009 | SL-094 | high | Item title "SL-094 residual" (wont-do); discovered during zoom-factor work |

---

## Class 3 — `slices` fulfilment → `fulfils` (41; two-file flip + degree)

**Two-file flip**: delete the `slices` row from the backlog item's toml, add a
`fulfils` row (with degree) to the *slice's* toml. The slice was *created to do*
the item's work. Degree examined per row (SR-4); `full` is the post-examination
default, **3 `partial`** carry rationale.

| item | slice | degree | conf | rationale |
|---|---|---|---|---|
| CHR-021 | SL-143 | full | high | SL-143 audits/improves shipped memory corpus exactly as CHR-021 requests |
| CHR-023 | SL-144 | full | high | SL-144 addresses ADR-005 compliance (reference-doc IA, hooks) per chore |
| IMP-001 | SL-040 | full | high | SL-040 scope "realising IMP-001"; slice exists to do this item |
| IMP-006 | SL-062 | full | high | SL-062 header "Scopes IMP-006"; implements unified lifecycle FSM extraction |
| IMP-020 | SL-140 | full | high | SL-140 implements traversal triplication refactor per item |
| IMP-033 | SL-060 | full | high | SL-060 header "Realises IMP-033"; extends needs/after to slices |
| IMP-035 | SL-048 | full | high | Item "Realised by SL-048"; slice implements slice<->ADR relation edge |
| IMP-036 | SL-092 | full | high | SL-092 lists IMP-036 as one of two deferred SL-046 findings; fixed here |
| IMP-037 | SL-077 | full | high | SL-077 scope lists IMP-037 item #1 (extract read_spec reader) |
| IMP-038 | SL-079 | full | high | SL-079 scope lists IMP-038 as item #1; slice created to do this work |
| IMP-039 | SL-079 | full | high | SL-079 scope lists IMP-039 as item #2; slice created to do this work |
| IMP-040 | SL-079 | full | high | SL-079 scope lists IMP-040 as item #3; slice created to do this work |
| IMP-047 | SL-158 | full | high | SL-158 title/scope match item ask; built to implement trinary actionability |
| IMP-050 | SL-096 | full | high | SL-096 delivers SPEC-019 FR-005 knowledge-record relation seam |
| IMP-058 | SL-077 | full | high | SL-077 scope lists IMP-058 item #2 (render requirement prose) |
| IMP-064 | SL-095 | full | high | SL-095 scope item #2 (migrate governance supersedes) |
| IMP-075 | SL-121 | full | high | SL-121 folds in IMP-075 as integrated scope (shared journal cycle extraction) |
| IMP-078 | SL-121 | full | high | SL-121 Scope §3 addresses "legible outcome" worktree disposition reporting |
| IMP-082 | SL-095 | full | high | SL-095 scope item #1 (add related label for slice) |
| IMP-090 | SL-093 | full | high | SL-093 addresses all 10 RV-065 findings from IMP-090 |
| IMP-102 | SL-126 | full | high | SL-126 implements structural close-gate; title/scope align with item |
| IMP-112 | SL-132 | full | high | SL-132 scope implements IMP-112 (estimate/value display, ungate helpers) |
| IMP-118 | SL-133 | full | high | SL-133 implements IMP-118 multi-dimensional priority scoring |
| IMP-132 | SL-134 | full | high | SL-134 implements IMP-132 risk set/clear verbs |
| IMP-134 | SL-136 | full | high | SL-136 implements full cross-kind tagging scope from item |
| IMP-136 | SL-137 | full | high | SL-137 implements corpus-level relation query verb (list, census) per item |
| IMP-138 | SL-138 | full | high | SL-138 adds --transitive relation-walk flag per item |
| IMP-145 | SL-139 | partial | high | SL-139 delivers paths/show; broader info command deferred wont-do |
| IMP-161 | SL-146 | full | high | SL-146 implements config get/set for priority coefficients per item |
| ISS-001 | SL-027 | full | high | SL-027 scope "DRY backlog test-fixture TOML builders"; slice created to fulfil issue |
| ISS-007 | SL-052 | full | high | SL-052 title addresses issue; vocab hit + gate closure confirmed |
| ISS-011 | SL-124 | partial | high | SL-124 cites ISS-011 Defects A+B (stale matcher, deleted path); Defect C to SL-125 |
| ISS-011 | SL-125 | partial | high | SL-125 cites ISS-011 Defect C only (provision-source); A+B remain in SL-124 |
| ISS-021 | SL-094 | full | high | SL-094 Context traces to ISS-021; zoom/pan/crop scope matches issue |
| ISS-022 | SL-121 | full | high | SL-121 Scope §1 addresses phantom reverse-diff via clean index requirement |
| ISS-030 | SL-121 | full | high | SL-121 Scope §2 addresses tree-true verify + stale worktree desync |
| ISS-034 | SL-123 | full | high | SL-123 cites ISS-034 Defect A; resolves wrong-base/fallback detection chain |
| ISS-036 | SL-127 | full | high | SL-127 objective is "Wrong base at start (ISS-036)"; issue closed by slice |
| ISS-041 | SL-135 | full | high | SL-135 wires concept-map contextualizes read path to fix visibility bug |
| RSK-007 | SL-092 | full | high | SL-092 lists RSK-007 as one of two deferred SL-046 findings; fixed here |
| RSK-010 | SL-127 | full | high | SL-127 objective "Mid-drive base refresh + drift visibility (RSK-010)"; risk closed |

### The 3 `partial` degrees (SR-4 — affirmatively examined)

- **IMP-145 / SL-139** — SL-139 delivered `paths`/`show`; the broader `info`
  command in IMP-145's ask was deferred (design wont-do). Partial completion.
- **ISS-011 / SL-124** — SL-124 fixed Defects A+B (stale matcher, deleted path);
  Defect C deferred to SL-125. Partial.
- **ISS-011 / SL-125** — SL-125 fixed Defect C only (provision-source); A+B were
  SL-124's. Partial. (ISS-011 is split across two slices, each partial — together
  full; the item is fulfilled by BOTH.)

---

## Classes 4–6 — `drift` rows (7)

| # | source | target (verbatim) | disposition |
|---|---|---|---|
| 4 | CHR-023 | `SL-143: carved out from shipped-memory corpus overhaul` | → `references(originates_from)` (provenance: chore carved out from SL-143's work) |
| 5 | CHR-021 | `IMP-148: …response-field section feeds into this audit` | → `needs` (CHR-021 needs IMP-148; "feeds into" = dependency). **Reviewer: `needs` vs `after`?** |
| 6 | CHR-021 | `mem.pattern.distribution.shipped-memory-authoring: …` | untouched (free-text memory ref) |
| 6 | IMP-148 | `mem.concept.doctrine.memory-model: …` | untouched (free-text memory ref) |
| 6 | IMP-150 | `install/review-ledger.md: …` | untouched (free-text file ref) |
| 6 | ISS-041 | `RFC-003` | untouched (bare entity drift — see OQ-2) |
| 6 | ISS-048 | `IMP-148` | untouched (bare entity drift — see OQ-2) |

`drift` label is **retained** (only `RelationLabel::Slices` is dropped this slice);
the 5 class-6 rows stay `drift`.

---

## Reviewer attention (VH-1)

### A. Disagreements with the IMP-207 doctor flag

**Doctor said provenance, classified FULFIL (2):**
- **IMP-112 / SL-132** — SL-132's scope *is* implementing IMP-112 (estimate/value
  display). Slice built to do the item → fulfil. (Doctor false-positived on
  all-slices-terminal.)
- **IMP-138 / SL-138** — SL-138 adds the `--transitive` flag the item requested.
  Slice built to do the item → fulfil.

**Doctor NOT flagged, classified PROVENANCE (24 "missed provenance"):**
IMP-020, RSK-001, RSK-002, RSK-003, RSK-004 (all SL-036 post-close review);
IMP-102, IMP-122 (SL-121 non-goals / audit); CHR-010, IMP-094, RSK-009 (SL-094
residuals); IMP-036, IMP-037, RSK-007 (SL-046 review); IMP-051 (SL-096 deferred);
IDE-004, IMP-052 (SL-056 design); ISS-001 (SL-020 audit); IMP-039, IMP-040
(deferred by SL-053); CHR-022 (SL-140 finding R2); IMP-005 (SL-024 review);
IMP-008 (SL-043 close); IMP-082 (SL-080 design); RSK-005 (SL-039 review).
Each cites the slice work it was born from (rationale in the Class-2 table).

### B. Med-confidence rows (please sanity-check)
- **IMP-052 / SL-056** — provenance, but *partially delivered by SL-123*; origin
  is SL-056 PHASE-05 design. (Note: also carries a class-1 `scoped_from` row from
  SL-123 — coexistence.)
- **IMP-162 / SL-147** — "SL-147 F-7 follow-up"; deferred from the slice's
  discover-path phase. Title-based provenance.
- **IDE-021 / SL-148** — deferred coordination half of PRD-005; SL-148 implements
  the reservation half only. Provenance by carve-out.

### C. Coexistence (SR-2) — same item, both directions, different slices
Confirmed internally consistent (validates the classification): an item born from
slice X (provenance) and fulfilled by slice Y (fulfilment) holds BOTH edges.
Examples: **IMP-036** (born SL-046 / fulfilled SL-092), **IMP-037** (born SL-046 /
fulfilled SL-077), **IMP-039 & IMP-040** (born SL-053 / fulfilled SL-079),
**IMP-082** (born SL-080 / fulfilled SL-095), **IMP-102** (born SL-121 / fulfilled
SL-126), **ISS-001** (born SL-020 / fulfilled SL-027).

### Open questions — RESOLVED at VH-1 (2026-06-29)
- **OQ-1. CONFIRMED.** The **41/41** re-proportioning (vs design's 19/63) is
  approved as the migration's class-2/class-3 partition.
- **OQ-2. LEAVE.** The two bare-entity `drift` rows (ISS-041→RFC-003,
  ISS-048→IMP-148) stay `drift` (class 6, untouched).
- **OQ-3. `needs`.** Class-5 drift CHR-021→IMP-148 → `needs` (CHR-021 needs IMP-148).
- **OQ-4. RETCON.** Dogfood `IMP-210 references(concerns) SL-176` → relabel to
  `references(originates_from)` (provenance). A deliberate, dispositioned exception
  to B.5's "concerns untouched" for this one provenance-shaped edge.

---

## Sign-off

- [x] **VH-1** — prov-vs-fulfil + degree classification APPROVED (OQ-1–OQ-4 resolved)
- Reviewer: David (User)
- Date: 2026-06-29
- Notes: 41/41 confirmed; OQ-2 leave; OQ-3 needs; OQ-4 retcon IMP-210→originates_from.
