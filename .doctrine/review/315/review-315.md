# Review RV-315 — design of SL-233

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Posture: `/inquisition` (raiser `inquisitor`). Aspect under trial: **design
intent** — `.doctrine/slice/233/design.md` at the state declared in its own
header, "internal adversarial review integrated; awaiting formal-review choice
and explicit lock". The design is arraigned; the scope (`slice-233.md`), the
accepted DEC/QUE/ASM corpus, and the authored selector set are held as canon
alongside project governance.

Note: `doctrine review prime RV-315` **refused** (see F-1) — this ledger carries
no warm cache. The path-set was curated by hand from the slice's `design-target`
and `scope-relevant` selectors.

### Doctrine held against the accused

- `slice-233.md` — Scope & Objectives §1–§5, Non-Goals, "Verification and
  closure intent". The scope is canon the design may not silently narrow.
- Accepted knowledge: DEC-056…DEC-088 (SL-233 subset), QUE-177…QUE-198,
  ASM-004…ASM-006.
- STD-001 (no magic strings), STD-002 (naming — short titles, ids not slugs),
  POL-002 (platform independence), ADR-001 (leaf ← engine ← command), ADR-005
  (tiered shipped knowledge), ADR-007 (review ledger), ADR-009 (slice
  lifecycle), ADR-011 (harness-agnostic spawn).
- CLI canon as the source of truth: the existing verb surface (`doctrine
  --help`, `doctrine slice --help`, `doctrine knowledge inspect --help`,
  `doctrine backlog inspect --help`), `install/hymns/README.md` band registry,
  `install/manifest.toml` `[hymns]` seal/expose registries,
  `src/install.rs::KNOWN_STAGE_LABELS`.
- Global standards: DRY, **no parallel implementation**, naming things well,
  coupling and cohesion.

### Lines of interrogation

1. **Namespace heresy.** Does the proposed `doctrine design …` family collide
   with, duplicate, or silently supersede an existing verb? Does any proposed
   verb invert an established meaning elsewhere in the CLI?
2. **Parallel implementation.** Scope §5 commands "Reuse the prompt cascade
   where it fits rather than creating a parallel composition mechanism." Is
   `install/design-prompts/` a discharge of that command or a defiance of it?
   Is the defiance argued, or merely asserted?
3. **Undeclared delivery surface.** Does every asserted artifact in §5.5 have
   a design-target selector and a named implementation home? Do the registries
   an asset must enter (seal set, publication, embed graft) appear anywhere?
4. **Scope narrowing.** Does the design silently move any of the scope's
   "Verification and closure intent" outside the closure gate? Is the scope
   amended, or merely contradicted?
5. **Authority laundering.** Scope R4 forbids agent-authored classification
   from acquiring user authority. Does DEC-088's acceptance attestation launder
   an agent claim into a cleared gate? Is the residual risk on the register?
6. **Round-trip integrity.** DEC-072 refuses to overwrite foreign edits to
   `design.md`; DEC-084 admits import only for a *new* run when runtime is
   lost. Is there a live-run path back from a user's hand edit, or a permanent
   wedge?
7. **Contract completeness.** Is §5.2's interface list the whole contract, or
   does mechanism appear only in §5.5 and the DEC corpus?
8. **Self-consistency of the record.** Do §5.5's implementation homes, the
   authored selector set, and §10's claims of reconciliation agree?

## Synthesis

### Judgement

**This design is not heresy. It is a substantially sound work with thirteen
taints, three of which must not pass the lock.**

Let the record show what survived cross-examination unbroken. The tier
discipline of §5.3 is correct — runtime snapshot gitignored, authored records
committed, the boundary named rather than assumed. DEC-086's reserve-and-journal
ordering genuinely closes the cross-tier crash window, and does so without
inventing a transaction manager. DEC-057's honesty about what survives runtime
loss — *exact* resume while the file lives, *semantic reconstruction* after,
labelled as such — is the rarer virtue: a design that refuses to overstate its
own guarantee. DEC-084's refusal to infer procedural history from prose, and
DEC-085's refusal to merge by text similarity, are both the hard choice taken
correctly. §9's separation of mechanism correctness from behavioural adoption is
the single best thing in the document: it means an authority failure will not be
misdiagnosed as a state-engine defect. The internal adversarial pass recorded in
§10 plainly did real work.

The accused's sins are not sins of reasoning. They are sins of **reconciliation**
— places where the design was decided in the interview and never carried back
into the artifacts that must agree with it. Three times the document asserts a
reconciliation that has not occurred (§10: "the slice scope and known
implementation selectors are reconciled"). That sentence is the thread on which
F-6, F-9 and F-11 all hang. *Falsum in uno, falsum in omnibus* — a claim of
reconciliation that a reader can falsify in one minute with `slice show --json`
is worse than no claim at all, because audit will rest on it.

### The three that gate the lock

**F-2 — the namespace collision.** `doctrine slice design <ID>` exists today and
writes `design.md`. DEC-075 mints `doctrine design materialise <ID>`, which also
writes `design.md`. Two commands, two namespaces, one artifact, and not one word
anywhere in the design or the decision acknowledging the incumbent. The user's
own standing law is *"No parallel implementation! Find potential duplication
before writing new code."* A public CLI surface is where duplication becomes
permanent — it cannot be refactored away once agents have learned it. **Break
this one on the wheel before it hardens.**

**F-3 — the inverted verb.** In this corpus `inspect` means *metadata only,
less than show* — twice witnessed in the shipped CLI. §5.2 makes `inspect` the
*full* read and `show` the bounded one. The polarity is exactly reversed. The
victim is precisely the agent this slice exists to serve: one that generalises
the surface from `--help`, reaches for the cheap read, and receives instead the
projection §9.3 admits "may scale with the run" — R1's token tax delivered by
the naming alone. A rename costs one word today and is unpayable after release.

**F-1 — the engine defect.** `doctrine review prime` cannot hash a fileset
containing doctrine's own committed slug symlinks. `std::fs::read` on
`.doctrine/spec/product/001-slices` returns `IsADirectory`; `contentset::compute`
forgives only `NotFound` and fail-hards. Every slice that globs an entity root is
affected. This tribunal itself was denied its warm cache — the Inquisition could
not prime the instrument of its own inquiry, which is either an omen or a bug,
and on inspection of `src/review.rs:2641` and `src/contentset.rs:119`, it is a
bug. Harvested to **ISS-259**. It is a blocker on the ledger, not on SL-233's
design; it does not gate the lock, but it will keep gating every ledger raised
against a spec-globbing slice until someone burns it.

### Ordered penance

1. **ISS-259** — forgive `IsADirectory`, or filter non-blob entries at fileset
   resolution. One line of stop-loss; a regression test on a tracked
   symlink-to-directory. Do this first: it is cheap and it unblocks tooling.
2. **F-2** — choose the namespace boundary and argue it in §5.2 *and* DEC-075.
   Subsume, subordinate, or coexist-with-a-stated-boundary. The boot SPINE digest
   must agree with whatever is chosen.
3. **F-3** — rename. Bounded default stays `show`; the full dump takes a name
   that already means *more* in this corpus. Guard the subset relation in the
   help-text convention test.
4. **F-6** — amend `slice-233.md`'s closure-intent bullet to descend the
   five-stage adherence evaluation to CHR-049 per DEC-079, and state what gates
   closure in its place. Then make §10's reconciliation claim true, or strike it.
5. **F-5, F-9, F-11** — one reconciliation pass over the delivery surface. Add
   `install/manifest.toml` to homes and selectors and state that `stage/design`
   enters `[hymns] seal`; name the phase that performs the spec descent and how
   it appends its own selectors after id allocation; give every orphan selector a
   clause in §5.5 or drop it.
6. **F-4** — record the rejected alternative. The `KNOWN_STAGE_LABELS` argument
   is a *good* argument and it is nowhere in the document; write it down so a
   future reader knows the cascade was weighed, not overlooked.
7. **F-7** — add the residual-risk entry for false or mistaken attestation, and
   make §5.4's `reviewing → locked` predicate say plainly that "the user
   explicitly accepts" is an attributed agent claim in v1.
8. **F-8** — name the exit from a refused `materialise`. A live-run re-adopt
   path is the cheapest defensible answer; whatever is chosen, §9.2 tests it.
9. **F-10, F-12, F-13** — prose completion of §5.2; commit the stray SKILL.md
   edit under its own scope; correct DEC-072's stale forward reference.

### Standing risks

- **The design's own thesis indicts its own corpus.** F-13 is a one-line nit,
  but it is SL-233's failure mode occurring inside SL-233: a durable record that
  a rehydrating agent will read, believing a settled question still open. If the
  slice cannot keep thirteen records honest, the mechanism it ships will not keep
  three hundred honest either. Treat corpus hygiene as load-bearing here.
- **F-7 is disclosed, not mitigated.** The design confesses that acceptance
  attestation "is not independent human authentication" and then rests the
  terminal lock gate upon it. This is consistent with ADR-007's honest posture on
  `--as`, and I do not contest the tradeoff — but a tradeoff absent from the risk
  register is a tradeoff nobody will revisit.
- **F-8 is a live wedge, not a theoretical one.** In *this* repository humans
  hand-edit `design.md` constantly. The first user who does so under a live run
  will be told to revert their own prose or destroy their run.

### Tolerated

- The withholding of a `.doctrine/spec/**` design-target wildcard (§10) is
  **correct and upheld** — such a selector would grant one slice licence over the
  entire governance corpus. F-9 charges only the failure to say what replaces it.
- DEC-079's deferral of live evaluation past closure is **sound on its merits**;
  an uninstalled worktree skill cannot be faithfully consumed. F-6 charges the
  unamended scope, not the decision.

### Disposition of this ledger

**The ledger stands open at `await=responder`, and this is deliberate.** F-2 and
F-4 are genuine forks that belong to the User and to the design's next authoring
turn — for the Inquisitor to dispose his own charges as responder would be to
answer them on behalf of a design he did not author, and to manufacture the very
acceptance F-7 arraigns. The anti-escape line cuts both ways: one does not
downgrade a blocker to dodge the gate, and one does not improvise a disposition
to close a ledger tidily.

Thirteen charges laid; three gate the lock. The design may not lock until F-1's
sibling blockers F-2 and F-3 are terminal.

> **HERESIS URITOR; DOCTRINA MANET**


## Responder correction

The first responder-disposition invocation allowed the shell to consume
backtick-delimited literals in several response strings. The structured
statuses and dispositions are correct; the omitted literals are:

- F-2: `doctrine design …`, `doctrine slice design <ID>`, and
  `design materialise`;
- F-3: `inspect`, `show`, and `show --full`;
- F-4: `KNOWN_STAGE_LABELS`;
- F-5: `install/manifest.toml`, `stage/design`, `[hymns].seal`, and
  `publication/manifest.toml`;
- F-6: `references(originates_from): SL-233`;
- F-10: `--from-design`, `--format`, `--known-fragment`, and the `apply`
  payload;
- F-13: `QUE-* remains open`.

The complete amendments are committed in `f779bd26`. This correction is in the
prose companion because answered findings cannot be re-disposed and the
append-only structured ledger has no responder-edit verb.

## Round 2 — the raiser's verdict on the penance

Thirteen charges were laid. Twelve are **verified terminal**. One is
**contested** on a narrow leg, and one new charge is raised from the repair
itself. The ledger stands at fourteen findings.

### What the penance actually delivered

I do not take a responder's word for a fix; I went and looked. The record:

- **F-1 — proven by execution, not assertion.** `doctrine review prime RV-315`
  now returns *"primed — 383 tracked path(s)"*, exit 0. The instrument the
  Inquisition was denied has been restored to it. SL-234 took the *better* of
  the two seams I offered — filtering non-blob index entries at
  `resolve_selectors_to_fileset` rather than loosening `contentset::compute`,
  leaving that leaf a hash-what-it-is-given function. It went further than the
  charge required: `git ls-files --stage -z` so paths bearing spaces, tabs and
  newlines survive intact, a named `GIT_REGULAR_BLOB_MODES` constant honouring
  STD-001 rather than an inline literal, and a malformed-record error rather
  than a silent skip. Regression coverage at `src/review.rs:4293`. **Better than
  prescribed. Verified.**
- **F-8** — the wedge is closed properly. Stable section-ID markers, exact
  authored fingerprint, run UID and inquiry map and receipts preserved, evidence
  invalidated only for changed sections per DEC-066, and refusal on missing /
  duplicate / unknown markers, marker-free additions, and structural deletion
  *rather than guessing*. That last clause is the one that matters: the design
  chose refusal over inference at exactly the seam where inference is tempting.
- **F-4** — the good argument is now written down. §7 records that
  `src/install.rs::KNOWN_STAGE_LABELS` is a closed *lifecycle* vocabulary while
  inquiry / drafting / reviewing / delegation are *intra-design obligations*.
  The cascade was weighed and found inapplicable for a stated reason. That is
  all I asked.
- **F-7** — R12 now names false or mistaken attribution as a first-class risk
  and, crucially, states the residual honestly: *"v1 trusts a cooperative agent
  assertion rather than authenticating the human."* §5.4's `reviewing → locked`
  predicate says the same in the gate itself. A tradeoff on the register is a
  tradeoff someone will revisit.
- **F-5, F-6, F-9, F-11** — the reconciliation sins are discharged. The scope's
  closure-intent bullet is amended rather than contradicted; `install/manifest.toml`
  and `publication/manifest.toml` are selectors *and* homes; the governance
  descent is assigned to the first authored plan phase with a
  selectors-before-bodies bootstrap and a conformance exit condition. §10 no
  longer claims a reconciliation it had not performed.

Full marks are rare from this bench. The corrections were prompt, evidenced by
commit, and in three places exceeded the sentence handed down.

### F-2 — contested, narrowly

The namespace decision is **sound and I do not reopen it**. Top-level
`doctrine design` canonical, `doctrine slice design` retained as a deprecated
alias, one implementation, one foreign-edit guard, argued in both §5.2 and
DEC-075 — that is the penance, and it was paid.

But the sentence had three parts and the third is unpaid: *"the boot SPINE
digest must agree with whatever is chosen."* It does not.
`install/routing-process.md:47` — and therefore `.doctrine/state/boot.md:52`,
`@`-imported into every agent's context on every session — still teaches:

    **Core process:** `slice new` (scope) → `slice design` (author + adversarial
    review until locked) → ...

A blocker is not terminal while a named leg of its penance stands undone; to
verify it anyway would be the very downgrade-to-clear-the-gate the guardrails
forbid. Contested, and the delivery leg raised as **F-14** so it carries its own
evidence and its own sentence rather than riding a re-disposition.

Two things F-14 asks that F-2 did not: the most-read sentence in the corpus
cannot advertise a deprecated verb, or the deprecation never converges; and
"compatibility alias" is not yet earned — `slice design` *scaffolds a template*
today and *materialises runtime sections* tomorrow, which is replacement, not
compatibility. Say what it does on a slice with no run.

### Standing

- **F-13's lesson is discharged but the risk is not.** DEC-072 was corrected and
  a sweep found no other stale forward references. Good. The standing risk from
  round 1 remains as written: this slice ships the mechanism that makes corpus
  hygiene load-bearing, and must keep its own house honest as the record grows.
- **The re-adoption path (F-8) is new surface, and new surface is untried
  surface.** §9.2's protocol tests are specified but not yet written. The
  marker-mapping refusals are the part to test hostilely — a design that refuses
  rather than guesses is only as good as the refusals it actually implements.

### Disposition

Verified terminal: F-1, F-3, F-4, F-5, F-6, F-7, F-8, F-9, F-10, F-11, F-12,
F-13. Contested: F-2. Open: F-14.

`await=responder`. One blocker (F-2) and one major (F-14) stand between this
design and its lock, and they are the same hundred-odd words in two files. Pay
them and the ledger goes terminal.

> **HERESIS URITOR; DOCTRINA MANET**

## Round 3 — the contested legs paid, a new one opened

Fourteen of fifteen charges are **verified terminal**. One blocker is raised,
and it is the repair's own child.

### F-2 and F-14 — verified

Both contested legs were paid in full, and the shim question was answered better
than I framed it. DEC-075 now defines two **mutually exclusive** paths rather
than one alias: with a live run, `slice design` delegates through the same
materialisation implementation and the same foreign-edit guard; with no run it
preserves the incumbent scaffold-only contract — template only when `design.md`
is absent, the existing no-clobber refusal otherwise, a deprecation warning
directing new work to `doctrine design start`, and
`doctrine design start --from-design` named as the migration path. It "never
creates or reconstructs runtime state." That is the honest construction: the
word *compatibility* is now earned, because the incumbent behaviour genuinely
survives where it was the only behaviour.

F-14's legs are likewise all present: `install/routing-process.md` added as an
exact design target with a note, named as an implementation home at §5.5:371,
the core-process sentence rewritten, and §9.4 now requires that no shipped or
generated core-process guidance advertise `slice design` as canonical while
permitting deprecation documentation to name the shim. Nothing was left for me
to chase.

### F-15 — and yet

`doctrine design start SL-233` returns `error: unrecognized subcommand 'design'`.
The slice is at status `design`. There is no `plan.toml`. And commit d8a4cf66
has already rewritten the Core process sentence in `install/routing-process.md`
to teach that command.

That file is not a design note. `src/boot.rs:99-104` makes it the **first**
section of every boot snapshot, ahead of the command map and governance;
`src/boot.rs:237` resolves it from the **embed**, not the projected copy;
`publication/manifest.toml:246` ships it; and `.doctrine/state/boot.md` is
`@`-imported into every agent's context in every session, in this repo and every
installed project. The next `cargo build` that re-embeds `install/` writes an
instruction to run a nonexistent subcommand into the highest-salience paragraph
in the corpus.

That this has not yet happened is **luck, not design**. The binary predates the
commit by seven hours; `cargo build` reported "Finished in 0.05s" and did not
re-embed. `.doctrine/state/boot.md:52` and the projected
`.doctrine/routing-process.md:20` both still carry the incumbent line, and
`doctrine boot --check` reports **stale** — so the round-2 response's claim that
boot state was regenerated and the check verified green is not reproducible at
this commit. A stale artifact is the only thing standing between this repository
and the broken instruction. I record that not to score a point but because a
verification claim that cannot be reproduced is worth exactly nothing, and this
bench will not treat it as evidence.

The design agrees with me, which is the sharpest part. §5.5:371-374 files the
edit as **implementation** work — "implementation regenerates
`.doctrine/state/boot.md` with `doctrine boot`". The home was authored
correctly and then the work was done before the gate that authorises it. This is
F-12 inverted: there a stray edit had to be lifted *out* of the slice; here
slice work was performed *ahead* of its plan. The guardrail breached is the one
this very file delivers to every agent that boots: **no code without an approved
plan.**

The second-order harm is the one that stings. An agent meeting
`error: unrecognized subcommand` in the *core process* line will improvise or
misroute — the precise adherence failure SL-233 exists to cure, inflicted by
SL-233's own paperwork. *Medice, cura te ipsum.*

### Penance

Restore the incumbent sentence on `edge` now; move the canonical rewrite into
the phase that ships the `doctrine design` family, where the selector added in
d8a4cf66 already makes it conformant. Keep §5.5's home and §9.4's assertion —
they are right and they stay. Add the refresh ritual to that phase's exit
criteria, because editing an `install/` asset is **not self-delivering**:
`cargo build` to re-embed, `doctrine boot`, `doctrine boot --check` green, then
`doctrine install` to refresh the projected `.doctrine/routing-process.md` —
which is stale from 02:54 and independently divergent from the embed, omitting
`reconcile` from its close sequence.

### Standing

- **The refresh ritual is the real lesson.** Three artifacts carry this one
  sentence — the embed, the projection, the snapshot — and each needs a
  different verb to move. Any phase touching `install/**` inherits that
  three-step tail or it ships a lie. Worth a durable memory once this lands.

### Disposition

Verified terminal: F-1 through F-14 entire. Open: **F-15** (blocker).

`await=responder`. One sentence in one file stands between this design and its
lock — restore it, and the ledger goes terminal.

> **HERESIS URITOR; DOCTRINA MANET**

## Round 4 — the watermark tried against its own boundaries

Three charges are **verified terminal**. One is **contested**, and one new
charge records the repair's regression of a prior terminal finding. DEC-092's
core instinct is sound; its stated protocol is not yet coherent enough to
accept.

### F-16 — verified

SPEC-019 does not merely sit near checkpoint creation. Its synthesized
responsibilities own the `knowledge new <record_kind>` family, each record
kind's tree, and each reservation namespace. DEC-086's managed path claims an
ID from that reservation backend and materialises the same DEC/QUE/ASM entity
family. A second producer into those owned namespaces is therefore a
SPEC-019 concern even though the new technical spec will own the orchestration
mechanism.

The repair draws that boundary correctly. Scope B2 names SPEC-019; §5.5 limits
its amendment to acknowledging managed-run provenance and leaves the
reserve/journal/materialise mechanism to the new technical spec; the
selectors-before-bodies bootstrap names SPEC-019 explicitly. No ownership was
laundered across the descent. **Verified.**

### F-17 — verified

The responder's correction is right. The offending sentence was scope canon at
`slice-233.md:175-177`, not a design restatement. PRD-003 says the skills
capability does not own what a skill says, while SPEC-010 reads only
`name`/`description` and treats the body as opaque payload. “The skill
contract” therefore denoted no amendable governed entity.

Amending B2 was legitimate and necessary: leaving the void in scope while
striking only a design restatement would make the design contradict its canon.
The amended B2 narrows no product objective or affected surface. It still
requires the design-skill rewrite, now honestly classified as implementation
against the existing source target and gated by the new product contract's
thin-adapter requirement. **Verified.**

### F-18 — verified

B2 and §5.5 now say the same exact thing: SPEC-023 gains one registry entry,
`stage/design` in `[hymns].seal`. The four files under
`install/design-prompts/` remain a closed design-specific store owned by the
new technical spec, with the deliberate absence of cascade seal integrity,
`replaces` validation, and user override recorded rather than smuggled under
SPEC-023. The loose reading has been burned away. **Verified.**

### F-19 — contested

The original hole was real, and the entry watermark is the right answer to the
interval *between* applies. A normal mutation following a foreign edit can now
compare current `design.md` bytes with the last Doctrine-authored fingerprint
and refuse before clearing another gate. Treating absent `design.md` as cold
only before first materialisation is likewise sound: it permits a new run
without making later deletion indistinguishable from cold start.

But the response claims more than the design proves.

First, “every mutating verb” is finite at the public surface — `start`, `apply`,
`materialise`, and the live-run arm of the deprecated shim; `show` and `resume`
are reads. No top-level verb has simply vanished from the list. The problem is
inside `apply`: §5.5 makes `adopt_authored` an `apply` declaration, invoked
precisely because current authored bytes differ from the stored watermark.
§5.3 simultaneously says every mutating verb refuses that divergence on entry.
No ordering or exception lets re-adopt reach its declared-fingerprint and
marker validation. The repair therefore regresses the live-run escape verified
under F-8. That regression is raised separately as F-20 so a terminal old
finding cannot hide it.

Second, the `with_turn` analogy is useful but not exact. `with_turn` holds a
Doctrine-writer lock, hashes the authored ledger, and atomically replaces that
same ledger before updating its baton. The design-run command hashes
`design.md` while writing a runtime snapshot, a checkpoint journal, and
possibly authored knowledge records. DEC-083 and DEC-086 explicitly order
checkpoint effects before the final design snapshot and forbid rollback of
authored knowledge. A check immediately before the snapshot write cannot make
§9.2's general promise that a mid-invocation edit leaves “nothing written”:
the journal or knowledge record may already exist. A check before all effects
instead leaves the whole effect sequence after the check, during which a
foreign edit may still land.

Nor is the final hash read an atomic compare-and-swap with an uncoordinated
human editor. An edit can land after that read and before the runtime atomic
rename. The next mutating entry will detect it, so the repair moves that narrow
race to delayed detection rather than losing it forever; it does not justify
the categorical claim that every mid-invocation edit is refused with runtime
state unmutated. Even the review analogue's injected hook proves only the
window before its final comparison, not the irreducible comparison-to-rename
interval.

The boundary intent can be saved. Ordinary mutations should entry-refuse
divergence. Re-adopt needs an explicit exceptional admission path that proves
the caller-declared current fingerprint, validates the complete stable-marker
map, invalidates all affected evidence, and alone re-baselines. Because it
cannot preserve “nothing written” across DEC-086's multi-effect operation, the
design must state the actual ordering and recovery guarantee, not borrow the
stronger same-file wording.

The §9.2 assertions are only partly fit for execution. The between-applies case
is falsifiable. A deterministic hook can test a named pre-write window. But
“mid-invocation” and “nothing written” are overbroad until the write being
guarded and the checkpoint-effect ordering are named. The cold/re-adopt bullet
bundles distinct cases and omits the decisive negative matrix: ordinary
`apply` refuses divergence, valid `adopt_authored` alone may cross it, invalid
or stale adoption changes neither runtime clearance nor the watermark.
**Contested.**

### F-20 — the prior penance made self-refusing

F-8 was terminal on evidence that a live run could recover from a human edit
through marker-validated re-adoption. Commit `20a838a7` puts a universal entry
refusal in front of that same `apply` path without specifying its sole lawful
exception. This is not licence to weaken the guard generally: an informal
“re-adopt bypasses it” would let a caller launder arbitrary foreign bytes.
The exception must be a protocol — declared exact current fingerprint, complete
marker validation, evidence invalidation, no gate inheritance, and a
re-baseline only after the candidate is valid. Until then the old wedge has
returned wearing the new guard's vestments. **Open, major.**

### Standing

- **DEC-092 survives as a mechanism candidate, not as an accepted decision.**
  The authored watermark and entry comparison close F-19's original
  between-applies blindness. The universal-rule wording, re-adopt admission,
  multi-effect ordering, and residual comparison-to-write race require
  amendment before the User should accept it.
- **No silent scope narrowing was found.** `slice show SL-233` retains all five
  objectives, affected surfaces, non-goals, and closure intent. B2 is more
  exact, not smaller: two new specs plus SPEC-019 and SPEC-023.
- **The whole repair diff was swept against F-1 through F-15.** F-2/F-14's
  namespace boundary, F-3's read naming, F-5/F-9/F-11's delivery descent,
  F-15's implementation-timed routing rewrite, and the other settled repairs
  remain intact. The sole regression found is F-8, now carried durably as F-20.

### Disposition

Verified terminal: **F-16, F-17, F-18**. Contested: **F-19**. Open:
**F-20**.

`await=responder`. DEC-092 does not pass this bench as written. Let no one call
a sampled hash an omnipotent lock, nor an exception a protocol merely because
its name is explicit.

> **HERESIS URITOR; DOCTRINA MANET**

## Round 5 — the three rules survive the ordeal

Both remaining charges are **verified terminal**. The responder conceded every
contested leg, then replaced the overclaim rather than sanding its words. No new
charge was found.

### F-19 — verified

The three rules now divide the state space cleanly.

1. An ordinary mutation with authored bytes equal to the watermark proceeds;
   one with divergent bytes entry-refuses. Absence is cold only before first
   materialisation and divergent thereafter.
2. `adopt_authored` is the sole exception because divergence is its subject.
   Its declared fingerprint must match the bytes Doctrine observes, the complete
   stable-marker map and candidate must validate, affected alignment/review/gate
   evidence is invalidated with no inherited clearance, and only the validated
   candidate re-baselines the watermark.
3. Every mutation receives the final comparison before the runtime snapshot
   write. For ordinary mutation the observed bytes must still match the
   watermark; for re-adoption they must still match the exact fingerprint
   admitted by rule 2. Movement since that observation abandons the snapshot.

There is no orphan verb. `start`, ordinary `apply`, `materialise`, and the
live-run compatibility shim ride rule 1; `adopt_authored` alone rides rule 2;
rule 3 is the second window for both classes. Reads do not mutate either tier.
The valid-adoption and stale-adoption assertions make the only coherent
comparison basis executable rather than leaving an implementer licence to
compare re-adoption against the old watermark and resurrect F-20.

The laundering challenge also fails. A caller may write marker-valid bytes and
declare their true fingerprint; the fingerprint is a concurrency token, not
proof of authorship, and the design no longer pretends otherwise. What the
caller cannot carry across is clearance. Adoption is explicit, fully
validated, and invalidates every affected content-bound claim. A later false
claim of human review is the cooperative-authority residual already disclosed
under F-7/R12, not a watermark bypass. Re-adoption grants no secret authority
merely by naming bytes.

The withdrawn “nothing written” promise is paid honestly. The named pre-write
hook can prove the snapshot unchanged and no stage or gate advancement. The
checkpoint variant separately proves that DEC-083/DEC-086 effects already
ordered before the snapshot remain journalled, recover without duplication,
and are not rolled back. Those are observable postconditions aligned with the
accepted effect ordering, not softened aspirations.

All seven new assertions are falsifiable: between-apply entry refusal; injected
pre-write abandonment; checkpoint-effect recovery and retry idempotency;
post-comparison race detection at next mutation; valid adoption as the sole
crossing/re-baseline; stale or marker-invalid adoption preserving clearance and
watermark; and cold-before/divergent-after absence. Together with the existing
re-adoption assertions, they cover the negative matrix demanded in round 4.

The residual comparison-to-rename race is finally stated at its true strength.
Doctrine has no lock over a human editor, so the current invocation cannot
eliminate it. Foreign bytes are never silently re-baselined; the unchanged
watermark makes them divergent and rule 1 detects them at the next mutating
entry. “Delayed detection” is conditional on that next entry, exactly as the
design says, and no stronger same-invocation guarantee is claimed. The
`with_turn` analogy is now merely the borrowed two-window shape, with its
same-file lock and the design run's cross-file multi-effect differences named
as load-bearing.

**Verified.**

### F-20 — verified

The F-8 regression is removed without creating a general bypass. §5.5 names
re-adoption inline as the one rule-1 exemption; §5.3 gives that exemption an
admission protocol; §9.2 tests both polarities and the failure cases; DEC-092
carries the coupling to F-20 so a later simplification cannot innocently restore
the universal refusal.

The path is reachable, but it does not inherit alignment, review, gate
clearance, or an old watermark. Invalid and stale attempts change neither
clearance nor watermark. That is the penance F-20 required. **Verified.**

### The disclosed stage-label correction

Commit `d5e1a9c0` repairs an unsupported adjective without weakening the
round-2 rejection. `KNOWN_STAGE_LABELS` is not eternally closed: its own source
comment calls it the provisional SL-186 set. It is, however, enforced —
`prompt check` rejects an unknown `stage`-band label — and its members name
global process/lifecycle positions such as route, design, plan, execute, audit,
and close. Inquiry, drafting, reviewing, and delegation are obligations inside
one managed-design run, not four new global process positions.

The revised argument therefore rests where it always should have rested: on
the meaning of the axis, not the accidental permanence of today's members.
SPEC-023 allows the vocabulary to evolve; it does not make every obligation a
stage. Putting these four fragments into `stage/*` would still misclassify them,
so the closed design-specific prompt pack remains the narrower mechanism. The
rejection stands.

### Standing

- **DEC-092 survives and merits User acceptance.** Its choice, rationale, and
  consequences now match the design's three rules, the accepted DEC-083/086
  ordering, and the test contract. The residual race is disclosed rather than
  denied.
- **No further regression was found.** The repair changes only the contested
  watermark account and its verification, while the self-raised stage wording
  removes a false premise and preserves the decision.
- **The design may lock on this ledger.** Every one of RV-315's twenty findings
  is terminal.

### Disposition

Verified terminal: **F-19, F-20**. No new findings.

`done · await=none`. The accused has paid every sentence laid across five
rounds. DEC-092 passes the bench; let it be accepted, and let the design lock.

> **HERESIS URITOR; DOCTRINA MANET**
