# Inquisition — SL-020 (Backlog entity v1)

> **HERESIS URITOR; DOCTRINA MANET**

Convened 2026-06-08 against the design-whole and v1 scope of SL-020
(`backlog-entity-v1`), status `proposed`, design under lock-gate. Target:
`slice-020.md`, `design.md`, `slice-020.toml` — tried against PRD-009,
`entity-model.md`, `glossary.md`, ADR-003/004, the work-intake membership memory,
and the `src/entity.rs` / `src/meta.rs` reuse seams.

## Charges

### C1 — False witness against the glossary (`glossary:109`). MEDIUM.
**Doctrine:** "use the CLI / sources, don't guess; cite the durable source." **Evidence:**
`slice-020.md` Design-Direction divergence table, Status row, cites the canon status
vocab as **`(glossary:109)`**. Under cross-examination `doc/glossary.md` confessed to
being **40 lines long** and contains **no status vocabulary whatsoever**. The vocab
`open|triaged|started|resolved|closed` lives solely at **`entity-model.md:109`** — which
the *same scope document* cites correctly in its Context-Bundle ("`doc/entity-model.md` …
`:109` fixes the status vocabulary"). The document bears false witness against itself.
**Risk:** a future agent follows a pointer into the void, mistrusts the canon, or
"corrects" the wrong file. **Sentence:** amend the divergence row to `(entity-model:109)`.

### C2 — The unconsecrated kind-dir: `list`/`show` over an absent reservation namespace. MEDIUM.
**Doctrine:** total-function reads; PRD-009 REQ-050 (survey) must hold from the empty
state. **Evidence:** `design.md` §5.4 `list` "for each `ItemKind`, read its tree into
`Vec<BacklogItem>`"; §5.3 wires only the **parent** `.doctrine/backlog` into
`install/manifest.toml [dirs].create`. The five per-kind dirs (`issue/…/idea`) are
**created lazily by the engine on first `new`**. Therefore on a fresh install — or for
any kind with zero items — `.doctrine/backlog/<kind>` **does not exist**, yet `list`
iterates all five and `show` reads one. No tolerance for a missing kind-dir is stated;
§5.5 declares only "empty backlog → first id per kind" (a `new` concern, not a read
concern). The cited precedent **diverges**: `spec` pre-creates *both* leaves
(`.doctrine/spec/product`, `.doctrine/spec/tech`) in the manifest — backlog wires the
parent alone while claiming "the spec subtype seam ×5." **Risk:** `backlog list` errors
on a virgin repo — REQ-050 fails at the first breath. **Sentence:** pick one and state
it — either pre-create all five kind dirs in the manifest (spec parity), or declare and
test that the backlog-local read treats a missing kind-dir as the empty set.

### C3 — The dangling promotion-origin edge (D9 ⨯ ungated re-open). MEDIUM.
**Doctrine:** ADR-004 §1 (the slice→item promotion-origin edge is **slice-authored**);
PRD-009 OQ-003 (promotion *consumes*; correction is **slice-side** — abandon the slice,
tearing down the origin edge). **Evidence:** `design.md` §5.4 D9 makes any non-terminal
transition auto-clear `resolution`, and explicitly blesses re-opening a
`resolution=promoted` item by hand ("v1 is ungated … the OQ-003 escape hatch"). But
`backlog edit <ID> --status open` reaches into the **item** only; the **slice's** authored
origin edge is untouched. Result: an *active* backlog item that a slice still claims as
its consumed promotion origin — a contradiction OQ-003 resolved by routing correction
*through the slice*. The two correction paths are not reconciled. **Risk:** silent
referential drift; the derived reverse-surface (PRD-011) will report an origin edge to a
live, unpromoted item. **Sentence:** state the interaction in §5.4/§5.5 — either name
backlog-side re-open of a `promoted` item improper (correction is slice-side per OQ-003),
or accept the dangling edge explicitly as a derived-surface reconciliation concern with a
named owner. Do not leave it unspoken.

### C4 — Conformance to hollow requirements. MEDIUM (taint inherited from PRD-009).
**Doctrine:** the storage rule (testable, stated requirements); PRD-009 as conformance
basis. **Evidence:** `design.md` §9 maps every test class "→ PRD-009 acceptance gates /
`REQ-049..053,057,058,059`." Under examination each requirement entity
(`.doctrine/requirement/0NN/requirement-0NN.md`) confessed an **empty body** — Statement
and Rationale are placeholder HTML comments. The v1-reach claim rests on requirement
**titles alone**; there is no testable statement to conform to. **Risk:** "REQ-NNN
satisfied" is unfalsifiable; `/audit` and `/close` will have nothing to check against.
**Sentence:** upstream — PRD-009 must fill REQ-049..059 Statement/Rationale before SL-020
can claim conformance. SL-020 inherits the taint, not the origin; flag it as a blocking
dependency for the design lock, not a SL-020 rewrite.

### C5 — Redundant invocation of the mobile label (`FR-006 / REQ-054`). LOW.
**Doctrine:** CLAUDE.md reference-form — "cite the **durable** id, never a mobile
membership label (`FR-/NF-` move per spec)." **Evidence:** `slice-020.md:112,227,311` all
write `PRD-009 FR-006 / REQ-054`. Not bare-`FR` (the durable `REQ-054` rides alongside),
so it escapes the stake — but `FR-006` is redundant and mobile; it will drift the day
PRD-009 reorders its members. **Sentence:** drop the `FR-006 /` prefix; keep `REQ-054`.

### C6 — Unwitnessed seeding of the plain-kind template. LOW.
**Doctrine:** the `edit` verb (D6/D8 mutation note) "refuses a file missing the seeded
`status`/`resolution`/`updated` keys." **Evidence:** `design.md` §5.3 shows the *risk*
TOML in full but renders the four plain kinds only as "omit `[facet]`." The plain template
`templates/backlog.toml` **must still seed `resolution=""`, `updated`, `tags`** or `edit`
refuses every non-risk item as malformed. Implied by the model-wide empty-string-↔-Option
seam, never shown. **Sentence:** add the round-trip assertion to §9 for **all five** kinds
(not risk alone): every scaffolded item carries the seeded mutable keys.

### C7 — Naming drift `WontDo` vs `wont_do`. TRIVIAL (footnote).
`design.md` §5.2 `Resolution::WontDo` renders kebab `wont-do`; the membership memory prose
lists `wont_do` (snake). The memory is prose, not a serialization spec — no relapse risk.
Noted only so a reviewer does not "harmonise" them into a wrong wire value.

## Acquittals (tried, found sound)
- **Engine-unchanged (R6).** Confessed true under the iron: `src/entity.rs:66` `Kind { … ,
  scaffold: fn(&ScaffoldCtx<'_>) -> anyhow::Result<Fileset> }`, `MaterialiseRequest` (:113),
  `materialise` (:237). Backlog needs only five new const `Kind`s + `Fresh`. No engine edit.
- **`meta::Meta` round-trip.** `src/meta.rs:28` — `Meta { id:u32, slug, title, status:String }`,
  **no `deny_unknown_fields`**. The design's parenthetical is accurate; the extra backlog
  keys are ignored, `status` is a `String`. The round-trip claim holds.
- **Kind set, precedence, three-never-overlap.** Faithful to PRD-009 and the membership
  memory verbatim: five glossary-reserved kinds; `problem` excluded; precedence
  `risk > issue > improvement > chore > idea`; `status ⟂ resolution ⟂ facet`.
- **`entity-model:74` (one kind + `item_kind`), `:147` (roadmap supersede), the six-vs-five
  `problem` reconciliation (OQ-001), OQ-002/003/004/005/006 honoured, ADR-004 outbound-only.**
  All cited accurately. The `problem` divergence is correctly formalised, not papered over.
- **Gitignore negation (R5).** The existing `.doctrine/*` + top-level `!.doctrine/<entity>/`
  pattern means a single `!.doctrine/backlog/` re-includes the whole subtree. Sound.
- **Success measure** "intake stops leaking" — present at PRD-009:16 and §5. Accurate.

## Questions (interrogatories)
1. **C2** — on a fresh install, what does `backlog list` do? Pre-create five kind dirs
   (spec parity), or tolerate-missing in the reader? State and test it.
2. **C3** — is hand re-opening a `promoted` item proper, or must correction route through
   the slice (OQ-003)? If proper, who reconciles the now-dangling slice-side origin edge?
3. **C4** — will PRD-009's REQ-049..059 bodies be written before this design locks, or does
   SL-020 lock against requirement titles alone?

## Pronounce Judgement
**The accused is not a heretic in substance.** The umbrella decision holds, the reuse
seams are real and verified in the code, the canon is — with one self-contradicting
exception (C1) — quoted faithfully, and the deferred layers attach without reshaping the
item. The internal adversarial pass (R1–R7) was honest work.

But the design **is not yet clean enough to lock.** Three medium taints — a false
citation (C1), an unconsecrated read path (C2), and an unreconciled re-open/origin-edge
interaction (C3) — are concrete deviations, not stylistic quibbles, and C4 reveals the
conformance basis (PRD-009's requirements) is presently **hollow**. Lock is withheld
until C1–C3 are remediated in the design text and C4's upstream dependency is acknowledged
as blocking.

## Sentencing (ordered penance)
1. **C1** — correct `slice-020.md` divergence row: `(glossary:109)` → `(entity-model:109)`.
   *Verify:* `grep -n 'glossary:109' .doctrine/slice/020/` returns nothing.
2. **C3** — add the re-open/origin-edge interaction to `design.md` §5.4 (and the §5.5
   invariant list): name backlog-side re-open of a `promoted` item improper, or accept the
   dangling edge with a named reconciler. *Verify:* a reader can answer Q2 from the text.
3. **C2** — decide and state the missing-kind-dir contract; reflect it in §5.3 (manifest)
   or §5.4 (reader). *Verify:* a `list`-on-empty-repo test appears in §9's matrix.
4. **C6** — extend §9's round-trip class to assert seeded mutable keys for **all five**
   kinds. *Verify:* the test class names "all kinds," not "risk."
5. **C5** — strip `FR-006 /`, keep `REQ-054`, at `slice-020.md:112,227,311`.
6. **C4** — record PRD-009 REQ-049..059 body-fill as a **blocking dependency** on the lock
   (a `backlog new chore` candidate once the CLI ships, or a note in PRD-009). *Verify:*
   the dependency is written down where the next agent will see it.
7. Re-submit for a second inquisition (`inquisition-2.md`) confirming C1–C3 cleared, then
   advance to `/plan`.

*Penance for the scribe who wrote `glossary:109`: to copy `doc/glossary.md` by hand, all
forty lines, and find no status vocab among them — that the false pointer be felt, not
merely told. Let the unconsecrated read path (C2) be drawn and quartered into its two
honest halves — manifest or reader — and one chosen in daylight. And let the dangling
edge (C3) not fester in silence: name it, or burn it.*

> **HERESIS URITOR; DOCTRINA MANET**
