# Inquisition — SL-026 (lazyspec read-only projection)

> **HERESIS URITOR; DOCTRINA MANET**

**Target:** the authored design (`design.md`) and scope (`slice-026.md`) of
SL-026, status `proposed`. No plan, no code yet — the heresy, if any, is doctrinal
and architectural, judged against: the integration brief
(`../lazyspec/research/lazyspec-doctrine-integration-brief.md`), the durable
decisions in `mem.thread.lazyspec.frontend-integration`, ADR-001/ADR-004, the
pure/imperative split, the storage rule, and — above all — the *actual serde of
the consumer*, `../lazyspec/src/engine/document.rs`, which the design cites but
does not read.

**Verdict in brief:** the design is fundamentally sound — layering, ADR-004
reciprocity, lossy-v1, no-masquerade, the adversarial round-1 — all aligned. But
it bears **venial heresies** that must be scourged before the design locks. Chief
among them: it declared **external and unknowable** a question that lies open in a
file it cites by line number, and left its own status table in a form that, taken
literally, makes *every* projected entity fail at the boundary.

---

## 1. Charges

### CHARGE I — The False Externalization of OQ-3 (the wire strings) ⚠ HIGH

**Doctrine violated:** "use the CLI / the source, don't guess"; the
version-fragile-pin discipline (`mem.pattern.parse.toml-error-classification-fragile`);
correctness-first.

**Confession extracted.** Design §6 OQ-3 and §8 R5 brand the exact wire strings
for `status` and relation `type` as an **"unverified, blocking, external"**
unknown — *"Pin against lazyspec's `Status`/`RelationType` serde (document.rs)
before locking the golden file."* The brief itself gives the address:
`document.rs:117`. The Inquisitor walked the three paces to that file. The answer
was lying there in the open, no fork required, no external dependency:

- `../lazyspec/src/engine/document.rs:89` — `Status` derives `Deserialize` with
  `#[serde(rename_all = "lowercase")]` and `InProgress` carries
  `#[serde(rename = "in-progress")]`. **Binding wire strings:**
  `draft` · `review` · `accepted` · `in-progress` · `complete` · `rejected` ·
  `superseded`.
- `document.rs:128` — `RelationType::ALL_STRS = ["implements","supersedes","blocks","related-to"]`,
  and `FromStr` accepts `related-to`/`related to`. The design's guessed
  `related-to` (§5.2 line 134) is **correct**.

So the relation guess was right by luck; the question was never external; and it
is now **resolved**. That a "blocking prerequisite" stood three directory hops
away, citable by line, unread — this is the heresy of the scribe who copies the
index and never opens the book.

**Aggravating evidence.** §5.3's status-mapping table writes the *right-hand
sides as lazyspec variant NAMES* — `Draft`, `Accepted`, **`InProgress`**,
`Complete`, `Review`, `Rejected`, `Superseded`. The binding contract is
`Status`'s `Deserialize`, **not** its `Display`. An emitter that copies the table
literally writes `"InProgress"` / `"Draft"` into the JSON; lazyspec's
`rename_all="lowercase"` deserializer expects `"in-progress"` / `"draft"` and
**rejects every entity as an unknown-variant**. The very risk the design names in
the abstract (R5) is *baked into its own table* in the misleading form.

**Risk:** a guessed-but-wrong string passes the in-repo golden file and silently
breaks at the lazyspec boundary — exactly the failure mode R5 describes, except
the design itself plants the seed.

**Sentencing:** record the verified strings in §5.3; rewrite the status table's
RHS as the **wire strings** (`in-progress`, not `InProgress`); downgrade OQ-3 from
"external/blocking" to **resolved**, citing `document.rs:89,128`. The golden file
encodes these verified strings. *Penance:* the breaking-wheel for the limb that
wrote "external" over an open book.

### CHARGE II — The Unguarded Sibling: the `date` wire format ⚠ MED-HIGH

**Doctrine violated:** correctness-first; the same version-fragile discipline that
earned OQ-3 its own risk entry — applied unevenly.

**Confession.** §5.2 annotates `date: String, // ISO-8601`. But the consumer's
`deserialize_naive_date` (`document.rs:11`) ultimately calls
`NaiveDate::parse_from_str(&s, "%Y-%m-%d")` — it accepts **date-only
`YYYY-MM-DD`** and nothing else. "ISO-8601" *includes* RFC3339 datetimes
(`2026-06-08T12:00:00Z`); emit one for an entity `date` and the parse **fails**.
§5.1 correctly injects `now` as RFC3339 **for `meta.generated_at`** (doctrine's
own field, not NaiveDate-parsed) — which makes it more likely an implementer
reaches for the same RFC3339 stamp for entity dates. The design interrogated the
`status`/`relation` wire strings (CHARGE I) but let their **sibling field `date`
walk free**, despite it carrying a *stricter* parser.

**Risk:** identical to R5 — silent boundary break — on a field the design never
flagged.

**Sentencing:** correct the comment to `date: String, // YYYY-MM-DD (lazyspec
parses %Y-%m-%d — NOT a datetime)`; add it to the OQ-3 family of verified wire
constraints; assert the date-only form in the conformance test.

### CHARGE III — The Plan Node Without a Birthday ⚠ MED

**Doctrine violated:** edge-case completeness (§5.5); the storage/contract
discipline.

**Confession.** `RawFrontmatter` (`document.rs:200`) makes `title`, `type`,
`status`, `author`, `date`, `tags` **mandatory** (no `#[serde(default)]`).
`date` further demands a parseable `YYYY-MM-DD`. The synthetic `PLAN-NNN` node
(D3, §5.3) draws its **status** from `PhaseRollup` — but the design **never names
the source of its `date`**. An empty or absent date → `NaiveDate` parse failure →
the plan node is rejected by the very consumer it was invented to please.

**Risk:** the headline feature of D3 (the slice→plan graph child) fails to
materialize at the boundary; or worse, fails the whole `materialize` pass.

**Sentencing:** specify the synthetic node's `date` source (the owning slice's
`updated`/`created`, or the plan.toml mtime injected as data — *not* read in the
pure layer). Add an edge-case line to §5.5 and a fixture row to the golden.

### CHARGE IV — The Hand-Wavy Axis, and the Phantom `blocks` ⚠ MED

**Doctrine violated:** "this is the contract this slice owns" (scope) vs. an owned
element left vague — *and partly fabricated* — at design lock.

**Confession under cross-examination.** §5.3's edge table maps "backlog outbound
axes → by axis (implements / **blocks** / related-to)", and §6 OQ-5 admits the
mapping is "hand-wavy … enumerate in planning." The Inquisitor enumerated it.
`src/backlog.rs:374` — `Relationships` has **exactly three** outbound axes:
`slices`, `specs`, `drift`. All three are reference/association links; **none is a
dependency or sequencing edge**. There is **no axis that maps to `blocks`** —
yet the design lists `blocks` as a backlog projection target. The contract this
slice claims to *own* names an edge type its source data cannot produce.

**Risk:** an implementer wires a `blocks` projection with no input, or guesses a
mapping; the "owned contract" is decided at the keyboard, not in review.

**Sentencing:** enumerate in §5.3 — `slices`/`specs`/`drift`, and justify each
target (the honest default for all three is `related-to`; none is `blocks` or
`implements` absent a stated reason). Strike `blocks` from the backlog row unless
an axis is shown to bear it.

### CHARGE V — Idempotence Claimed, Ordering Unpinned ⚠ MED

**Doctrine violated:** §5.4's own claim ("single-shot, stateless, **idempotent**")
vs. unpinned output order; golden-test integrity.

**Confession.** §5.1 calls `project` "deterministic given `(corpus, now,
version)`" — true *for a fixed corpus*. But the real command assembles `corpus`
from disk via the readers, whose iteration order (directory walk) is **not stated
to be sorted**. Two runs over the same tree may emit `entities[]` in different
orders → the output is **not idempotent** as §5.4 asserts, and the golden test is
deterministic only because its fixture happens to be hand-ordered — masking the
real-command nondeterminism.

**Risk:** spurious diffs in any downstream that compares output; a future
"snapshot the live corpus" test flakes; the idempotence claim is false.

**Sentencing:** pin a total order — sort `entities[]` by canonical id (and
`types[]` by name) inside `project`, before serialization. State it in §5.1 and
assert stable ordering in a conformance case. Then §5.4's idempotence is earned,
not asserted.

### CHARGE VI — The Synthetic Type With No Kind Const ⚠ LOW-MED

**Confession.** §5.3 says `types[]` is "built from `Kind` consts (prefix, dir) +
icon + plural." But §2 itself confesses **plan is not a reserved entity** —
`PLAN_KIND` shares `SL`. The synthetic `plan` lazyspec type therefore has **no
`Kind` const to build from**; its `TypeDef` (prefix `PLAN`, dir, icon, plural)
must be **hand-authored**, an explicit exception to the stated construction rule.
The design narrates the rule and the exception in different sections and never
reconciles them.

**Sentencing:** add one line to §5.3 — "the synthetic `plan` TypeDef is
hand-authored (no `Kind` const); all others derive from `Kind`."

### CHARGE VII — The Rename That Stranded the Recipe ⚠ LOW

**Confession.** D1 renames the command `emit-lazyspec-brief --json` →
`export lazyspec` (justified by no-masquerade — *sanctioned*). But the brief's
piece-4 materialize recipe (`brief §7` line ~493, `§8` line ~512) still hardcodes
`doctrine emit-lazyspec-brief --json`. Piece 4 lives in `../lazyspec` and is out
of scope here — but the design's Follow-Ups say piece 4 "rides this slice's wire
format" without flagging that **the invocation string changed**.

**Sentencing:** add a Follow-Up note — "piece-4 `materialize_doctrine_cache` must
invoke `doctrine export lazyspec` (renamed from the brief's working
`emit-lazyspec-brief`)." Coordination debt, not a doctrine-side defect.

### CHARGE VIII — Dangling Edges Vanish in Silence ⚠ LOW

**Confession.** Every `related[].target` must resolve in lazyspec's `build_links`
`id_to_path`. A target not present in `entities[]` (a `supersedes` to an
un-emitted slice, a spec `descends_from` outside the corpus) is dropped — caught
by `BrokenLinkRule`, but **suppressed by `validate_ignore: true`** (brief §6). So
a dangling edge silently disappears from tree and graph. The design addresses edge
*vocabulary* (INV-2) but not edge *referential integrity*.

**Sentencing:** acknowledge in §5.5 — dangling outbound targets are dropped
silently (validation suppressed by design); accept for v1 or filter to in-corpus
targets at projection time. A `log`/no-op note suffices; do not over-engineer.

---

### CHARGE IX — The Golden Corpus Has No Quarry (SL-027 collision) ⚠ MED

**Doctrine violated:** "no parallel implementation — find duplication before
writing"; DRY; do not re-open a just-closed issue (ISS-001).

**Confession under fresh evidence.** SL-027 (`done`) just paid down ISS-001 by
collapsing the backlog test-fixture builders into one seam —
`Fixture<'a>`/`FacetLit`/`RelLit` + `toml_list`/`render_fixture_toml`/
`write_fixture` (design §3, audit F1). SL-026 §9 demands a "minimal corpus →
expected Brief JSON" golden test, and that corpus **must include backlog items**
(§5.3 projects backlog → five lazyspec types). The design says nothing about
*where the fixtures come from*.

Two heresies hide in that silence:

1. **Re-triplication.** If SL-026 hand-rolls backlog-NNN.toml literals for its
   corpus, it re-incurs the exact debt SL-027 just retired — a *second* copy of
   the head literal, in a *second* module, days after ISS-001 closed.
2. **The builder is imprisoned.** SL-027's `write_fixture`/`Fixture` live in
   `src/backlog.rs`'s `#[cfg(test)] mod tests` — a **private** test module.
   SL-026's tests (in `src/lazyspec.rs` or `tests/`) **cannot reach them** without
   promoting the seam to a `pub(crate)` test-support location. SL-026 inherits a
   visibility decision SL-027 had no reason to make.

**Aggravating.** Backlog is only *one* of five corpus kinds. There is **no**
unified fixture builder for `slice`/`spec`(+members+reqs)/`adr` — SL-026's corpus
construction is a non-trivial, multi-kind fixture problem, and §9 waves at it with
"minimal corpus." Worse, §5.1 makes `project` pure over a `Corpus` of **loaded
structs** (`Vec<Item>`, `Vec<Spec>`, …) — building those in-memory is blocked by
**private struct fields** across modules (backlog `Item.relationships` is private),
so the corpus must either go through on-disk TOML + the real loaders (needs the
fixture builders) **or** force test-constructor visibility on every entity. Either
path is a deliberate test-support seam, not a footnote.

**Sentencing:** §9 must state the corpus-construction strategy and consciously
**reuse** SL-027's builder (promote it to a shared `pub(crate)` test-support seam
— a `/consult`-worthy decision, *not* an execute-time improvisation), rather than
re-roll backlog TOML. Name the slice/spec/adr fixture gap explicitly. *Penance:*
the heretic who copies a literal a sixth time, the week ISS-001 burned, joins it
at the stake.

### CHARGE X — Mapping the Dead Kingdom (SL-028 collision) ⚠ HIGH

**Doctrine violated:** correctness-first; totality of a status projection over a
model that *tolerates drift*; coupling discipline (a consumer built on a
vocabulary another in-flight slice is replacing).

**Confession.** SL-026 §5.3 maps the slice status thus:
`{proposed→Draft, ready→Accepted, started→InProgress, audit→InProgress,
done→Complete, abandoned→Rejected}` — **six** states, **no `else` arm**.

The Inquisitor read the source. `src/slice.rs:349` —
`SLICE_STATUSES = ["proposed","ready","started","audit","done","abandoned"]` is a
**free-`String` allowlist, not an enum** (`status: String`, `:468/:491`), and
`:368` `is_drifted` confesses that an **out-of-vocab stored status is tolerated on
disk** and rendered with a `?` marker, *never rejected*. So a total mapping is
mandatory **today**: a drifted status has **no lazyspec target** in the 6-entry
map, and `DocMeta.status` is **mandatory + must be one of the 7 wire strings**
(CHARGE I). A partial map either panics or silently invents a status at the
boundary.

And the kingdom it maps is **already condemned.** SL-028 (`proposed`, design
locked 2026-06-09) **replaces the lifecycle vocabulary** with a 10-state FSM:
`proposed → design → plan → ready → started → review → audit → reconcile → done`
(+`abandoned`). Four live states — **`design`, `plan`, `review`, `reconcile`** —
have **no row** in SL-026's map. Whichever of SL-026/SL-028 lands second, the
slice-status projection is stale-on-arrival: SL-026 maps a vocabulary SL-028
deletes, and SL-028 owns the vocabulary SL-026 silently consumes.

**Risk:** every slice in a `design`/`plan`/`review`/`reconcile` state — i.e. most
*active* slices, the ones a lazyspec viewer most wants to see — projects to a wrong
or fallback status, silently, at the boundary. The very break-class of CHARGE I,
now driven by an *adjacent slice's* roadmap.

**Sentencing:**
1. Make the status projection **total** — an explicit default arm for unknown /
   drifted statuses (the honest default is `Draft` or a documented "unknown"),
   so a tolerated-drift status never breaks the wire.
2. Map the **SL-028 FSM** now (or declare the dependency): add
   `design→Draft`/`plan→Draft`(or `Review`)/`review→InProgress`/
   `reconcile→InProgress` so the map survives SL-028 landing.
3. Record SL-028 as an **outbound relation / risk** on SL-026 (the lifecycle
   vocabulary is a shared contract SL-028 changes; ADR-004 — store the edge
   outbound). Sequence-coordinate the two, or SL-026 ships a map to a dead
   kingdom.

*Minor, same family:* SL-028 also stubs requirement-lifecycle + coverage enums
into `src/requirement.rs`/`src/spec.rs` — the very modules SL-026 reads and widens
to `pub(crate)` (CHARGE-set §5.1/F5). Both slices edit `spec.rs`; coordinate the
visibility edits so neither clobbers the other. LOW.

## 2. Questions (interrogatories)

1. **Command name — final?** `export lazyspec` (D1). Confirm, so the brief's
   piece-4 recipe can be amended and the rename closed out.
2. **Entity `date` — which?** created or updated (or per-kind)? Binds CHARGE II/III
   and the status/date fixtures.
3. **Plan node `date` — source?** Owning slice's date, or plan.toml mtime injected
   as data? (CHARGE III.)
4. **OQ-4 (body tier) & OQ-5 (backlog axes) — resolve now or in plan?** OQ-5 is
   fully enumerable today (3 axes; CHARGE IV) — recommend resolving in design;
   OQ-4 is a legitimate planning-time per-kind call.
5. **Corpus source (CHARGE IX):** reuse SL-027's `write_fixture` (promote to a
   `pub(crate)` test-support seam) or build the `Corpus` structs in-memory (needs
   test constructors / field visibility)? This is a `/consult`-grade design call.
6. **Sequencing vs SL-028 (CHARGE X):** does SL-026 land *before* or *after*
   SL-028's lifecycle FSM? Drives whether SL-026 maps the new 10-state vocabulary
   now or carries an explicit dependency edge.
7. **Lock gate:** with OQ-3 resolved (CHARGE I), is the User content to lock once
   CHARGES I–V **and X** are folded in, leaving VI–IX as design-text + planning
   touch-ups?

---

## 3. Pronounce Judgement

**This is heresy — but venial, not mortal.** The design's bones are orthodox: the
layering reasoning (F2's pure-function-not-leaf correction), ADR-004 outbound-only
with derived reciprocity, the lossy-v1 covenant, the no-masquerade command name,
and a genuine adversarial round-1 that did real work. The architecture may stand.

But it confessed three sins that bar the lock:
- it branded **external and unknowable** (OQ-3) a fact lying open in a cited file,
  then encoded the *wrong* form of that fact in its own status table (CHARGE I);
- it scourged the `status`/`relation` wire strings while letting their sibling
  `date` — bearing a *stricter* parser — walk free (CHARGE II), and left the plan
  node with no birthday at all (CHARGE III);
- it claimed ownership of a contract (backlog axes) it left vague and partly
  **fabricated** — a `blocks` target no source axis can feed (CHARGE IV) — and
  claimed an idempotence its unpinned ordering does not earn (CHARGE V).

And — revealed only by reading the **adjacent** slices the design predates — it
maps a kingdom already condemned: SL-028 (`proposed`) replaces the slice
lifecycle vocabulary out from under SL-026's 6-state map (CHARGE X), and SL-027
(`done`) just built the backlog fixture seam SL-026's golden corpus must ride but
cannot reach (CHARGE IX). These are not SL-026's faults of authorship — they are
**coupling blind spots**: a slice designed in isolation against neighbours in
motion. The Inquisitor names them so the design is not blind to the surface its
neighbours are reshaping.

None taints the core mechanism. All are remediable in the design text, the
conformance suite, and a relation edge or two. **Do not lock until CHARGES I–V
and X are folded in; resolve IX before the golden test is written.**

## 4. Sentencing (ordered penance)

1. **CHARGE I** — fold the verified wire strings into §5.3; rewrite the status
   table RHS as wire strings (`in-progress`, …); mark OQ-3 **resolved**
   (cite `document.rs:89,128`). *Verify:* the golden file contains the literal
   lowercase/hyphenated strings; a conformance case asserts the exact 7-status and
   4-relation vocabularies.
2. **CHARGE II** — fix the `date` comment to `YYYY-MM-DD`; add to the verified-wire
   set. *Verify:* conformance asserts date-only; a datetime fixture is a negative
   test (or documented as forbidden).
3. **CHARGE III** — specify the plan node's `date` source (slice date / plan mtime
   injected as data). *Verify:* golden has a `PLAN-NNN` row with a parseable date.
4. **CHARGE IV** — enumerate `slices`/`specs`/`drift` in §5.3 with justified
   targets; strike `blocks` unless an axis bears it. *Verify:* OQ-5 closed in
   design; conformance covers each axis.
5. **CHARGE V** — sort `entities[]` by id and `types[]` by name in `project`;
   state it in §5.1. *Verify:* an ordering conformance case; the idempotence claim
   in §5.4 now holds.
6. **CHARGE X** — make the status projection **total** (explicit default arm for
   drifted/unknown status); map the SL-028 10-state FSM
   (`design`/`plan`/`review`/`reconcile`); record SL-028 as an outbound
   relation/risk on SL-026. *Verify:* a conformance case feeds an unknown status
   and asserts the default; the map covers every `SLICE_STATUSES` member plus the
   SL-028 additions.
7. **CHARGE IX** — §9 states the corpus strategy and reuses SL-027's
   `write_fixture` via a promoted `pub(crate)` test-support seam (`/consult` the
   promotion); name the slice/spec/adr fixture gap. *Verify:* no new
   `backlog-NNN.toml` head literal appears in SL-026's diff; `grep` for
   `created = \"…\"` in `src/lazyspec.rs` tests returns 0.
8. **CHARGES VI–VIII** — design-text touch-ups: hand-authored plan TypeDef
   exception (§5.3); piece-4 rename Follow-Up; dangling-edge edge case (§5.5).
   *Verify:* present in the design before lock.

For the scribe who wrote "external" across an open book: let the public square
witness the **breaking on the wheel**, that no future Inquisitor again finds a
"blocking unknown" three directory hops from a `sed -n`. The doctrine endures; the
guessed string burns.

> **HERESIS URITOR; DOCTRINA MANET**
