# SL-220 design — Ledgered value claims

Status: drafted 2026-07-16, post-clarification loop (three operator
adjudications) + external adversarial review (codex GPT-5.5, same-session
section-by-section, eight sections, every finding integrated); governs
implementation. Governing contracts: RFC-020 (Phases 0+1; T1–T7, RV-275
gate obligations), SL-219 design (AnchorMap seam §2, D8 lemma, DomainSystem
split — substrate, executes unchanged), SL-213/217 (constraint layer, elicit
queue), ADR-015 (amended by this slice's REV), ADR-001 (layering), REV-022
(dissolved by this slice), REV-023 (estimate ladder — untouched), STD-001,
STD-002.

## Decision ledger

| id | decision | rationale (compressed) |
|---|---|---|
| D1 | Anchor claims are `[[judgement]]` rows on the existing wire — `form = anchor`, single subject, **per-domain payload column sets** (value: `{magnitude}`; estimate Phase 2: the facet shape as further optional columns, additive within v3). Validation, not parse, enforces payload exactness per domain | One judgement interface (RFC-020 T2) made executable; Phase 2 needs zero schema motion |
| D2 | Version gate: parse accepts {2, 3}, writes 3; every v2 file is a valid v3 document | No corpus rewrite; SL-219 v2 goldens byte-identical |
| D3 | Same-tier conflict → arithmetic **mean over the full active winning-tier row multiset** as point anchor (D8-safe); interval `{min, max}` as rendered bounds; loud `ClaimConflict` finding + reprobe candidate; uniform across tiers (conflicted pin = "contested pin"). **No identity-based dedupe anywhere** — `by` is optional; every active row is evidence; identical re-fires are harmless corroboration. *(Operator-adjudicated 2026-07-16: average likely beats either guess; don't break the graph pending a conversation)* | Deterministic, surfaced, never silent; no lower tier wins because a higher tier disagrees; no invented winner among claims — the mean is an aggregate, not a pick |
| D4 | **Anti-laundering**: `anchor_map()` exposes pin/human tiers only; agent/migrated claims never enter `compile` — they bypass the constraint layer and re-enter at the graph ladder below projection | A point anchor pins projection to itself; anchoring an agent guess launders it into the tier that outranks it (the exact RFC-020 §Context failure) |
| D5 | Ladder × lens (RFC-020 OQ-2 answered): independent per-partition resolution; the unlensed partition feeds everything; lensed partitions resolve into an inert `lensed` output (IDE-035 seam) — no cross-partition mixing, ever | RFC-019 T5 pooling discipline, extended to claims |
| D6 | Transitional rung: an **unmigrated `[value]` facet** reads at the bottom of the evidence ladder (below attributed agent claims), consulted only when zero claim rows exist for the item; `UnmigratedFacet` finding fires on facet **presence**, not consumption. For compared items, projection out-ranks the facet permanently — the flip's semantics, stated without euphemism (§3). Facet read path deletes when both domains have migrated (Phase 2 exit criterion) | Shipping the binary IS the flip for unmigrated corpora (IMP-290); uncompared values contribute instead of vanishing; compared guesses are calibrated by evidence — the point |
| D7 | Capture admissibility mirrors today's `value set` surface exactly (warn-never-refuse, REV-022); consumption is gated at the scoring surface (`effective_raw_value`'s caller contract — value-bearing scored kinds only). Paired test over `ALL_KINDS` | Records' claims: capture-lossless, consumption-inert; Phase 3 widens consumption, not capture |
| D8 | Migration + Phase 0 diagnostic are throwaway Python scripts in `scripts/`, not product verbs *(operator-adjudicated)*. Idempotency key = **facet state** (source path + magnitude): changed-facet re-runs import a superseding row; census `facets_found == imported + already-imported + re-imported`, one active migrated row per source facet | One-shot machinery stays out of the product; path-only skip keys strip un-imported edits (review finding) |
| D9 | `[value]` tables are **physically stripped** by the migration pass after its census *(operator-adjudicated)*; `value pin` / `pin --retire` ride the worker-refused write class AND the D13 interactive-TTY gate; `value clear` refuses while a pin is active | Dead authored-looking data is a standing lie; pin admission is a contract, not a column (RV-275 F-5) |
| D10 | No capture no-op guard: every `value set` invocation mints a row | The guard cannot key on identity without the optional-`by` inference D3 forbids; identical re-fires change no resolved value |
| D11 | JSON `value_source` is a disclosed **breaking token-set change**: `authored` removed; `pin`/`human-claim`/`agent-claim`/`migrated-claim`/`unmigrated-facet` added; `projected`/`gauge`/`default` byte-stable | Honest: no code path emits `authored` post-flip; "additive" would be false |
| D12 | The ADR-015 REV **and SPEC-020's normative value-surface amendments** gate the flip phase — approved before it lands; earlier phases are strictly additive and REV-independent. Additive/documentary spec REQs remain reconciliation obligations | Canon must never be stale about what `value set` does while the corpus is live (review finding) |
| D13 | Pin admission gate = **interactive-TTY check + worker-mode refusal** *(operator-adjudicated 2026-07-17, RV-278 F-1)*: `value pin`/`--retire` refuse when stdin is not an interactive terminal, naming the posture. Honest caveat: a posture bar, not authentication — the append-only attributed ledger is the backstop. Gate check is a pure-input seam (`is_interactive: bool` threaded from the shell) so both branches unit-test | Worker-refusal gates workers, not agents; the agent population that wrote the unattributed corpus drives doctrine via non-TTY tool shells — the TTY bar covers the threat that matters at one `isatty` call |
| D14 | Conflict findings fire at **every** tier (surfaced-never-silent holds); **reprobe nomination and the "contested" render treatment are anchored-tiers-only (Pin/Human)** *(operator-adjudicated 2026-07-17, RV-278 F-5)*. Agent/migrated conflicts render as ordinary findings with "calibrate via comparison" guidance | The reprobe queue is human attention; agent self-contradiction is exactly what projection calibrates — routing it to humans inverts the RFC's intent |

## §1 Wire schema v3: the anchor row

**Target behaviour.** An absolute magnitude claim is one `[[judgement]]` row
in the existing session files — same store, same resolution, same
supersession/tombstone machinery. Domains differ only in payload; nothing in
this section is value-specific (RFC-020 T2 invariant).

**`Judgement` struct motion (all additive or optionalising):**

| field | v2 | v3 | rule |
|---|---|---|---|
| `form` | `order \| ratio` | + `anchor` | closed enum grows |
| `b` | `String` | `Option<String>` | required for order/ratio; **absent** for anchor |
| `response` | `Response` | `Option<Response>` | required for order/ratio; **absent** for anchor |
| `magnitude` | carried, never compiled | unchanged type | value-domain anchor payload (D1); still inert on order rows |
| `date` | `String` | `Option<String>` | asserted-at semantics; required on every row **except** `rater = migrated` (honestly absent) |
| `observed_at` | — | `Option<String>` new | present **iff** `rater = migrated` (strict biconditional; the migration date) |
| `basis` | — | `Option<String>` new | free-text evidence citation (e.g. `REQ-059`, or the source facet's provenance for migrated rows) |
| `admission` | — | `Option<AdmissionKind>` new | closed enum, sole variant `pin`; mintable only by the gated `value pin` path (§4) |
| `rater` | `human \| agent` | + `migrated` | closed enum grows |

**Anchor payload is a per-domain column set (D1, T2 made executable).**
Payload lives in dedicated optional columns; validation — not parse —
requires exactly the payload set for the row's domain. Value (this slice):
`{magnitude}` — finite f64, negatives included, mirroring `value::validate`
exactly (single source; no range policy smuggled in). Estimate (Phase 2):
the existing facet shape (`{lower, upper}` + rater-stated confidence
columns) as further optional columns — **additive within v3**, no version
bump: parse is lossless over absent optionals and the validation matrix is
per-domain capture policy, not wire structure. New domains add columns,
never reshape rows.

**Validation matrix** (`validate_judgement`, capture-time — parse stays
lossless):

- `form = anchor` ⇒ `b` absent ∧ `response` absent ∧ the domain's payload
  set present exactly (value: `magnitude` present, estimate-payload columns
  absent).
- `form = order|ratio` ⇒ `b` present ∧ `response` present (v2 rows satisfy
  this by construction).
- `form = anchor ⇔ frame = anchor_frame_for(domain)` — each domain's frame
  set names exactly one anchor frame (`value-anchor`; `cost-anchor` arrives
  with Phase 2). A pairwise frame on an anchor row, or an anchor frame on an
  order/ratio row, is rejected at capture.
- `rater = migrated` ⇒ `form = anchor` ∧ `date` absent ∧ `observed_at`
  present; `rater ≠ migrated` ⇒ `date` present ∧ `observed_at` absent
  (strict — every row carries exactly one of the two).
- `admission = pin` ⇒ `form = anchor` ∧ `rater = human` — contradictory
  provenance states rejected at capture (RV-275 F-5).
- Anchor subject admissibility: §3/D7 — mirrors the `value set` surface
  (warn-posture), NOT `admissible_value_pair` (which remains the pairwise
  gate, untouched).

**Frame.** `DOMAIN_FRAMES` grows `value → {equal-effort, value-anchor}`
(`FRAME_VALUE_ANCHOR`). Never user-typed — `value set|pin` stamps it.
`domain_for_frame` stays a total function.

**Version gate (D2).** `COMPARISON_VERSION` bumps to 3; parse accepts
{2, 3} and writes 3. The strict-equality gate becomes a two-member set with
the same remedy-naming error for anything else. (Rejected: script-rewrite of
all files to `version = 3` — churn on every session file for zero semantic
gain.)

**Ordering totality.** `Judgement::ordering_date()` — `date` if present,
else `observed_at` — replaces `date` in the tier-1 ordering key
(`(ordering_date, session_uid, seq)`). Total by the validation matrix;
deterministic output order for mixed migrated/live rows follows.

**Identity & resolution.** `IdentityKey.pair_hi` becomes `Option<String>`
(Ord — BTreeMap key stays valid): anchor rows key as
`(pair_lo = a, pair_hi = None)`; `form_key` gains `"anchor"`, `rater_key`
gains `"migrated"`. Same-session same-subject anchor rows by one rater at
one lens group under one identity key — R3 implicit revision applies
verbatim; `form` in the key means anchors never collide with order/ratio
rows on the same subject. Lifecycle inertness: a row is inert iff **any
present subject** is terminal/superseded — for anchors, the single subject
`a`; `entity_superseded()` checks `b` only when present. Explicit
`supersedes`, tombstones, and cross-session concurrency (concurrent
contradiction is a finding, never latest-wins — RFC-019 T6) apply unchanged.

**Sample rows** (live human claim; migrated import; a pin is the first
shape plus `admission = "pin"`):

```toml
[[judgement]]
uid = "…"
seq = 0
a = "SL-204"
form = "anchor"
domain = "value"
frame = "value-anchor"
magnitude = 6.5
rater = "human"
by = "david"
basis = "REQ-059"
date = "2026-07-16"

[[judgement]]
uid = "…"
seq = 0
a = "IMP-118"
form = "anchor"
domain = "value"
frame = "value-anchor"
magnitude = 3.0
rater = "migrated"
basis = "facet [value] .doctrine/backlog/imp-118.toml @ 4a12e576 david 2026-06-30"
observed_at = "2026-07-16"
```

**Code impact (§1):** `src/comparison/wire.rs` — enum variants, field
motion, validation matrix, version-gate set, frame-table row, goldens.
`src/comparison/resolve.rs` — **semantic** changes: `ordering_date()`
accessor in the ordering key, degenerate-pair `IdentityKey`, new
`form_key`/`rater_key` tokens, single-subject lifecycle rule, tests.

## §2 Claim resolution: the ladder as a pure pass

**Target behaviour.** A new pure pass between tier-1 resolution and tier-2
compilation: active anchor rows → per-item tier resolution → (a) the
`AnchorMap` fed to `compile`, (b) below-projection priors for the graph
ladder (§3), (c) findings. This IS the "claims + comparisons + config →
AnchorMap + bounds + projection" builder RFC-020 T2 names; SL-219 §2's
non-foreclosure clause ("a future claim ledger replaces the builders, never
the seam") is honoured **at the seam**: `compile` and `project` keep their
rule structure and behaviour unchanged. Honesty about source (RV-278 F-3):
§1's `b`/`response` optionalisation mechanically touches `compile`'s field
accessors; the design closes that with a **pairwise projection type** — the
filter seam constructs `PairRow { a, b, response, … }` views for
order/ratio rows (field presence guaranteed by type), so `compile`'s rule
code stays total with no `Option` handling inside the constraint layer.
Proof obligation: the existing compile suite green with **zero golden
churn** — behaviour preservation is behavioural, not byte-of-source; the
SL-219 D-NF "reused as-is" deviation is recorded here.

**Input filter, at every consumer (RV-278 F-6):** `compile` receives only
active value-domain `order|ratio` rows (as `PairRow` views) — **anchor rows
terminate at `claims` and never reach any compile consumer**: the store's
own pipeline AND the SL-217 elicit `assemble` path, which recompiles its
baseline from `Pipeline.active_judgements`. That field splits into
pairwise/anchor views; every recompiler consumes the pairwise view.
`src/priority/elicit.rs` joins the code-impact list. Consequence: anchor
rows never acquire a `CompilationStatus`; their display token (`anchored` /
`prior` / `conflicted`, plus the existing resolution tokens) is produced at
the store's `RowSummary` join — the seam that holds `ClaimResolution` —
NOT in `resolve.rs::display_token`, which changes only for
`Option<CompilationStatus>` handling (RV-278 F-8).

**Module.** `src/comparison/claims.rs`, pure leaf beside
`resolve`/`compile`/`project` (ADR-001): no clock, disk, config reads.
Input: the claims pass performs its **own input selection** over the
post-resolve row set (RV-278 F-2): value-domain anchor rows with resolution
status `Active` **or `InertLens`** — R5's lens inertness is a *constraint
compilation* gate, not a claim-capture gate; without this, `resolve.rs:
176`'s unconditional InertLens marking would empty the `lensed` output
forever and make the lens-isolation gate test pass vacuously.
Superseded/tombstoned rows stay excluded (supersession reduces lensed
threads too). Order rows' R5 semantics are untouched. Output:

```rust
// Ascending declaration order so derived Ord + Iterator::max are correct
// by construction; pinned by `pin_outranks_all_tiers_under_derived_ord`.
pub(crate) enum ClaimTier { Migrated, Agent, Human, Pin }

pub(crate) struct ResolvedClaim {
  pub value: f64,                       // singleton magnitude, or conflict mean
  pub tier: ClaimTier,
  pub conflict: Option<ClaimConflict>,  // present ⇔ >1 distinct magnitude in the winning tier
  pub rows: u32,                        // active winning-tier row count (render)
}

pub(crate) struct ClaimConflict { pub low: f64, pub high: f64, pub distinct: u32 }

pub(crate) struct ClaimResolution {
  pub anchored: BTreeMap<String, ResolvedClaim>,  // Pin/Human — AnchorMap + graph rung 1
  pub priors:   BTreeMap<String, ResolvedClaim>,  // Agent/Migrated — graph ladder below projection
  pub lensed:   BTreeMap<(String, String), ResolvedClaim>, // (lens, item) — inert (IDE-035 seam)
  pub findings: Vec<ClaimFinding>,
}
```

**Algorithm (per item, deterministic).**

1. Partition by lens; the unlensed partition drives everything below.
   Lens-tagged rows resolve identically per (lens, item) into `lensed` —
   captured, rendered on demand, consumed by no scoring surface (D5).
2. Tier each row: `admission = pin` → Pin; else by `rater` (pin ⇒ human is
   a §1 capture invariant — tiering is total).
3. Winning tier = highest non-empty. Lower tiers contribute nothing for
   that item — not even bounds.
4. Within the winning tier: one distinct magnitude → that value (`rows` may
   exceed 1 — corroboration, no conflict). Multiple distinct magnitudes →
   `value =` arithmetic mean over the **full** active winning-tier row
   multiset (D3 — no dedupe; three humans at (5,5,7) → 5.67),
   `conflict = {min, max, distinct}`, `ClaimFinding::Conflict` fires
   (named "contested pin" when tier = Pin). Deterministic under row
   permutation (mean is order-free; BTreeMaps throughout).
5. Route: Pin/Human (incl. conflict means — the tier's resolved output) →
   `anchored`; Agent/Migrated → `priors`.

**Anti-laundering (D4).** `ClaimResolution::anchor_map() -> AnchorMap`
exposes exactly the `anchored` magnitudes. `priors` bypass the constraint
layer entirely and re-enter at the graph ladder below projection.

**Row-gating honesty (scope R1).** `compile` attaches anchors only to items
with comparison rows in that system
(mem.fact.comparison.anchor-attachment-row-gated-per-system). Correct for
the constraint layer (an anchor with no rows constrains nothing), harmless
for the item: the graph ladder reads `anchored` directly — an item whose
only evidence is a human claim resolves to it whether or not any comparison
row exists. The AnchorMap is a *projection input*; `anchored` is the
*authority record* — same numbers, two seams.

**Supersession/demotion (RV-275 F-5).** No mutation path exists in this
pass: demoting a pin = appending a superseding row (reduced by resolution
before claims sees it); the pass is a pure fold over already-resolved rows.

**Pipeline wiring (`store.rs`).** The value `DomainSystem` gains the pass:
rows → resolve (shared) → **claims** → compile(value order/ratio rows,
`claims.anchor_map()`) → project. `Pipeline` carries
`value_claims: ClaimResolution`. The estimate `DomainSystem` is untouched
(anchors stay `authored_est_cost` until Phase 2). The facet→AnchorMap
builder (`graph.rs::comparison_anchor_map`) is **deleted** — facets stop
anchoring the compile entirely; where unmigrated facets still enter is
§3's transitional rung, not here.

**Findings.** `ClaimFinding::Conflict { item, tier, low, high, distinct,
rows }` — fires at every tier, surfaced through the existing findings
render. **Reprobe nomination is anchored-tiers-only (D14)**: a Pin/Human
conflict is a "the humans must talk" probe (SL-217 stale-anchor precedent);
an Agent/Migrated conflict is an ordinary finding carrying "calibrate via
comparison" guidance and never enters the human reprobe queue.
Domain-tagged at construction (SL-219 D9).

**Code impact (§2):** new `src/comparison/claims.rs` (+ the RV-275 F-1
gate battery — §8.3); `src/comparison/store.rs` (value-system wiring,
`Pipeline` pairwise/anchor view split, `RowSummary` joins anchor rows
against `ClaimResolution` — where the claims display tokens originate);
`src/comparison/compile.rs` (mechanical `PairRow` input adaptation, zero
rule/golden churn); `src/comparison/mod.rs` (module decl);
`src/comparison/resolve.rs` (`RowState.compilation` →
`Option<CompilationStatus>` only); `src/priority/elicit.rs` (assemble
consumes the pairwise view); `src/commands/compare.rs` (list render
consumes the extended token set, goldens); deletion of the facet→AnchorMap
path in `src/priority/surface.rs`/`graph.rs`.

## §3 The resolver flip: consumption ladder and determinacy

**Current behaviour.** `graph.rs::effective_raw_value`: authored `[value]`
facet wins outright → comparison projection → `DEFAULT_VALUE`. The facet
also feeds `comparison_anchor_map`, so a hand-typed float both outranks and
*shapes* projection (REV-022 anchors-win — the laundering RFC-020
dissolves).

**Target ladder** (per item, first hit wins; RFC-020 T3). Evaluated **only
for kinds on the existing scoring surface** — `effective_raw_value` keeps
its caller contract (value-bearing scored kinds only); the consumption gate
is the caller's kind, the capture gate is `value set`'s warn-posture (D7),
pinned by a paired test over `ALL_KINDS`.

1. **Anchored claim** — `ClaimResolution.anchored` (Pin or Human tier,
   conflict means included). Wins outright and (via `anchor_map()`) shapes
   projection for compared items — the two seams the authored facet
   occupied, now with provenance.
2. **Comparison projection** — unchanged machinery; anchors now
   claim-derived only. All projection provenances feed (incl. Gauge —
   value multiplies, never divides; SL-219 D2's gauge exclusion is
   cost-side only).
3. **Agent-tier prior** — `priors` with `tier = Agent`.
4. **Migrated-tier prior** — `tier = Migrated`.
5. **Unmigrated `[value]` facet** (transitional, D6) — reads at the bottom
   of the evidence ladder, below attributed agent claims; consulted only
   when zero claim rows exist for the item (a coexisting claim means the
   facet is residue awaiting its strip). The `UnmigratedFacet` finding
   fires on facet **presence**, not rung-5 consumption (RV-278 F-4) — a
   facet shadowed by projection is still unmigrated debt.
6. **`DEFAULT_VALUE`** (1.0).

**Compared facet-bearing items — the flip stated without euphemism (RV-278
F-4).** For an item with an unmigrated facet AND comparison rows, rung 2
wins: the facet's absolute magnitude stops anchoring the value scale the
moment the binary ships, and migration does not restore it (a migrated
claim sits below projection identically — this is the flip's permanent
semantics for evidence-out-ranked guesses, not a migration-window
artifact). A corpus whose only absolute magnitudes were facets loses its
projection *anchoring* entirely until a human re-asserts (`value set
--rater human`, or a pin) — projection degrades deterministically to
gauge/bounds placements per the existing P-rules, disclosed by provenance.
D6's "keeps contributing" is scoped honestly: the facet contributes at rung
5 where no higher-rung evidence exists; where evidence exists, evidence
wins — that is the design, and the loud presence-based finding plus the
`explain` provenance line are the operator's re-assertion prompts. The
rejected-"discard" framing does not recur: nothing is deleted, everything
renders, rollback holds.

**Code motion.** `effective_raw_value` takes the claims output;
`build_from_with_cfg` gains the claims parameter (SL-219 cost-feed
precedent — pure input threaded from the shell); `comparison_anchor_map`
deleted (§2). The facet read path survives only to serve rung 5 and the
migration census; **deletion trigger: both domains migrated** (Phase 2 exit
criterion).

**Determinacy / elicitation (RFC-019 T7/D7, generalised).** The ladder is
fixed policy — no knob reorders it. `[priority.compare]
demote_agent_evidence` extends to claims: when set, agent- and
migrated-tier resolved values do not *retire* elicitation — an item on
rungs 3–5 stays probe-eligible (it has a number, not an answer). When
unset, rungs 3–5 still rank below projection but count as valued for queue
purposes. **Anchored-tier** `ClaimFinding::Conflict` items enter the
reprobe queue knob-independently; agent/migrated conflicts never do (D14).

**Behaviour-change accounting (scope R2), three VT classes:** (a) corpora
with no anchor rows and no `[value]` facets score **bitwise-identically**
(empty claims pass — the SL-213 empty-projection precedent); (b) corpora
with facets re-rank **deliberately** (rung 1 → rung 5) — the Phase 0
script's before/after diff is the accepted evidence artifact; (c) every
existing suite that doesn't author `[value]` facets stays green unchanged
(engine gate).

**Capture admissibility (D7; amends §1's first-draft line).** Anchor-claim
subject admissibility mirrors the current `value set` surface exactly —
any kind accepted today is accepted as a claim subject,
`scoring_inert_warning` preserved verbatim, pinned by a test asserting
claim-capture admissibility ≡ facet-write admissibility over `ALL_KINDS`.
`VALUE_BEARING − RSK` remains the *pairwise comparison* gate, untouched.
Scoring-inert subjects' claims resolve normally but nothing consumes them
(consumption gate above); their anchors enter `anchor_map()` where
row-gating drops them — capture-lossless, consumption-inert; Phase 3 widens
consumption, not capture.

**Config.** No new knobs. `demote_agent_evidence` semantics widen (docs +
tests). Ladder order, like value-source resolution before it, is fixed
policy (ADR-015 REV, §7).

## §4 Verb surface: `value set | pin | clear`

**`value set <id> <magnitude> --rater human|agent [--by <who>] [--basis
<text>] [--lens <lens>] [--note <text>] [--supersedes <row-uid>]`**
Appends a session-of-one anchor row (the `compare record` mint path
verbatim: shell mints uid + date, stamps `frame = value-anchor`,
`domain = value`, `form = anchor`). No entity TOML is touched. `--rater`
is **mandatory with no default** — a default would fabricate provenance
(RFC-020 T2). `--supersedes` is the explicit correction path; without it a
new row coexists as concurrent evidence (§2 conflict semantics). **Every
invocation mints (D10)** — there is no no-op guard: it cannot key on
identity without optional-`by` inference, and an accidental identical
re-fire changes no resolved value and raises no conflict.

**`value pin <id> <magnitude> --by <who> [--basis <text>] [--note <text>]
[--supersedes <row-uid>]`**
Appends an anchor row with `admission = pin`, `rater = human` stamped (not
flags — the verb IS the admission path, RV-275 F-5). `--by` **mandatory**:
a pin is a deliberate, attributed, auditable operator act. Gating (D13,
resolves RV-278 F-1 — worker-refusal alone gates workers, not agents):
`value pin` and `--retire` require **an interactive operator session** —
refused when stdin is not a TTY, with a message naming the posture — AND
join the worker-refused `WriteClass` (exact variant confirmed at
phase-plan). The TTY check reaches the pure layer as an `is_interactive:
bool` input (shell seam, date/uid pattern) so both branches unit-test; the
e2e asserts the piped-stdin refusal. Stated honestly: this is a posture
bar, not authentication — an agent driving a PTY can defeat it; the
append-only, attributed, supersedable ledger and the reprobe/contest path
are the backstop. `value set` stays in the ordinary write class: agents
*should* claim — that's the ladder's point; they just can't mint
constitutional weight from a tool shell.

**`value clear <id> [--note <text>] [--lens <lens>]`**
Appends tombstones for **all active unlensed value-domain anchor rows on
the subject** — the meaning is "this item should carry no absolute claim",
not "undo my last edit" (correction is supersession). Lens-tagged rows need
`--lens` explicitly (clearing pooled evidence must not silently destroy
lens captures). **Refused while a pin is active**, naming the remedy.

**`value pin <id> --retire [--note <text>]`**
Gated pin retirement: tombstones the active pin row(s). Same worker-refused
class. Afterwards the ladder falls through normally.

**Supersession scope.** An anchor row may supersede only an anchor row with
the identical **(subject, domain, lens)** — including `None = None` lens
equality. Foreign subject, cross-domain, pairwise-row, and cross-lens
targets are refused at capture with a message naming the mismatch (lens
partitions resolve independently; cross-partition supersession would mix
them at tier-1, violating D5).

**Severance accounting.** `run_value_set`/`run_value_clear` re-plumb from
`facet_write::apply_set/apply_clear` to the session mint; `facet_write`
survives (estimate/risk/tags; Phase 2 takes estimate). `main.rs`
write-class tests update (`value_is_write` stays true; pin verbs assert the
refused class). The `[value]` paths in `facet_write` die with the facet
read path (§3's Phase 2 trigger) — the migration script is their last
writer (the strip).

**Failure modes.** Unknown subject id: refused (existing resolve path).
Non-finite magnitude: refused (mirrors `value::validate` — any finite f64
including negatives is legal). `--supersedes` scope violations: refused at
capture, never left to resolution-time findings.

## §5 Scripts: migration and the Phase 0 baseline

Both committed, throwaway, Python-3-stdlib-only (`tomllib` to parse; **no
write-capable TOML dependency** — see strip verification). Neither commits
anything; the operator reviews and commits. **Both refuse to run on a dirty
git tree** (clean revert path). Neither is product surface: doctrine's only
contract with them is that the emitted session parses (v3) and the census
holds.

**`scripts/migrate_value_facets.py`**

1. **Scan** — every `[value]` table under the authored entity dirs
   (`.doctrine/` entity TOMLs, excluding `comparisons/`, `state/`, derived
   caches) — the surface `facet_write` ever wrote.
2. **Emit** — one session file **per run** (RV-278 F-7) in
   `.doctrine/comparisons/`: per facet, an anchor row `rater = migrated`,
   `magnitude = <facet value>`, `observed_at = <run date>`, no `date`,
   `basis = "facet [value] <relpath> @ <commit> <author> <date>"` from
   `git blame` of the `value =` line (best-effort; on failure the basis
   carries the path only — recovered context, never asserted provenance).
   Session-per-run means a re-import's superseding row lives in a *later
   session* — it can never collide with its target on the within-session R3
   identity key, so the explicit `supersedes` edge is the sole supersession
   channel and no seq-ordering subtlety is load-bearing. A re-run that
   imports nothing writes no file.
3. **Verify** — shell out to the doctrine binary to prove the emitted
   session parses and resolves (exit-0 gate) **before** any strip.
4. **Census** — every facet found accounted exactly once: `imported` /
   `already-imported` / `re-imported (superseding)`. Idempotency key =
   **facet state** (D8): skip iff an active migrated row cites the same
   source path **and** carries the same magnitude; same path + different
   magnitude ⇒ the facet changed since a prior partial run — import the
   current value as a new migrated row explicitly superseding the stale
   one (the facet's single-slot semantics, recorded honestly; exactly one
   active migrated row per source facet). Counts must reconcile
   (`facets_found == imported + already-imported + re-imported`) or the
   script aborts pre-strip. Zero provenance conversions by construction:
   the script can only write `rater = migrated`.
5. **Strip** — remove each `[value]` table by line-level text surgery,
   then verify per file: re-parse with `tomllib`, assert parsed document ==
   pre-strip parse minus the `value` key (edit-preservation proven, not
   assumed; abort + revert instructions on mismatch). `--check` runs 1–4
   and reports without writing; strip requires explicit `--execute`.
6. **Interruption safety** — rows-written-but-not-stripped is a legal
   intermediate state: §3 rung 5 consults the facet only when no claim row
   exists, so a migrated row already shadows its residue; re-running
   completes the strip idempotently.
7. **Rollback** — lossless: every row cites its source facet path and
   carries the magnitude; the strip is one git revert away (clean-tree
   precondition).

**`scripts/value_baseline.py`** (Phase 0) — two subcommands over ranking
snapshots:

- `snapshot <out.json> [--neutral]` — runs `doctrine reports survey`
  (JSON), saves `[rank, id, score]`. `--neutral`: copies `.doctrine` +
  `doctrine.toml` to a temp root, sets `[priority.coefficients] value = 0`
  there, snapshots via `doctrine reports survey -p <temp>`. The live corpus
  is never touched (pinned: before/after tree-hash comparison in the
  operational checklist).
- `diff <a.json> <b.json> [--top N]` — rank-move report: entries
  entering/leaving top-N, position deltas, score deltas.

Phase 0 evidence = `snapshot live` + `snapshot --neutral` + `diff`
committed to `.doctrine/slice/220/phase0-baseline.md` (+ raw JSONs)
**before** any code phase. The same script is the post-flip regression
instrument: `diff pre-flip-live.json post-flip-live.json` quantifies what
the flip moved, attached to the audit (scope R2's accepted-evidence
contract).

**Sequencing pin.** Phase 0 snapshot precedes every code phase; migration
runs **after** the claims machinery ships (its output must parse) and
**before** the audit (post-strip corpus for the census VT). Client corpora
that never run the script degrade gracefully per §3 rung 5 — the design
never depends on it having run.

## §6 Rendering: provenance everywhere a value appears

**`explain` value-source block** — one shape per ladder rung, replacing the
`authored` shape; `projected`/`gauge`/`default` survive verbatim:

- `value 6.5 — pin (david, 2026-07-16, basis REQ-059)`
- `value 6.0 — contested pin · 2 claims, interval (5.0 ‥ 7.0), mean —
  resolve by superseding row` (conflict variant: tier token + interval +
  row count; the "contested" framing and reprobe disclosure apply to
  anchored tiers only — an agent/migrated-tier conflict renders its
  interval with "calibrate via comparison" guidance instead, D14)
- `value 6.2 — human claim (david, 2026-07-16)`
- `value 6.2 — projected · bounds (3.2 ‥ 9.1) · from 9 constraining
  judgements (7 human, 2 agent)` (unchanged)
- `value 3.0 — agent claim (claude, 2026-07-12) · below projection — no
  projection evidence exists`
- `value 3.0 — migrated claim (unattributed · observed 2026-07-16)`
- `value 3.0 — unmigrated [value] facet — run
  scripts/migrate_value_facets.py` (rung 5, transitional)
- default: unchanged.

**Row surfaces.** `survey` / `next` / `blockers` value cells re-path to the
resolved ladder — the row model's value input becomes the resolved
`(value, tier)` (the same input `explain` consumes); `value_cell` extends
the existing source-marker convention (the `*` default marker) with markers
for the new rungs; glyphs implementation-owned, pinned by row goldens.
**No display surface reads `EntityFacets.value` after this section** — a
repo-wide grep for the field is a named verification step.

**`ReasonKind` motion (view.rs).** `ValueAuthored{value, conflict}` is
**retired** — no code path produces it post-flip. Replacements: `ValuePin`,
`ValueClaim{tier, by, date, conflict}`, `ValueFacetUnmigrated`. JSON
`value_source` is a **breaking token-set change (D11)**: `authored`
removed; `pin`/`human-claim`/`agent-claim`/`migrated-claim`/
`unmigrated-facet` added; `projected`/`gauge`/`default` byte-stable;
consumers keying on `authored` must migrate — disclosed, rides the same
release as the flip, pinned by a full post-flip vocabulary golden. Golden
churn on every explain fixture that authored a `[value]` facet is expected
and pinned as the flip's render evidence.

**`show` (entity render).** The value line re-sources from the **comparison
pipeline** (resolve → claims → compile → project) — never the priority
graph: `ClaimResolution` resolves claims for every captured subject
including non-scored kinds (capture-lossless, D7), so a record's human
claim renders fine, with the inertness annotation: `value 6.5 (human claim,
david, 2026-07-16) — scoring-inert (record kind)`. The impure scan seam
widens to `show`'s shell; the pure render consumes a resolved
`(value, provenance)` input. Absent any evidence the line is omitted
(matching today's absent-facet behaviour) — never `1.0 (default)`; the
scoring floor is not an authored fact about the entity.

**`compare list` / `elicit`.** List: the §2 token extension. Elicit:
`RenderCtx` value fragments re-path to the resolved ladder (same
`(value, provenance)` input as `show`); anchor-review candidates and §2
conflict-reprobe entries render with tier tokens.

**Findings.** `ClaimConflict` and `UnmigratedFacet` join the findings
render + JSON with domain tags (SL-219 D9), through `reports findings`
unchanged plumbing.

**Demotion disclosure.** `AGENT_DEMOTION_DISCLOSURE` (SL-218) widens to
claims: when `demote_agent_evidence` is set and any surfaced value rests on
an agent/migrated rung, the disclosure names that those values do not
retire elicitation (§3). Same single-line posture, no new config.

## §7 Governance: the REV against ADR-015

One revision (REV-NNN, minted at `/plan`), the REV-022/REV-023 pattern,
drafted at plan time, **approved before the resolver-flip phase lands**;
earlier phases (wire, claims pass, verbs, scripts) are strictly additive
and REV-independent — they change no resolution outcome until the flip
phase rewires `effective_raw_value`.

Content:

1. **Value-source resolution rewritten.** `authored → projected → default`
   (REV-022) dissolves into the T3 ladder: `pin > human claim > comparison
   projection > agent claim > migrated claim > unmigrated facet
   (transitional) > DEFAULT_VALUE`. Anchors feeding the constraint layer
   come from pin/human tiers only (D4); same-tier conflict resolves to the
   tier mean with interval bounds, surfaced, never silent (D3).
2. **The authored `[value]` facet is retired as an input** — transitional
   rung + deletion trigger recorded; `value set` now appends ledgered
   claims. The Consequences bullet "No per-item priority override … beyond
   the authored value anchor (authored-wins, REV-022)" is rewritten: the
   override is the **pin** — a governed, attributed, supersedable ledger
   row admitted through an operator-gated verb.
3. **Fixed-policy clause extended:** ladder order is policy, not a knob;
   claim resolution is deterministic given (ledger, config); conflict
   surfacing cannot be suppressed by configuration.
4. **`demote_agent_evidence` domain widened** (documented, not new config):
   agent/migrated-tier claims never retire elicitation when set.
5. **Estimate-source resolution untouched** — REV-023's ladder stands until
   Phase 2's own REV; the inter-domain asymmetry during the interregnum is
   deliberate and named.
6. **Contradiction surfacing preserved:** a pin/human anchor conflicting
   with comparison-derived bounds still quarantines via `AnchorConflict`
   (claims feed the same `AnchorMap` seam); wording gains the claim-tier
   vocabulary.

**Spec routing (D12).** Normative-semantics amendments gate with the flip:
SPEC-020's value-facet sections (facet as value surface, `value set` writer
semantics, hydration reader) amended, and PRD-014 pointed at the claims
model, in the same approval gate as the REV — canon is never stale about
what `value set` does or what resolves value while the corpus is live. Only
additive/documentary spec work (full claim-schema retention REQs,
PRD-011/SPEC-001 descent prose) remains a reconciliation obligation, and
none of it may contradict the amended normative text.

**RFC-020 status motion.** Phase 1 shipping moves RFC-020's Phase 0/1 rows
to delivered-by-SL-220 at reconciliation; the RFC stays open (Phases 2–3).

## §8 Verification plan

Suites → rules pinned; VT/VA/VH ids mint at `/plan`.

1. **Wire (v3)**: golden battery — live human anchor, pin, migrated,
   maximal/bare rows; round-trip losslessness incl. new optionals; version
   gate accepts {2,3}, rejects others with remedy; validation matrix —
   every rule in both directions (payload exactness per domain, form⇔frame
   biconditional, migrated⇔observed_at strict biconditional,
   date-required-unless-migrated, pin⇒human∧anchor, b/response presence by
   form); v2 fixture files parse unchanged (SL-219 goldens byte-identical).
2. **Resolution**: `ordering_date()` totality + deterministic mixed
   migrated/live ordering; degenerate `IdentityKey` — same-session
   same-subject anchor rows implicitly revise (R3), anchor vs order rows
   never collide; new `form_key`/`rater_key` tokens; single-subject
   lifecycle inertness; supersession chains + cycle detection over anchor
   rows; `RowState.compilation` None for anchors.
3. **Claims pass (the RV-275 F-1 gate battery — designed and green before
   the flip phase)**: tier ordering pinned
   (`pin_outranks_all_tiers_under_derived_ord`); permutation invariance;
   corroboration (N rows, one distinct magnitude ⇒ no conflict, value =
   magnitude, rows = N); conflict (multiset mean, interval {min,max},
   distinct count); conflicting pins ⇒ contested-pin finding; cross-session
   concurrency (two sessions, same subject/tier ⇒ conflict, never
   latest-wins); lens isolation — **non-vacuous** (RV-278 F-2): with
   lens-tagged anchor rows present, `lensed` is non-empty AND deleting all
   lensed rows leaves `anchored`/`priors` byte-identical (both directions
   asserted, so InertLens-input regressions fail loudly instead of passing
   vacuously); anti-laundering (property over generated ledgers:
   `anchor_map()` ≡ `anchored`, agent/migrated absent by construction);
   anchor rows reach **no** compile consumer (store pipeline and elicit
   `assemble` — an anchor-bearing ledger yields a baseline `ConstraintSet`
   identical to the same ledger without its anchor rows, RV-278 F-6);
   duplicate-merge posture (identical re-fire changes no resolved value,
   raises no conflict).
4. **Ladder (graph)**: each rung wins in isolation; adjacent-rung dominance
   pairs (pin > human > projection > agent > migrated > facet > default);
   facet consulted only when zero claim rows exist; **compared
   facet-bearing item** (RV-278 F-4): facet neither anchors the compile nor
   fills the ladder (rung 2 wins), `UnmigratedFacet` finding fires anyway
   (presence-based); row-less human claim
   resolves at rung 1 (scope R1 — the row-gating footgun test);
   scoring-inert kinds — paired capture/consumption test over `ALL_KINDS`;
   `demote_agent_evidence` — rungs 3–5 leave items probe-eligible when set,
   retire them when unset; anchored-tier conflict items enter the reprobe
   queue knob-independently, agent/migrated-tier conflicts never do (D14 —
   both directions tested).
5. **Behaviour preservation**: corpora with no anchor rows and no `[value]`
   facets score bitwise-identically (empty-claims property); the compile
   suite green with **zero golden churn** under the `PairRow` input
   adaptation (RV-278 F-3 — the "rule structure unchanged" proof); every
   existing suite green unchanged EXCEPT the enumerated goldens that author
   `[value]` facets — churn list produced at the flip phase and pinned as
   evidence (scope R2 classes a/b/c).
6. **Verbs**: `value set` mints session-of-one with stamped
   frame/domain/form; mandatory `--rater` (parse-level); every invocation
   mints (two identical sets ⇒ two rows); `--supersedes` scope refusals
   (foreign subject, cross-domain, pairwise target, cross-lens); `value
   pin` non-TTY refusal (unit: both `is_interactive` branches; e2e:
   piped-stdin refusal naming the posture) + worker-mode refusal (guard
   test) + mandatory `--by`; `value clear`
   tombstones all active unlensed rows, refuses under active pin naming
   remedy, `--lens` variant; `pin --retire` gated + ladder falls through
   after; non-finite magnitude refused; write-class regression (`main.rs`
   write_class_tests).
7. **Render**: explain shapes per rung incl. conflict + contested-pin
   variants; row-surface value cells from the resolved ladder with source
   markers (goldens); `show` resolved line, absent-evidence omission,
   scoring-inert annotation; no display surface reads `EntityFacets.value`
   (grep-gate); `compare list` goldens over a mixed fixture (anchor +
   order + superseded + tombstoned + lensed rows) asserting full token
   routing — anchor rows render `anchored`/`prior`/`conflicted`, never a
   `CompilationStatus` token; elicit render goldens — value fragments carry
   ladder tier tokens (incl. an anchor-review candidate and a
   conflict-reprobe entry with tier + interval); findings render + JSON
   parity for `ClaimConflict`/`UnmigratedFacet`; JSON `value_source` full
   post-flip vocabulary golden (breaking change pinned); demotion
   disclosure line.
8. **E2E** (`tests/e2e_value_claims.rs`): `value set` → resolution →
   `explain` provenance; pin overrides projection; human claim beats agent
   claim; migrated loses to projection; conflict → finding → superseding
   row resolves it; `clear` → tombstone → ladder falls through;
   capture-to-scoring round trip on a fixture corpus.
9. **Scripts (operational verification — throwaway, not unit-tested; the
   Rust suites pin system behaviour under migrated rows)**: both scripts
   refuse on a dirty git tree (staged + unstaged fixtures; refusal names
   the precondition); `value_baseline.py --neutral` leaves the live corpus
   untouched (before/after tree-hash check); migration `--check` census on
   a fixture corpus AND the live corpus pre-execute; post-execute census
   reconciles (`facets_found == imported + already-imported + re-imported`,
   one active migrated row per source facet); doctrine-binary parse gate
   before strip; per-file strip verification (tomllib re-parse equality
   minus `value`); re-run after execute is a no-op; interrupted-state
   fixture (rows without strip) scores via the rung-5 shadow rule. Phase 0
   baseline committed before any code phase; post-flip diff attached at
   audit.
10. **VA**: RFC-020 T2 invariant — nothing value-specific in row schema,
    tier machinery, supersession, or the resolution seam (reviewed against
    Phase 2's declared needs: estimate payload columns slot in with zero
    schema motion); capture-everything posture holds (no refusal path added
    for evidence, only for malformed provenance); REV + SPEC-020 normative
    amendments approved before the flip phase (checked at audit against the
    phase log).

## Resolved scope OQs / risks

- Scope R1 (row-gated anchor attachment) → dual-seam consumption: `anchored`
  read directly by the graph ladder; `anchor_map()` is a projection input
  only (§2).
- Scope R3 (same-tier conflict) → D3, operator-adjudicated.
- Scope OQ-1 (ladder × lens, RFC-020 OQ-2) → D5: independent per-partition
  resolution; unlensed feeds everything; no cross-partition mixing.
- Scope OQ-2 (abstention anchor-analogue, RFC-020 OQ-4) → **not built**;
  nothing forecloses it (a future `form` or response-bearing anchor
  variant rides the same wire posture). Deferred.
- Scope OQ-3 (verb surface for migration/diagnostic) → D8: scripts.

## Review history

- **Clarification loop** (2026-07-16, operator): Q1 same-tier conflict →
  mean + interval + loud finding (D3); Q2 facet retirement → physical strip
  (D9); Q3 migration + Phase 0 → throwaway scripts (D8). Compile-anchor
  assumption (pin/human only) carried and confirmed by adoption (D4).
- **Codex (GPT-5.5 default), same-session section-by-section, hostile**
  (2026-07-16), all findings accepted and integrated:
  - §1 (2 blockers, 3 majors): scalar-only payload forecloses Phase 2 →
    per-domain payload column sets (D1); `date: Option` breaks resolver
    ordering totality → `ordering_date()`; form⇔frame invariant missing
    from the validation matrix; degenerate `IdentityKey`/lifecycle/key
    tokens unspecified → made concrete; `observed_at` live-capture
    contradiction → strict biconditional.
  - §2 (1 contradiction, 4 majors): "midpoint" vs adjudicated "mean"
    transcription error → slice doc corrected; derived-`Ord` tier footgun →
    ascending declaration + pinned test; identity-based dedupe rejected
    twice (re-assertion evidence loss; optional-`by` collapse) → no dedupe
    (D3/D10); compile input filter made explicit (anchor rows terminate at
    claims); `RowState`/`RowSummary`/list-render plumbing added to code
    impact.
  - §3 (1 blocker): scoring-inert subjects would score via rung 1 →
    explicit consumption gate at the ladder seam (D7).
  - §4 (4 majors): no-op guard identity trap + supersedes suppression →
    guard deleted (D10); cross-lens supersession → (subject, domain, lens)
    scope; negative-magnitude refusal contradicted `value::validate` →
    mirror exactly.
  - §5 (1 blocker): path-only idempotency key strips un-imported facet
    edits → facet-state key + superseding re-import + census class (D8).
  - §6 (1 blocker, 2 majors): `survey`/`next`/`blockers` value cells never
    re-pathed → row-surface bullet + grep-gate; `show` sourcing from the
    scoring pipeline contradicted D7 → comparison pipeline + inertness
    annotation; "additive JSON" overclaim → disclosed breaking token-set
    change (D11).
  - §7 (1 major): spec amendments at reconciliation leave canon stale
    mid-flight → normative SPEC-020/PRD-014 amendments gate the flip phase
    (D12).
  - §8 (2 majors): `compare list`/elicit render commitments unverified →
    §8.7 pins; script dirty-tree/live-corpus safety unverified → §8.9 pins.

- **Opus (fresh-context, whole-doc, hostile) — "RV-278"** (2026-07-17,
  cross-section pass over the codex-cleared draft): 2 blockers, 4 majors,
  2 minors. Integrated in this revision: F-2 (blocker — R5 InertLens
  marking would empty `lensed` forever; lens-isolation gate test vacuous →
  claims pass performs its own input selection over {Active, InertLens}
  anchor rows; test asserts non-vacuity both directions); F-3 (major —
  `b`/`response` optionalisation mechanically touches `compile`,
  contradicting "untouched" → `PairRow` pairwise projection type; invariant
  restated behaviourally; zero-golden-churn proof; SL-219 D-NF deviation
  recorded); F-4 (major — compared facet-bearing items lose anchoring with
  no VT covering it → flip semantics stated without euphemism, D6 scoped,
  presence-based finding, dedicated ladder VT); F-6 (major — anchor rows
  leak into elicit `assemble` via `Pipeline.active_judgements` →
  pairwise/anchor view split enforced at every compile consumer, elicit.rs
  in code impact, no-consumer test); F-7 (minor — single-session migration
  collides re-imports on the R3 identity key → one session file per run,
  explicit supersedes sole channel); F-8 (minor — `display_token` cannot
  see `ClaimResolution` → tokens originate at the `RowSummary` join;
  resolve.rs claim corrected). Adjudicated by the operator 2026-07-17 and
  integrated: F-1 (blocker — worker-mode refusal does not gate non-worker
  agents → D13 interactive-TTY gate + worker refusal, honestly framed as a
  posture bar with the ledger as backstop); F-5 (major — agent
  self-contradiction floods the human reprobe queue → D14 findings at
  every tier, reprobe nomination + contested framing anchored-tiers-only).
  **Design locked 2026-07-17** (operator: "lock & hand over for dispatch").

## Deferred (named seams, not built)

- Estimate claims (Phase 2): payload columns + `cost-anchor` frame +
  migration script rerun + REV-023 successor; facet read path + `facet_write`
  `[value]`+`[estimate]` machinery delete then.
- Hierarchy admissibility (Phase 3): REQ/PRD/SPEC subjects — capture
  admissibility widening only; consumption spine per RFC-020 T6.
- Abstention anchor-analogue (RFC-020 OQ-4).
- Lens-resolved claim surfacing (IDE-035): `lensed` output exists, inert.
- Aggregation modes, cascade, container views (RFC-020 OQ-1 + ADR-018 REV).
- Magnitude coarsening config gate (RFC-020 T5).
