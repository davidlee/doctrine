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
