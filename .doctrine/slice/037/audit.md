# SL-037 — Inquisition of the locked design

> **HERESIS URITOR; DOCTRINA MANET**

Target: `.doctrine/slice/037/design.md` (decisions D1–D9, §3 column model, §4
per-kind migration, §6 verification, §8 risks). Doctrine consulted: ADR-001
(layering), SL-025 design (A-1/A-2/A-3/D7), the storage rule, IMP-009 / IMP-013 /
IMP-014 / IMP-017, and the living source the design rides (`listing.rs`,
`main.rs`, `backlog.rs`, `slice.rs`, `spec.rs`, `governance.rs`, `memory.rs`,
`boot.rs`).

Posture: hostile review only. The gate (`no code without an approved plan`)
holds. Five charges below; each dispositioned. The design must be re-approved
after the design-change charges are integrated.

---

## Charges

### CHARGE I — MAJOR — the spec extractor cannot keep its non-capturing vow (D5 ✗; the IMP-013 R1 risk, made flesh)

**Doctrine violated:** D5 ("extractors are non-capturing `fn(&R)->String`;
external context is pushed into the row type `R`, not captured"); §4 spec bullet;
the IMP-013/R1 over-config warning.

**Evidence, confessed under cross-examination:**
- spec's prefixed id is subtype-dependent: `spec.rs:1088` calls
  `canonical_id(subtype, m.id)` — `SpecSubtype::Product → PRD`,
  `Tech → SPEC`. The prefix is a **runtime value**, not a module const.
- §4 declares spec's row type `R = (Meta, usize)` **"(existing)"**. That tuple
  carries **no subtype**. An `id` column extractor `fn(&(Meta, usize)) -> String`
  therefore **cannot** produce the prefixed id without capturing `subtype` —
  which D5 forbids.
- The design already knows the cure and applies it elsewhere: governance's bullet
  pre-materialises the prefixed id into `GovRow` ("Built per-call where `GovKind`
  is in scope so the prefixed id is materialised before extraction"). slice and
  backlog escape the trap because their prefix source lives *in* the row
  (`SLICE_KIND.prefix` is a const; `BacklogItem.kind` is a field). **spec is the
  unique kind whose external context is absent from the stated `R`.**

**Risk:** This is precisely the config-surface fracture IMP-013 (R1) warned of,
surfacing at the one kind the user ordered attacked hardest. Left as written, the
plan inherits a row type that does not compile against D5; the implementer either
silently captures `subtype` (breaching D5 / re-introducing `Box<dyn Fn>`
pressure) or improvises a row type the design never sanctioned.

**Sentence — DESIGN CHANGE.** Correct §4: spec's `R` is **not** `(Meta, usize)`
but a small pre-materialised display row (prefixed-id `String` + status + slug +
`members: usize`), built per-block where `subtype` is in scope — exactly the
governance/`GovRow` pattern. Then every spec extractor is a trivial non-capturing
`fn(&R)->String` and D5 holds across all four kinds. Verify: a compiling
`select_columns`/`render_columns` over spec in the per-kind unit (§6) and a spec
golden that pins the prefixed ids per subtype.

### CHARGE II — MODERATE — D2 closes IMP-013 while half its named scope (the JSON row-assembly) stays fully duplicated

**Doctrine violated:** IMP-013's own writ; honest-closure (the design states it
"Resolves … IMP-013").

**Evidence:** IMP-013 confesses its lift to be "**row assembly + table + JSON**,
parameterized by a per-kind column/decoration spec" and "the lift is the
row-assembly + **JSON layer** above [the table], not the table itself." D2 lifts
**only the table column projection** and leaves the JSON path per-kind: the
`*Row` structs (`BacklogRow`, `SliceRow`, `SpecRow`, `GovRow`) and the five
`json_rows` assemblers remain duplicated, untouched. The slice's design closes
IMP-013 outright regardless.

**Cross-examination (the defence weighed):** D2's *engineering* is sound. The
typed rows carry structured values a `String` column cannot — slice `phases` is
an object, spec `members` an int, backlog `resolution` nullable — so stringly
columns would regress SL-025 D7 (faithful JSON). The `json_envelope` +
build→retain→sort skeleton is *already* shared (SL-025). What remains per-kind is
genuinely irreducible at the struct level.

**Risk:** Not a code defect — a **bookkeeping heresy**. Closing IMP-013 as fully
resolved when the JSON-row half it explicitly scoped was deliberately *not*
delivered makes the backlog lie. A future reader trusts IMP-013 as done and never
revisits the duplicated `json_rows`.

**Sentence — DESIGN CHANGE (closure-scoping) + DEFER.** D2 must state plainly
that the JSON-row-assembly half of IMP-013 is **assessed and deliberately
descoped** (typed rows irreducible under D7), and that IMP-013's `/close`
resolution will record this — OR spin a thin follow-up (or fold into IMP-017's
orbit) for any future JSON-row convergence. Do not let IMP-013 close silent and
whole. Verify: the §8 / closure-intent text names the descope; the `/close`
reconciliation cites it.

### CHARGE III — MODERATE — D9's silent accept-but-ignore `--columns` on memory is a least-surprise heresy; a third, more honest option was under-weighed

**Doctrine violated:** least-surprise; the design's own R4 ("a bounded documented
gap"). The user explicitly ordered: *challenge both directions.*

**Evidence & verdict:** The **defer** itself is sound and well-argued — memory has
no slug (IMP-009's driver absent), its cells are `scrub_line`d
(`memory.rs:1088-1090`, the R5/IMP-017 security invariant), it is the strongest
over-abstraction case, and it has no triggering edit. **Migrating memory now is
correctly rejected.** But the framing "no-op vs migrate" is a false binary. A
third option dominates the chosen one:

- **Chosen (D9):** `--columns` on `memory list` is **silently accepted and
  ignored** — a classic footgun. `doctrine memory list --columns uid,title`
  parses (the flag rides the shared `CommonListArgs`) and does nothing. "Documented
  on the flag" is weak defence; users do not read `--help` mid-command. Worse, when
  IMP-017 later wires memory, that same invocation **silently changes behaviour** —
  a second surprise.
- **Superior:** **reject** `--columns` on memory with a clean error ("`--columns`
  not supported for `memory list`"). It fails loud now, and IMP-017 simply removes
  the guard — no silent behaviour shift. This fits the **existing** per-kind read
  validation seam exactly: `validate_statuses` is already called per-kind in
  memory's `list_rows` (`memory.rs:~1288`); a `columns.is_some()` guard is the same
  shape, command- or leaf-side, no spine fracture.

**Risk:** A shipped flag that silently does nothing is the kind of quiet lie the
inquisition exists to burn.

**Sentence — DESIGN CHANGE.** Keep the defer; replace the silent no-op with a
loud rejection on `memory list` until IMP-017 lands. Update D9, §5's flag doc,
and R4. Verify: a black-box test asserting `memory list --columns …` errors with
the unsupported message (this also closes part of CHARGE IV).

### CHARGE IV — MODERATE — the IMP-014 harness (§6) leaves load-bearing claims unverified

**Doctrine violated:** IMP-014's mandate; mem `conformance-asserts-surface`
("assert every surface, not just the JSON envelope"); the design's own D9/R4
no-op claim.

**Evidence — coverage gaps in §6's harness enumeration (default table /
`--columns` table / `--json`, per the four migrating verbs):**
- (a) **memory's no-op is unpinned.** §6 covers "the four slug-bearing verbs"
  only. D9/R4 assert a memory `--columns` no-op (or, post-CHARGE-III, a
  rejection) — that behavioural claim has **no test**. An unverified claim is
  contraband.
- (b) **empty-list output unpinned.** `render_columns` returns `""` to suppress
  the header on empty input (§3, §5.5). §6 pins populated output only; the
  virgin-empty path per verb is unguarded.
- (c) **spec's multi-block layout not enumerated.** R3 names "a spec golden," but
  §6's harness list does not explicitly pin the per-subtype labelled-block
  structure **nor** the omitted-empty-block case (`spec.rs:1076-1094`) — the exact
  fragility the lift threatens.
- (d) **governance breadth unstated.** `adr`/`policy`/`standard` share one
  `render_table` (`governance.rs:98`). §6 should either pin all three or state
  that one representative suffices because the path is shared.

**Sentence — DESIGN CHANGE.** Extend §6's coverage list to enumerate (a)–(d).
Verify: the harness fixture and assertions name each.

### CHARGE V — MINOR — false universals and miscounts in the D6/R4 behaviour-preservation argument

**Doctrine violated:** precision in a behaviour-preservation justification (the
gate that protects shared machinery).

**Evidence, three imprecisions — the load-bearing claim survives, the scaffolding
around it does not:**
1. "**6 `build()` callers**" (D6) / "memory is the **6th** `build()` caller"
   (R4): there are **5** source-level `listing::build(args)` call sites
   (`backlog`, `slice`, `spec`, `governance`, `memory`) and **7** list verbs
   lowering into `ListArgs` (governance's one site serves adr/policy/standard).
   Neither is 6.
2. "**every existing `ListArgs` literal uses `..Default::default()`, so none
   break**" (D6): **false.** `into_list_args` (`main.rs:94`) is an **exhaustive**
   literal with no `..Default` — it *must* gain `columns` (it is the §5 wiring
   site, so the break is intended, but the universal as stated is wrong). The
   command-side test helper `clist()` (`main.rs:1485`) is likewise exhaustive
   (`CommonListArgs`) and will need `columns: None`. (boot.rs's two literals
   *do* use `..Default` — those are safe.)
3. "**6 call sites stay green unchanged**" (D6): the verb `list_rows` fns **do**
   change — each migrating verb gains `mut args` + the `let columns =
   args.columns.take();` line. What is genuinely unchanged is `build()`'s
   signature and its ~10 **leaf** tests (all use `..Default::default()` —
   `listing.rs:359+` — **confirmed true and load-bearing**).

**Risk:** Low — the protective core (build's leaf suite stays green) holds. But
imprecise blast-radius claims mislead the plan's verification scoping.

**Sentence — DESIGN CHANGE (cheap precision fix).** Reword D6/R4: state the build
*function + its leaf tests* are unchanged; the verb `list_rows` fns each gain
`mut args` + a `take`; `into_list_args` and `clist()` are the exhaustive literals
that move with §5; give the call-site count at verb granularity (7) or source
granularity (5), not 6.

---

## Non-charges (interrogated, found clean — recorded so the acquittal is on the record)

- **ADR-001 / A-3 (clap out of the leaf):** `--columns` is a free
  `Vec<String>` on the command-side `CommonListArgs`; validation lives in the
  leaf `select_columns(&[String])`; `Column<R>`/`fn(&R)->String` import no clap.
  The design explicitly rejects a per-kind `ValueEnum` (A-3). ✓
- **A-2 (one uniform error):** `select_columns`' unknown-column error mirrors
  `validate_statuses`. ✓
- **D7 (faithful JSON):** untouched by `--columns` (D2). ✓
- **Pure/imperative split:** extractors and `render_columns` are pure; no clock /
  rng / git / disk. ✓
- **slice & backlog extractors:** non-capturing under D5 — slice's prefix is the
  `SLICE_KIND.prefix` const and its markers (`decorated_status`, `phases_cell`)
  destructure the existing tuple `R`; backlog's `kind` lives in `BacklogItem`. ✓
- **`select_columns` default ⊆ available** is a dev-discipline invariant, not
  compile-enforced — but each kind's "default omits slug" unit (§6) exercises the
  default path and would catch a mismatched curated default. Accept.

---

## Questions for the User

1. **CHARGE II:** close IMP-013 with an explicit JSON-row descope note, or keep a
   thin follow-up open for the JSON-row convergence? (Recommendation: descope note
   in the design + a one-line resolution at `/close`; no new backlog item unless
   the json_rows duplication later bites.)
2. **CHARGE III:** confirm the flip from silent no-op → **loud rejection** of
   `--columns` on `memory list` until IMP-017. (Recommendation: yes.)

---

## Pronounce Judgement

**The design is sound in its bones but tainted at the edges — it shall not pass to
plan unshriven.** Its spine choices are doctrinally clean (ADR-001, A-2, A-3, D7,
the pure split all hold). But one **major** heresy festers — CHARGE I, where
spec's stated row type cannot honour the D5 non-capturing vow, the very IMP-013
config-surface fracture the user ordered hunted. Three **moderate** taints follow:
a backlog falsehood (II), a silent footgun flag (III), and unverified claims (IV).
A **minor** taint of imprecise arithmetic rounds out the indictment (V).

None demands burning the design to the ground. All five are integrable with small,
known remedies — four of them the design already half-knows (governance's
pre-materialised id, the per-kind `validate_statuses` seam, the
assert-every-surface mem). **Re-enter `/design`, integrate I–V, and present for
re-approval before `/plan`.**

## Sentencing — ordered penance

1. **CHARGE I** — rewrite §4 spec to a pre-materialised display row (governance
   pattern); confirm all four kinds' extractors are non-capturing. *Verify:* spec
   unit + spec golden (prefixed ids per subtype).
2. **CHARGE III** — D9/§5/R4: silent no-op → loud rejection on `memory list`.
   *Verify:* black-box test asserts the unsupported-`--columns` error.
3. **CHARGE IV** — extend §6 to pin (a) memory's rejection, (b) empty-list per
   verb, (c) spec multi-block + omitted-empty-block, (d) governance breadth.
4. **CHARGE II** — D2/§8/closure-intent: record the JSON-row descope; settle
   Question 1.
5. **CHARGE V** — correct the D6/R4 counts and the false-universal blast-radius
   claims.
6. Re-present the design for User re-approval. The gate holds until then.

*Penance for the scribe who let spec's subtype slip the row type: three days
barefoot before the `GovRow` that already showed the way, and the breaking-wheel
for any implementer who reaches for `Box<dyn Fn>` to paper over it.*

> **HERESIS URITOR; DOCTRINA MANET**
