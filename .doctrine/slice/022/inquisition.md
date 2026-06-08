# Inquisition — SL-022 Design

> **HERESIS URITOR; DOCTRINA MANET**

Convened upon `design.md` of SL-022 ("Technical-spec system support: descent,
decomposition & integrity"), tried against PRD-012, the acceptance criteria of
REQ-082 / REQ-083 / REQ-084 / REQ-087, ADR-004, ADR-001, the storage rule, and
the living flesh of `src/spec.rs` / `src/registry.rs`. The accused presented a
self-review and begged that finding **D** alone be put to the question. The
Inquisitor does not take the prisoner's word for which of his sins are mortal.
The witnesses — the source files themselves — were cross-examined, and they
confess to more than the design admits.

---

## 1. Charges

### CHARGE I — *Falsum testimonium de codice* — the design bears false witness about the living code (CRITICAL)

**Doctrine violated:** the §5.3 contract and the behaviour-preservation gate
(CLAUDE.md: "when changing shared machinery … the existing suites are the proof —
they must stay green unchanged").

**Evidence, confessed under cross-examination of the source:**
- `design.md` §5.3 proclaims: *"`build_registry` already loops `[Product, Tech]`
  … read the parsed `Spec.parent` / `Spec.descends_from` … **No new file reads —
  both fields ride `spec-NNN.toml`, already parsed.**"*
- The witness `src/spec.rs:705-743` (`build_registry`) reads **only**
  `members.toml` and `interactions.toml`. It **never** reads `spec-NNN.toml` and
  **never** constructs a `Spec`.
- The lone parse of the `Spec` struct in the whole subsystem lives at
  `src/spec.rs:675-677` — inside **`run_show`**, not `build_registry`.

So the spec TOML is **not** "already parsed" on the validate path. To harvest
`parent` / `descends_from`, `build_registry` must gain a **new, fallible
`read_to_string` + `toml::from_str::<Spec>` per spec** — across every product
**and** tech spec in the corpus (a dozen products already on disk, plus the tech
corpus to come).

**Risk:** grave and twofold.
1. The "additive set + new methods only, zero perturbation" thesis (R1) and the
   "Affected surface" ledger rest on a premise that is **false**. The true delta
   to `build_registry` is a per-spec disk read and parse, not a free ride on data
   already in hand.
2. That new parse **changes the error surface of the shared machinery.** Today a
   malformed `spec-NNN.toml` does *not* break `doctrine spec validate` — validate
   never opens it. After this slice it **will** (`build_registry` returns `Err`,
   `run_validate` bails). The behaviour-preservation analysis (§9) accounts only
   for hand-built `Registry` unit tests staying green — it is **silent** on this
   genuine change to `build_registry`'s observable failure behaviour. The unit
   suites stay green precisely *because* they bypass the seam that actually
   changed; their greenness is therefore **not proof** the machinery is unchanged.

**Sentencing:** correct §5.3 and the Affected-surface ledger to state plainly that
`build_registry` gains a per-spec spec-TOML read+parse. Extend the
behaviour-preservation analysis to the `build_registry` error surface. Add a
`build_registry`-level test over a temp corpus proving a malformed spec is now
surfaced. *Let the prisoner who claimed "no new reads" be set to count every read
he denied, by candlelight, until the corpus is exhausted.*

### CHARGE II — *Multiplicatio testium* — one cycle, many accusations (MAJOR; missed by the self-review)

**Doctrine violated:** the design's **own** law, laid down resolving finding A:
*"One finding per defect."*

**Evidence:** §5.2 `parent_cycle` — *"walk the child→parent map **from each node**
with a visited `BTreeSet`; a revisit … is a cycle."* The self-review (finding A)
laboured to ensure the 1-cycle A→A yields **exactly one** finding (`self_parent`
sole reporter; `parent_cycle` skips self-loops). But it never turned its eye on
the **k-cycle**. Walk A→B→A from A: revisit → finding. Walk the same ring from B:
revisit → finding. A 2-cycle confesses **twice**; a 3-cycle (the design's own
`cycle-3` test case) **thrice**. The principle established for the self-loop is
**broken** for every larger ring.

**Risk:** one architectural defect spews k findings — noise that buries the true
count of distinct violations and contradicts the design's stated invariant. The
Layer-A tests name "cycle-2 / cycle-3" but (on the design's face) assert
*existence*, not *multiplicity* — so the relapse is **untested** and latent.

**Sentencing:** make `parent_cycle` report each cycle **once** (canonicalise the
ring — e.g. emit only when the walk's least node-id is the start), or amend the
"one finding per defect" law to confess "one finding per node on a cycle" and
**test the count** for cycle-2 and cycle-3. Heresy half-burned is heresy banked.

### CHARGE III — *Confessio muta* — the silent confession of the second parent (MAJOR; this is finding D, tried in full)

**Doctrine violated:** REQ-087 AC1 — *"A containment that would give a spec a
second parent is **rejected as a hard finding**"* — and AC3 — *"An integrity pass
returns a non-zero result on any decomposition violation."* PRD-012 §6 binds the
cycle and the second parent in **one sentence**, both *"rejected as a hard
finding."*

**Evidence:** the accused (finding D) makes a second parent *unrepresentable* — a
duplicate `parent =` key is a TOML duplicate-key error, `parent = ["A","B"]` a
type error — both failing at parse, surfacing through `build_registry` → `Err` →
`run_validate` bail. The design pleads this is *"stronger than a finding."*

The Inquisitor concedes the **prevention** is stronger — invalid state made
unrepresentable is a virtue. But two heresies hide inside the plea, **unconfessed
by the self-review**:

1. **The failure is mute and undiagnosable.** Mirroring `run_show` (`src/spec.rs:677`),
   the parse fails with a generic `"Failed to parse <path>"`. The operator who
   wrote two parents receives **no mention** of decomposition, of `parent`, of any
   integrity rule — indistinguishable from a stray bracket or a typo'd quote. A
   *finding* **names the sin**; a parse error merely slams the door. Within one
   acceptance-criterion sentence the **cycle** half is honoured by a true,
   named, non-zero finding while the **second-parent** half is honoured by an
   opaque syntax fault. That asymmetry is the rot at the centre of D.
2. **AC3's non-zero exit, for this case, is verified only at the wrong altitude.**
   §9 Layer B tests *"duplicate `parent` key → read errors"* — i.e. it asserts
   `toml::from_str` errors. It does **not** drive `doctrine spec validate` (nor
   `build_registry`) over a corpus bearing a duplicate-parent spec to prove the
   **integrity pass** returns non-zero. The AC speaks of the *pass*; the test
   inspects the *parser*. A verification gap squats on the very criterion the
   accused already weakened.

**Risk:** a literal-reading deviation from a binding acceptance criterion, shipped
on the strength of a self-assessment, with the corroborating exit-code test
absent.

**Sentencing:** the mechanism may **stand** (unrepresentable beats detectable),
but on three conditions: (a) record an explicit **REQ-087 reconciliation note**
— a sibling to the design's Q4 for REQ-082 prose — declaring that AC1's "hard
finding" is satisfied, for the second-parent case, by *structural impossibility +
non-zero exit*, not by a findings-list entry, and obtain the User's assent that
this reading is sanctioned; (b) add a test that exercises `build_registry` /
`run_validate` over a real duplicate-parent spec and asserts the **non-zero
exit** (AC3), not merely a parser error; (c) acknowledge in the design that the
operator's failure message is generic, and judge whether a friendlier diagnostic
is owed — or explicitly decline it, in writing.

### CHARGE IV — *Machina sine mandato* — infrastructure built without a warrant (MINOR)

**Doctrine violated:** "build only what is missing" (slice scope); traceability of
every mechanism to a requirement.

**Evidence:** the **severity split** — a whole new `warnings()` sibling method and
a `warning:`-prefixed exit-semantics tier — is justified (D5, §3, the slice's
"one structural addition beyond additive checks") **solely** to serve the WARN on
a product spec carrying `descends_from`. Yet **no acceptance criterion** demands
it: REQ-082's ACs speak only of a *tech* spec naming its product capability;
there is **no AC** governing a product spec as the *subject* of descent. The warn
case — and the entire severity apparatus erected for it — descends from a
**self-imposed** requirement (Q1), not from doctrine.

D5 defends the warn over a hard error because a hard error *"pre-empts Q1"*. The
Inquisitor finds the rationale **thin**: a hard finding reading *"`descends_from`
is a tech-only field"* forecloses nothing about whether **product specs** gain
**their own** hierarchy someday — when that day comes the field is added to the
product family then. By the design's **own symmetry doctrine** (finding E:
wrong-kind is a hard invalid-kind finding for `parent` and `interaction`), the
consistent treatment of a tech-only field on the wrong family is a **hard
invalid-kind finding** — which needs **zero** new infrastructure.

**Risk:** a new exit-semantics tier (a permanent widening of `validate`'s
contract) erected to serve a single untraceable case that the design's own
consistency principle would handle for free.

**Sentencing:** justify the `warnings()` tier on its **own** merits as reusable
machinery the corpus will need regardless — *or* collapse the case into a hard
invalid-kind finding (symmetry with Charges' kin E) and delete the severity split
from this slice. Decide deliberately; do not smuggle infrastructure under a
WARN no requirement asked for.

### CHARGE V — *Verbum lapsum* — "divergence" where the truth is "the contract moved" (MINOR)

**Doctrine violated:** precision of language (CLAUDE.md: "Naming things well is
VERY important"); honest accounting of the behaviour-preservation gate.

**Evidence:** §9 and §3 call the rewrite of `non_tech_interaction_target_is_flagged_tech_only`
a *"sanctioned divergence / reframe"* of the behaviour-preservation gate. But the
witness `src/registry.rs:71-72,214-220` shows the **current** code deliberately
reports a `PRD-*` interaction target as **dangling** — and PRD-012 §6 now declares
that very case an **invalid target kind, not a dangling reference**. The old
behaviour is now **incorrect**. Rewriting that test is not a *divergence from*
behaviour-preservation — it is a **required behaviour change** mandated by the
spec, to which behaviour-preservation **never applied**. The gate guards
*unrelated* machinery from *accidental* change; an *intended* contract move is
outside its scope entirely.

**Risk:** mild, but the muddled framing invites a reviewer to treat a mandated
correction as a grudging exception, obscuring that REQ-084 is a deliberate
behaviour change with its own verification burden.

**Sentencing:** rename it in the design: REQ-084 is an **intended behaviour
change** (the dangling→invalid-kind contract moved per PRD-012 §6), not a
"divergence" from a gate that does not reach it. State the old behaviour, the new,
and that the rewritten test asserts the new contract.

---

## 2. Questions for the Accused

1. **REQ-087 AC1 (Charge III):** do you, the User, sanction the reading that "a
   second parent, rejected as a hard finding" is satisfied by *structural
   impossibility + non-zero exit*, rather than a named findings entry? Or must the
   second-parent case produce a true finding (forcing `parent` to a representable
   shape the design rightly resists)?
2. **Charge II:** for a k-node cycle, is **one finding per cycle** required, or is
   **one finding per node on a cycle** acceptable? Your answer fixes both the code
   and the test.
3. **Charge IV:** is the `warnings()` severity tier wanted as durable corpus
   machinery, or shall product-spec descent be a hard invalid-kind finding
   (symmetry, zero new surface)?
4. **Charge III(c):** is a friendlier diagnostic owed to the operator who writes
   two parents (today: an opaque `"Failed to parse"`), or is the generic parse
   error accepted as the cost of unrepresentability?
5. **`DescentEdge.on_product` (minor):** the bool baked onto the edge couples the
   edge shape to the warn check, where `descent_findings` could test the subject's
   membership in `product_specs` directly. Deliberate, or incidental?

---

## 3. Pronouncement of Judgement

**Heresy is present — but it is venial, not damning, and the corpus is salvageable
without the stake.** The design is doctrinally **sound** on the matters its author
chose to fear: ADR-004 outbound-only is honoured (no stored reciprocity, `show`
outbound-only, the §5 carve-out correctly deferred), ADR-001 layering holds
(pure leaf checks, impure scan in command), the storage rule is kept (the cycle
inversion is ephemeral), and finding **D** — the one the prisoner offered up — is,
in its **prevention**, defensible.

But the prisoner confessed the wrong sins. The two gravest taints he did **not**
name: **Charge I**, a false claim that the spec is "already parsed" on the
validate path, which hollows out the behaviour-preservation argument; and **Charge
II**, a k-cycle multiplicity that breaks the design's own "one finding per defect"
law the self-review thought it had satisfied. Finding **D** survives the question
(**Charge III**) but only chained to a reconciliation note and a missing exit-code
test. The severity tier (**Charge IV**) stands accused of building without a
warrant.

The design may **not** proceed to `/plan` unremediated.

---

## 4. Sentencing & Penance — *the ordered march to absolution*

1. **Charge I — recant the false witness.** Correct §5.3 and the Affected-surface
   ledger: `build_registry` gains a per-spec spec-TOML read+parse. Extend §9's
   behaviour-preservation analysis to the new `build_registry` error surface; add
   a `build_registry`-level test proving a malformed spec is now surfaced.
   *Verification:* the design no longer contains the phrase "already parsed"; a
   test exists. *Punishment for relapse:* the **strappado** — hoisted by the wrists
   to re-read every line of `build_registry` he misquoted.
2. **Charge II — silence the false witnesses.** Choose: dedup the k-cycle to one
   finding, or amend the law to "one per node on a cycle." Either way, **assert
   the finding count** in the cycle-2 and cycle-3 tests.
   *Verification:* a test asserts the exact count. *Punishment:* the **wheel**, one
   turn per redundant finding, until the multiplicity is broken.
3. **Charge III — name the mute sin.** (a) Add the REQ-087 reconciliation note and
   secure the User's assent; (b) add the `validate`/`build_registry` **exit-code**
   test for the duplicate-parent corpus (AC3); (c) record the generic-message
   decision.
   *Verification:* note present, exit-code test present, decision written.
   *Punishment:* the **scold's bridle**, until he learns a parse error is not a
   finding.
4. **Charge IV — show the warrant or raze the works.** Justify `warnings()` as
   durable machinery the corpus needs, or collapse product-descent to a hard
   invalid-kind finding and delete the severity split.
   *Verification:* §7/D5 states a requirement-traceable justification, or the
   split is gone. *Punishment:* his unwarranted scaffold **burned** before his
   eyes, that he learn to build only what is mandated.
5. **Charge V — speak truly.** Rename the REQ-084 rewrite an **intended behaviour
   change**, not a "divergence."
   *Verification:* §3/§9 wording corrected. *Punishment:* the **pillory**, a placard
   reading *VERBVM LAPSVM* about his neck for a single market day — the lightest
   penance, for the lightest sin.

Recant charges I–III before any plan is authored; II and IV before any code is
cut; V at the author's earliest shame.

> **HERESIS URITOR; DOCTRINA MANET**
