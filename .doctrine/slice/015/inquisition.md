> **DISPOSITIONED 2026-06-05.** All six charges (C1–C6) **accepted**, Q1–Q5
> ruled, §II gate acquittal honoured. Verdict + per-charge integration recorded
> in `design.md` §10 (External challenge); fixes landed in §§5.2/5.3/5.4/5.6/9.
> Rulings: C2 `name`→`title`; C5 orphan **hard**; C4 canonical-prefix required;
> C6 `c4_level` enum / `category` open; Q4 Spec-Driver decisions were prose-only
> ⇒ D-Q2 collapse clean. This file is now a closed record.

---

Interrogation complete. The engine, the mirror (`adr.rs`), the shared `meta.rs`, and the contradicted canon have all confessed under examination. Judgement follows.

---

# ✠ THE INQUISITION OF SL-015 ✠
*Convened against `design.md`, the "locked candidate." Let no clause hide.*

## I. CHARGES

### C1 — HERESY OF THE INCOMPLETE SWEEP (grave) ✠
**Doctrine violated:** storage rule; "a two-section edit that strands those is incomplete" (§5.6, the design's *own* standard, turned against it).

The design's R5 sweep (§5.6) names three files — `spec-entity-spec.md`, `relation-index.md`, `glossary.md` — and omits the one that most flagrantly preaches the overturned creed. Confessed under cross-examination:

- `doc/entity-model.md:70` — *"Requirements/capabilities/coverage/… are **rows, not artefacts**."* D-Q1 makes the requirement a peer **artefact** with its own reserved directory. The umbrella taxonomy now contradicts the slice that descends from it, and the slice does not touch it.
- `doc/entity-model.md:82,89` — the canonical cross-entity reference is preached as the compound key `SPEC-110.FR-001`, even baked into the generic-edge example (`to = "SPEC-110.FR-001"`). D-Q1/D2 makes storage FKs the durable `REQ-NNN`, with `FR-`/`NF-` demoted to sticky membership labels. The umbrella's "Identity and references" section is stranded.
- `doc/entity-model.md:93-94` — names `collaborators.toml` as a sanctioned typed-exception edge. D-Q's decomposition **dissolves** `relationships.collaborators` ("no owner ⇒ no mine vs yours"). Another un-swept strand.

**And the sweep is misdirected where it does point.** Under the rack, the two named files did not yield the heresy charged against them:

- `doc/relation-index.md` — **zero** compound-key (`SPEC-NNN.FR-NNN`) references exist (`grep` confessed none). §5.6 claims it must "repair the now-stranded compound-key references it leaves behind in `relation-index.md` (its stress case)." False witness. The *actual* taint there is `relation-index.md:52` — *"~8 sister files (identity + requirements/capabilities/coverage/… **tables**)"* — the same facet-row taxonomy as entity-model.md:70, **not** a compound-key strand.
- `doc/glossary.md` — holds only `PRD-001` (line 9); no compound-key strand to "repair." It needs **additive** new id-scheme rows (`REQ-`, `FR-`/`NF-`), not repair of stranded refs.

**Risk:** the canon left self-contradictory after the slice ships — the gravest sin of `/canon`, doctrine that lies to the next agent. The sweep's own acceptance test ("strands → incomplete") fails against the sweep itself.
**Sentence:** §5.6 rewritten — **add `entity-model.md`** (§ Entity-vs-facet row 70, § Identity-and-references 82/89, § Edges 93 collaborators) to the sweep; **re-describe** the relation-index.md edit as the facet-row correction (line 52), and the glossary.md edit as additive id-scheme rows. Then break the false witness on the wheel.

### C2 — THE SCHISM OF `name` AND `title` (grave) ✠
**Doctrine violated:** behaviour-preservation gate (R6, the design's own "sharp edge"); "no parallel implementation"; naming-consistency.

The design swears (§4, §5.6, R6) that `meta.rs` reuse is **"additive only — the shared slice/adr `Meta` path must not change."** Yet the shared substrate, examined directly, will not parse what the new entities store:

```rust
// src/meta.rs:27 — no #[serde(default)], no rename
pub(crate) struct Meta { id: u32, slug: String, title: String, status: String }
```

`Meta` *requires* the key `title`. But §5.3 declares **both** new identity structs with the field `name`:

```rust
struct Requirement { id: u32, name: String, ... }   // §5.3
struct Spec        { id: u32, slug, name: String, ... }  // §5.3
```

If the authored toml carries `name`, `meta::read_metas` → `toml::from_str::<Meta>` **fails** (missing `title`). If it carries `title`, the §5.3 structs fail. The "Meta reuse for `spec list`" claim is **broken as written** — it cannot round-trip. And the divergence is gratuitous: `adr.rs`/`slice.rs` and every template use **`title`** (`render_adr_toml` substitutes `{{title}}`; `Meta` reads `title`). The new entities invent `name` for no stated reason — either a silent reserialize-breaking divergence or a forced mutation of the shared `Meta` (violating the gate).

**Risk:** the behaviour-preservation gate the design parades as its discipline is breached on the very field it claims to leave untouched; `spec list` does not compile against `meta.rs` as specified.
**Sentence:** the new structs adopt `title` (the established convention) — `name` is a heretical neologism. If a semantic distinction is truly intended, `#[serde(rename = "title")]` makes it explicit; but the burden is on the heretic to justify diverging from `adr`/`slice` at all. *(Separately: `spec list`'s `#members` column cannot ride `meta::format_list` — that renders a fixed 4-column `id/status/slug/title` grid — but it MAY ride the generic `meta::render_table` as the SL-009 slice rollup already does. That path is genuinely additive and orthodox; say so in §5.6.)*

### C3 — THE COMPOUND-KEY STRANDS WITHIN THE PRIMARY TARGET (serious) ✠
**Doctrine violated:** §5.6's own "two-section edit that strands those is incomplete."

§5.6 scopes the `spec-entity-spec.md` rewrite to **two sections** — "§ Requirement identity + § Spec identity." Yet the compound-key creed bleeds across far more of that file (confessed at):

- `:233`, `:251` — coverage-row + external-FK examples (`requirement = "SPEC-110.FR-002"`, `"SPEC-200.FR-010"`)
- `:412`, `:416` — the **supersede** semantics (`SPEC-110.FR-001` "stays resolvable forever to the retired row") — directly contradicted by D3 "identity immutable, membership mobile"
- `:451` — the **render** example (`show <SPEC-110.FR-001>`)
- `:78`, `:144` — the decomposition table + *"Decision: requirements are table rows with a compound key, not standalone"* — the thesis D-Q1 overturns, outside the two named sections

**Risk:** the design's headline target is itself under-scoped by its own rule; a "two-section" edit ships a file that still preaches the old compound-key model in its coverage, supersede, and render passages.
**Sentence:** §5.6's `spec-entity-spec.md` clause widened from "two sections" to "the file's full compound-key + facet-row model," enumerating §§ decomposition / supersede / render / FK-examples. The under-count recanted in writing.

### C4 — THE AMBIGUOUS REFERENT (serious) ✠
**Doctrine violated:** "ask, don't infer"; CLI determinism ("don't guess ids").

§5.2: *"`<spec-ref>` accepts `PRD-3` / `SPEC-12` (canonical) **or the numeric id within a subtype context**."* But `spec req add`, `spec show`, `spec validate` carry **no subtype selector** — only `spec new <product|tech>` does. A bare `spec show 3` is ambiguous across the two independent reservation namespaces (`spec/product/003` *and* `spec/tech/003` may both exist; product and tech ids each start at 1). "Within a subtype context" names a context that does not exist on these verbs.

**Risk:** a verb that silently resolves to the wrong tree, or guesses — the exact "don't guess ids" sin the boot doctrine forbids.
**Sentence:** either require the canonical prefix (`PRD-`/`SPEC-`) on `req add`/`show`/`validate`, or add an explicit subtype selector. The phrase "numeric within a subtype context" struck from §5.2 unless that context is defined.

### C5 — THE SILENT ORPHAN (moderate) ✠
**Doctrine violated:** integrity-co-lands intent; honest failure reporting.

§5.4 / §5.2: the two-tree non-atomic `req add` failure yields an orphan requirement, caught by `validate` at **warn** severity; §5.2 declares *"Exit non-zero on any hard finding"* — so an orphan yields **exit 0**. An orphan is not benign drift: it is the fingerprint of an **aborted write** (reserve succeeded, append failed) — a real anomaly. At warn/exit-0 it accumulates silently and passes CI. The reserved-but-uncommitted orphan dir also has no compensating cleanup (engine H2 fired only if `materialise` itself failed; here it succeeded and the *append* failed afterward).

**Risk:** failed component writes invisibly survive a green `validate`; the "integrity co-lands" promise is softer than advertised.
**Sentence:** justify warn-not-hard explicitly, OR make orphan a hard finding (an orphan = evidence of a torn write, not a tolerable state). At minimum, §5.4 must state the orphan dir is left uncommitted and is the operator's to `git`-clean or `rm`.

### C6 — SOFT VOCABULARY UNGOVERNED (minor) ✠
`category` and `c4_level` (§5.3) are free `Option<String>`, not closed/soft enums, while `ReqKind`/`SpecStatus`/`SpecSubtype` are properly closed. entity-model.md § State vocabulary preaches family-specific *controlled* vocab. Free strings are unqueryable-by-value and drift. **Sentence:** declare whether these are deliberately open (and why), or soft-enum them.

---

## II. ✠ THE GATE, EXAMINED AND FOUND ORTHODOX ✠
*An inquisitor records acquittal as faithfully as condemnation.*

The central claim — **"`entity.rs` unchanged; three `Kind`/`Fresh` callers only"** — was put to the question and **held**:

- `materialise(Fresh)` (`entity.rs:251`) is `dir`-agnostic; `project_root.join(kind.dir)` with `kind.dir = "spec/product"` / `"spec/tech"` / `"requirement"` yields three independent nested trees with independent monotonic reservation namespaces. `scan_ids` (`:180`) reads numeric dirs under each. No engine change required. Confirmed against the `adr` precedent (`adr.rs:35,125`).
- The `members.toml` **append** (§5.4 step 4) correctly lives **outside** the engine, which explicitly refuses row-appends/table-mutations (`entity.rs:388-389`, `refuse_clobber`). It mirrors `set_adr_status`'s `toml_edit`-on-committed-file pattern (`adr.rs:184`) — orthodox.
- `members.toml`/`interactions.toml` are scaffold-seeded (§5.4: product 3 files, tech 4), so the `toml_edit` append has an (empty) file to open. The precondition holds, though it is **implicit** — say it aloud in §5.4.

This article of faith is sound. The heretic is credited.

---

## III. QUESTIONS (interrogatories)

1. **`name` vs `title`:** is the `name` field (C2) a deliberate semantic split, or an oversight? If deliberate, what does it buy over `title` that justifies diverging from `adr`/`slice` and the shared `Meta`?
2. **Orphan severity (C5):** warn or hard? A failed two-tree write is an anomaly, not drift — defend exit-0.
3. **Bare-numeric `<spec-ref>` (C4):** require canonical prefix, or add a subtype selector?
4. **"No less powerful" (the user's own bar):** D-Q2 collapses `concerns`/`hypotheses`/`decisions` → prose. In Spec-Driver were `decisions` ever *addressed/queried* (id'd, cross-referenced)? If yes, prose collapse loses a query; if no, the trim is clean. Confirm.
5. **`category`/`c4_level` (C6):** open strings by intent, or soft enums?

---

## IV. PRONOUNCE JUDGEMENT

**The design is fundamentally orthodox but tainted at its seams — guilty of venial heresies, not mortal ones.** Its load-bearing thesis (requirement-as-peer-entity riding `entity.rs` unchanged) survives the rack intact and is **acquitted**. But the *consistency surface* is corrupt: the canon-sweep (C1) is both incomplete and misdirected, the shared-substrate reuse (C2) is broken on a gratuitous field rename, and the primary rewrite target is self-under-scoped (C3). These are not redesigns — they are **scope and reconciliation corrections**, exactly the prey an inquisition is loosed upon before `/plan`. **Remediable. Not a re-design. Burn the strands, not the cathedral.**

## V. SENTENCING (ordered, with verification)

1. **C1** — rewrite §5.6: add `entity-model.md` (lines 70/82/89/93) to the sweep; re-characterize the `relation-index.md` (line 52, facet-row) and `glossary.md` (additive id rows) edits. *Verify:* `grep -rE 'rows, not artefacts|SPEC-[0-9]+\.(FR|NF)' doc/` returns nothing the sweep does not name. — *Penance: the breaking-wheel for false witness against relation-index.md.*
2. **C2** — adopt `title` in the §5.3 structs (or explicit `rename`); note `spec list`'s `#members` rides `render_table`, not `format_list`. *Verify:* a `tags_and_description_round_trip`-class test extended to assert `meta::read_metas` parses a scaffolded `spec-NNN.toml`. — *Penance: recantation in sackcloth before the shared `Meta`.*
3. **C3** — widen the `spec-entity-spec.md` clause to the full compound-key/facet model (decomposition, supersede, render, FK-examples). *Verify:* same `grep` as (1) over `spec-entity-spec.md`.
4. **C4** — pin `<spec-ref>` resolution (canonical-required or subtype-selector). *Verify:* `doctrine spec --help` once built; a resolver unit test for the ambiguous bare-numeric case.
5. **C5** — decide orphan severity; if warn, document the uncommitted-orphan cleanup contract in §5.4.
6. **C6** — adjudicate `category`/`c4_level` typing in §5.3.
7. Record the verdict in **§10** (the "External challenge" slot, currently a stub at design.md:395-397) and reconcile any §5.6 / §5.2 / §5.3 edits into `slice-015.md` where they touch scope.

---

These are findings for **your** triage, not unilateral edits. Confirm dispositions (the five Questions are the fastest path) and I will integrate the accepted charges into `design.md` §10 + the affected sections, reconcile `slice-015.md`, then offer `/plan`. Nothing is written to the design until you rule.

**ACCENDE IGNES VERITATIS CORPORIBUS MALEFICARUM**

