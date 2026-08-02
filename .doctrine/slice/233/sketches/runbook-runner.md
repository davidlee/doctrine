# Sketch: the obligation runbook runner

Authored artefact under `.doctrine/slice/233/` — durable and diffable, not
runtime scratch. It records the design conversation the owner ordered in place
of PHASE-08's six-question `EN-2` procedure ("design conversation first,
paperwork back-solved", 2026-07-30), and it is the design PHASE-16 implements.

It is **not** the `EN-2` sketch. `sketches/thin-adapter.md` still lands and still
takes a non-author RV; this is the mechanism that lets it be honest.

> **Revision 3** (2026-07-31), after a **second** external adversarial prose
> review — a fresh agent over revision 2, because revision 2 rewrote the parts
> revision 1 got wrong and those repairs had never been checked by anyone. Three
> structural findings stood: the runbook had **no selector**, the `Condition`
> vocabulary **cannot express** the gate it was given, and the verifier-results
> path **does not exist** in the form §4 described. All three land on one seam —
> see §2.5. §12 carries both disposition tables.
>
> Revision 2's history, for context: it folded nine findings from the first pass,
> of which the load-bearing two were that the gate clause was on the wrong
> function and that `customization` has no execution seam.

---

## 0 · Posture: this ships to be corrected

Owner's ruling, 2026-07-31, after the second review: **ship it, find out what got
shipped, fix it afterwards.** Two adversarial passes have now each found that the
previous pass's repairs rested on false claims about the code. That is evidence
the design-then-verify loop has poor yield here relative to its cost, and that
the faster oracle is a running mechanism.

The ruling is not "stop designing". It sorts the remaining decisions by
reversibility, and spends what is left of the design attention only on the ones
that harden into contract at the moment of shipping:

**Hardens on ship** — changing it later means migrating persisted records or
breaking published ids:

- the **canonical encoding** of the step-definition digest (§3);
- the **serialised shape of the discharge record**, which is snapshot state (§2.3);
- **shipped step ids**, which are API (§3);
- the **seventh act's wire key** (§11).

**Ship and learn** — cheap to get wrong, and the running mechanism answers them
better than another round of prose would: whether triage is one step or five,
which checks `doctrine verify` launches with, whether attestation-without-
verification is a problem in operation at all, and the envelope's rendering
detail.

This sorting is why `set` mode leaves the phase (§5) and why the discharge record
gains a version tag instead of a fourth outcome arm (§2.3).

## 1 · The finding this answers

The two D2 models (`sketches/workflow-as-prose.d2`, `sketches/surface-as-built.d2`,
`a6af8f30`) were drawn to compare the incumbent skill against the built surface.
They do not match closely: one state fully moved, one correctly merged, ~4
partial, ~4 dark. The pattern is sharper than the count —

> **Doctrine took the state and the invariants. It did not take the checklists
> or the craft.**

The register is the whole problem. Read the four shipped fragments back:
"a duplicate subject is refused"; "advancing to drafting needs every blocking
node dispositioned"; "a proposal payload carrying a stage change is refused".
Those are statements of *what the machine will reject*. `hymns/stage/design.md`
is the same, and correctly so — it is titled "Design stage invariants".

Apply the owner's stated test — instruction should arrive as **"do this now"**,
not "be on the lookout for a situation, in case of which…", and an invariant is
by definition not a trigger — and the conclusion is that **no part of the built
surface is in the "do this now" register at all.**

**Revision 3 states the gap mechanically rather than rhetorically.** Revision 2
argued from register, which is true but soft. The hard version is this: the
`Exploring → Inquiring` edge already exists (`gate::can_advance`,
`src/design_run/gate.rs:149-157`) and already carries a guard — `Condition::
GoverningContextRecorded` and `Condition::InitialConcernsRecorded`
(`:163-166`). **Both are `is_derived() == false`** (`:92-97`), which means the
agent clears the edge by *asserting* them.

So the edge exists, the guard exists, and the guard is two booleans a caller
declares. Nothing structurally sequences the explore ritual before the clarifying
questions. §1.1 records what those two conditions actually are, because it turns
out to matter.

### 1.1 · What the incumbent guard actually is

Established by inspection during the second review, and it changed the design:

- Its **entire specification is nine words**, at `design.md:268` — *"exploring →
  inquiring: governing context and initial concerns recorded"*. There is no other
  definition of it anywhere in the repo.
- **No shipped guidance names it.** A search of `install/`, `plugins/` and
  `.doctrine/spec/` returns nothing for either condition. Not in `inquiry.md`,
  not in a hymn, not in `SKILL.md`. Every occurrence outside `gate.rs` is in a
  **test** (`src/design_run/tests.rs:162`, plus four e2e files). No agent driving
  a run has ever been told the condition exists, let alone what satisfies it.
- **Its subject must be a draft section.** Evidence binds through
  `current_fingerprint`, which resolves only from `next.sections`
  (`src/design_run/run.rs:1283-1290`); anything else is `Refusal::UnknownNode`.
  Every test binds it to `sec-01` / `sec-1` / `sec-{index}`.

That last point is the giveaway. A claim that governing context is in view gets
bound to the fingerprint of an arbitrary draft section, and DEC-066 liveness then
expires it when that section's prose changes — an event with no relationship to
the fact being claimed. The binding does no semantic work.

It also produces an ordering inversion: `declare` has no stage guard, dispatching
on id kind alone (`run.rs:768-770`), so the way to leave `Exploring` today is to
**create a draft section first** and then bind a claim about your governance
reading to its bytes.

**Consequence for this design.** These two conditions are not a competing account
of the same facts the runbook records — they are an unimplemented placeholder for
them. There is therefore **no duplication to reconcile**, and no reason for the
runbook to derive them (§2.5). Both are filed as backlog rather than fixed here.

## 2 · What Rust owns

A generic ordered-obligation primitive. Rust owns the closed concept; the
contents are asset data. This is DEC-077's own split one level deeper — *"Rust
owns the closed obligation vocabulary and deterministic state semantics; the
operational prose behind stable obligation keys remains in Markdown"* — applied
to procedure rather than to prose.

1. **A runbook per forward edge of the stage machine.** Revision 2 said "per
   obligation, selected by the rule fragments already use (`Fragment::for_stage`)
   — one selection axis, not two". That is not implementable: see §2.1.

2. **One mode in PHASE-16 — `sequence`.** A cursor, one step at a time, no
   skipping. `set` mode is deferred to **IMP-373** (this first read *"to
   PHASE-08"*; reassigned 2026-08-01, RV-325 F-11); §5 records why.

3. **A discharge record per obligation** — new run state, **versioned**. Each
   record holds the step id, **the digest of the step's definition at the moment
   of discharge** (§3), the revision it was discharged at, and the outcome:
   attested / verified (with the verifier's exit and captured output) / skipped
   (with a required reason).

   The version tag is load-bearing and is the whole of §0's reasoning applied:
   the record is persisted snapshot state, so its shape hardens on ship. A fourth
   outcome arm — a *null result*, "swept this and found nothing applicable" — is
   genuinely wanted (§7.1) and is deliberately **not** added now, because the
   version tag is what makes adding it later cheap. Ship three arms and the
   ability to grow a fourth; do not ship a guess at the fourth.

4. **A seventh writer act.** `ApplyRequest::WRITER_ACTS` is `[WriterAct; 6]`
   (`src/design_run/submission.rs:600-641`), one presence predicate per act.
   The comment does **not** anticipate a seventh member — it names a seventh
   *branch* as the failure case the table exists to prevent. The extension path
   is nonetheless real and well guarded: a **second** enumeration in
   `tests/e2e_design_delegation.rs:386-398` asserts set-equality against the
   production array precisely so that a seventh act fails loudly there rather
   than quietly widening the class. Both tables move together, or the build tells
   you. *(Verified twice; accurate as stated.)*

5. **A clearance clause on `gate::advance` — as a third derived input, not as a
   new `Condition`.** Revision 2 got the function right and the mechanism wrong.
   See §2.5.

6. **Envelope rendering of the current obligation only** — the step at the cursor
   with its position. This composes with `resume --known-fragment`, the
   just-in-time lever already built: carrying one step's text is *cheaper* than
   today's whole fragment, not dearer.

### 2.1 · The selector: guards belong on edges

Revision 2 claimed the runbook is "the structured half of the fragment that
already exists", selected by `Fragment::for_stage`. **There is no such fragment.**

`Fragment` is a closed four-variant enum — `Inquiry`, `Drafting`, `Reviewing`,
`Delegation` (`src/design_run/prompt.rs:32-38`) — and `for_stage` collapses
`Exploring | Inquiring → Inquiry` (`:96-110`); `asset_key()` renders
`design-prompts/{name}.md`. There is no `exploring` fragment for a runbook to be
the structured half of. §1 *relies* on that collapse to argue exploring is the
sharpest gap, while §9 ships `exploring.toml` — revision 2 asserted both.
(`for_stage` is not even a total fragment selector: `Delegation` is deliberately
unreachable through it, being selected by delegation state per DEC-065.)

**The runbook is a transition guard, and guards belong on edges.** The code has
already made this move once: `boundary_conditions(from, to)` is an edge-keyed
guard table over the same four forward edges (`gate.rs:161-183`), and it added
zero states when it landed. A runbook per edge is a third column on a table that
already exists:

```rust
boundary_conditions(from, to) -> &'static [Condition]   // existing, static
boundary_runbook(from, to)    -> Option<RunbookKey>     // new, static; 4 rows, 1 populated
```

**No new states, and no new stages.** The thing that reads like a state expansion
is the cursor — five steps looking like five sub-states. It is not: the cursor is
run data a guard consults, not a node in the machine. The exact precedent is
`ReviewStanding` (`gate.rs:213-223`), four booleans accumulated from run state
and consulted by the `Reviewing → Locked` guard, which nobody calls states.

**The asset name survives unchanged.** `can_advance` is a total function on
non-terminal stages — each of the four has exactly one outbound forward edge — so
edge-keying and origin-state-keying are isomorphic here.
`install/design-prompts/exploring.toml` is exactly right as authored: the runbook
you discharge while standing at `Exploring`, guarding its outbound edge.

**It also picks up an open item for free.** §11's fourth item observed that
`Locked` yields no fragment yet state 8 still has work to instruct, and that "a
runbook keyed to the lock transition is the obvious home". That was the design
reaching for transition-keying already.

### 2.5 · The gate clause: a third derived input

Revision 2 said the clause "joins the existing `Condition` vocabulary — the
closed set of things that block an advance — which is a better home than a new
one". It named the right function and the wrong mechanism. `Condition` cannot
express this, for three independent reasons:

- **It is payload-free.** Ten variants in `Condition::ALL`, classified by
  exhaustive `is_derived` and `as_str` matches (`gate.rs:22-121`). There is
  nowhere for a step identity to ride.
- **`satisfies` is existential.** `DerivedDesignFacts::satisfies` is
  `live_evidence().any(|e| e.condition() == condition)`
  (`src/design_run/facts.rs:96-99`). One live discharge would satisfy a
  `RunbookDischarged`; the gate needs *every* required step — a universal
  quantifier over a set whose arity comes from an asset.
- **The refusal has no room.** `Refusal::GateNotCleared` carries
  `Vec<Condition>`, so it cannot name an undischarged step — which `EX-8`
  requires it to do.

Underneath all three is one fact: **every guard in the machine today is a static
conjunction**, its arity fixed at compile time by `boundary_conditions`'
`&'static [Condition]`. A runbook guard is a **dynamic** conjunction whose
members come from an asset and carry identity. That is a genuinely new kind of
guard, and it should be visible in the types rather than smuggled into a closed
vocabulary that was built to exclude exactly this.

So:

```rust
advance(from, to, facts, review_standing, runbook_standing) -> Result<Stage, Refusal>
```

with `RunbookStanding { outstanding: Vec<StepRef> }` derived shell-side — load the
edge's runbook, digest each step, read the discharge records, hand in what is
outstanding — and one new refusal variant naming the steps. One static table, one
derived struct, one refusal. **No new `Condition` variants, no `Stage` variants,
no `Fragment` variants.**

*(If a variant were added anyway: `const _: () = assert!(widest_condition(
&Condition::ALL) <= DESIGN_ID_BYTES)` at `gate.rs:141` caps its kebab name at 32
bytes. A build stop, not a lint. Recorded so nobody rediscovers it.)*

**The runbook does not derive the incumbent conditions.** An earlier draft of
this revision proposed that discharging `explore.canon` and `explore.memory`
should make `governing-context-recorded` *derived* rather than claimed — the
`is_derived` doc comment invites it, calling the current asymmetry "deliberate
rather than finished". It is withdrawn, on two grounds:

1. **It is a type error across an ownership boundary.** Truth cannot flow from
   user-customisable material into a Doctrine-owned closed vocabulary. Once the
   override seam lands (§8), a project could delete `explore.canon` or flip it
   `required = false` and the built-in condition would derive true with no sweep
   — the name still promising what it no longer delivers. The only repairs are
   mixed ownership at step granularity (a framework-owned subset inside an
   otherwise customisable file) or an OR-fallback that makes the derivation
   decorative in the worst case. Both are designs against a seam that does not
   exist yet.
2. **There is nothing to reconcile anyway.** Per §1.1 the incumbent conditions
   are a stub, not a competing account. The overlap that motivated derivation
   was an artefact of reading the condition's *name* as its semantics.

The two conditions therefore stay exactly as they are — claimed, content-free,
untouched. No `is_derived` flip, no `ReviewStanding` surgery, no change to
`cleared_through` (`src/design_run/tests.rs:37-53`), and the behaviour-
preservation gate on shared machinery stays intact.

### 2.6 · Cumulative or per-edge

`cumulative_conditions(to)` re-requires every edge up to the target
(`gate.rs:187-199`) — DEC-067's re-derivation, which exists so that a run cannot
regress and return forward carrying clearance it no longer earns.

Applied to a runbook this would mean a prose edit to `explore.canon`, made after
the run reached `Drafting`, blocks `Drafting → Reviewing` — a stage two
boundaries away.

**Owner's ruling, 2026-07-31: it does not block; it warns.** The runbook
requirement is evaluated on its own edge. This preserves DEC-067's actual
property by a different route: regressing to `Exploring` puts the run back at
that edge's origin, where it faces the guard again. Cumulative re-derivation is
right for the existing conditions because they are global facts that rot
independently of where the run stands; a runbook is an entry ritual for one
transition.

It is nonetheless the first non-cumulative condition in the machine, which is why
it is written down here and stated in `EX-16` rather than left to whoever
implements it first.

## 3 · The asset, and what a step id does and does not do

TOML, one sibling per guarded edge, beside the fragment it structures. Structured
data in TOML and prose in MD is this project's own two-tier convention.

```toml
# design-prompts/exploring.toml — the runbook guarding the edge out of Exploring.
mode = "sequence"

[[step]]
id       = "explore.scope"
text     = "Read the slice scope, the specs and ADRs it descends from, and prior art."
required = true

[[step]]
id       = "explore.research"
text     = "Ensure the pre-design research round has run for this slice."
verify   = ["doctrine", "verify", "research-current", "--slice", "{slice}"]
required = true

[[step]]
id       = "explore.canon"
text     = "Run /canon so the ADRs, policies and standards governing this surface are in view."
required = true

[[step]]
id       = "explore.memory"
text     = "Run /retrieve-memory over the files and subsystems you expect to touch."
required = true

[[step]]
id       = "explore.triage"
text     = "Triage the design surface: open questions, risks, assumptions, shaping decisions, constraining governance."
required = true
```

**Step ids are explicit and author-assigned, never positional.** If identity were
list position, a project inserting a step at the top would silently repoint every
persisted discharge. This is the same rule the repo already enforces on
`PHASE-NN` and `EN-/EX-/VT-` ids, for the same reason.

**Step ids are not `DesignId`s, and PHASE-16 must decide that explicitly rather
than discover it.** `DesignId::parse` requires one of seven closed prefixes
(`inq- sec- cp- att- int- fnd- dlg-`) and a body of `[A-Za-z0-9_-]` only, bounded
at 32 bytes (`src/design_run/ids.rs:82-83,117-134`). `explore.canon` satisfies
neither the prefix rule nor the charset. Since discharge records are their own
run state (§2.3) rather than `Evidence` rows, step ids need not be `DesignId`s —
but any slot that *does* take a `DesignId` (a refusal field, an evidence subject)
constrains them, and the dotted form is the authored contract.

**An id solves reference, not equivalence — and revision 1 confused the two.** The
id lets you say *which* step you mean. It does not say whether the step you mean
today is the same obligation the discharge was made against. A record reading
"`explore.canon` discharged at revision 40" carries no trace of what
`explore.canon` *said* at revision 40. Four consequences, all real:

- **Text mutation.** A step reading "Run /canon" is attested; the project rewrites
  it to "Run /canon **and** read every ADR tagged for this surface". The discharge
  stands. An agent attested to one sentence; the record now claims a larger one.
- **`required` flip.** An advisory step is discharged cheaply, then flipped to
  required. Whether the old discharge suffices is undefined.
- **Verifier substitution — the case rev 1 explicitly called safe, wrongly.** A
  step verified under one command keeps a "verified" record when the project
  substitutes a stricter one. Keying on the id is what makes substitution
  *possible*; it is not what makes it *safe*. The id carries the contract's
  **name**, never the contract.
- **Delete and reuse.** A removed step's record becomes a visible orphan — until
  a later step takes the same name and silently inherits it. Durable ids are
  exactly what make resurrection possible.

**So a discharge binds the digest of the step's definition** (`id`, `text`,
`required`, `verify`), not the id alone. Change any of them and the discharge is
stale by construction, surfaced as stale, and re-made deliberately. Delete then
re-add with different content yields a different digest, so nothing resurrects;
delete then re-add *identically* yields the same digest and stays discharged,
which is the right answer rather than a lucky one. Skips bind it too — "I skipped
this because X" must not outlive X's step changing.

**The canonical encoding is a shipped contract and must be specified, not
implied.** Revision 2 asserted the digest's *properties* without ever saying what
bytes are hashed. "Delete then re-add identically yields the same digest" is not
a stable claim across parser versions, field ordering, or optional-field
representation unless the encoding is pinned. PHASE-16 owes a **versioned,
length-delimited canonical form** over the parsed fields — computed shell-side,
compared in the pure core, and carrying its version so the encoding can change
without silently invalidating every persisted record. This is §0's first
hardens-on-ship item.

**The precedent citation is corrected.** Revision 2 wrote that "section
attestations already bind the payload fingerprint, the disposition, the node
**and** the revision". They do not: `Attestation` is
`{ id, subject, fingerprint, reviewer }` (`src/design_run/attestation.rs:36-41`).
The four-part binding belongs to `AcceptanceAttestation`, whose derived `digest`
covers the checkpoint payload fingerprint, the inquiry disposition and the run
revision (`:109-112`). The *principle* survives — content binding with
liveness-by-fingerprint is real, and `Attestation.fingerprint` under DEC-066 is
its precedent — but the sentence was citing the wrong type.

It was quoting a shipped asset, and **the shipped asset is wrong too**:
`install/design-prompts/reviewing.md:6-9` states the same four-part binding of
"an attestation" immediately after naming section attestations. Filed as backlog;
not this phase's to rewrite, since PHASE-08 owns the fragment bodies.

**Granularity is per step, not per asset.** `Fragment::parse_receipt` is a
whole-asset digest for deciding whether to re-send bytes. Applied here it would
invalidate every discharge in a runbook on any edit — a project fixing a typo in
step 5 would reset step 1.

**Shipped step ids are API.** Doctrine's downstream logic keys on the step id,
never on the command behind it, so once the override seam lands (§8) a project
will be able to substitute `explore.research`'s verifier and keep the contract's
identity. What the digest adds is that the substitution is *visible*: the
discharge goes stale and must be re-made under the new contract, rather than
silently inheriting a result the new contract never produced.

## 4 · The verifier contract

A step's `verify` is an **arbitrary executable, zero on success**. It is optional;
presence of a command *is* the discharge mode.

This is open-ended by construction, which is the point. A closed Rust predicate
vocabulary — an earlier proposal — would need a Rust change per new check,
reimporting exactly the recompile-per-revision cost that made "sub-state per step
in Rust" unattractive in the first place.

**An argv array, not a shell string.** `{repo_root}` can contain spaces and shell
metacharacters, so substituting into a command string turns data into syntax.
Escaping each value fails differently depending on the author's own quoting. The
command is therefore a **list of arguments**, each element interpolated as a whole
opaque value with no shell between it and `exec`. This is a correctness
requirement independent of the accepted trust boundary (§10).

*(Revision 2 rested this on two legs, the second being that "user step ids are
explicitly free-form". Under §3 that is not reliably true — a step id may be
constrained by whatever slot it rides. The `{repo_root}` leg carries the argument
on its own, so the conclusion is unchanged and the second leg is dropped.)*

**The closed set is the placeholders, not the checks** — `{slice}`, `{run}`,
`{repo_root}`, `{step}`, single-sourced named constants per STD-001. A verifier
cannot check anything without knowing what it is checking, so interpolation is
mandatory; that vocabulary is small and does not grow when checks grow. An
unknown placeholder is refused at validation, never discovered at execution.

**Exit code alone is too thin.** Capture stdout/stderr and make it the refusal's
detail. The refusal ethos here is already "the refusal names the key it objected
to"; a bare non-zero tells an agent it is blocked and nothing about why.

### 4.1 · Where verifier results actually go

Revision 2 wrote that verifiers "run shell-side, and their results enter as
payload", riding "into `DerivedDesignFacts` — which is already where
`gate::advance` takes its inputs". **Both halves are wrong**, and the second
review traced every hop:

- **The payload has no slot.** `StageDeclaration` is `{ to, reason }`
  (`src/design_run/submission.rs:445-450`).
- **`DerivedDesignFacts` is not the shell-input carrier.** It is *persisted
  snapshot state* (`src/design_run/snapshot.rs:386`), re-observed from sections
  (`run.rs:1272-1279`), and `advance` reads it off the snapshot
  (`run.rs:1263`). The carrier for shell-derived facts is **`DerivedInput`**
  (`run.rs:87-96`) — today section digests, authored sections, and the authored
  document's fingerprint.
- **And the obvious workaround is refused by an existing invariant.** Routing
  results through `request.evidence` *compiles*, and then
  `run.rs:196-203` returns `Refusal::DerivedConditionClaimed` for any evidence
  naming a derived condition. A verifier result is precisely a fact Doctrine must
  derive rather than accept on a caller's word, so the honest form is refused and
  the dishonest form — a *claimed* condition — makes the verifier decorative and
  cannot bind a fingerprint anyway, since `current_fingerprint` resolves subjects
  only from `next.sections` (`run.rs:1283-1290`).

**The corrected architecture.** The shell executes applicable verifiers after
parsing the act and before admission, and places the results in **`DerivedInput`**
(extended) or a sibling ephemeral verifier-standing type — never in caller-authored
payload. That value threads `apply` → `stage_move` → `advance`, arriving as the
`runbook_standing` argument of §2.5. A result is *persisted* only when a discharge
is admitted; a caller-supplied result is never trusted.

The two properties revision 2 wanted are preserved, and now by a path that exists:

- The pure layer takes verifier results as **inputs** and executes nothing — the
  standing pure/imperative rule.
- Nothing slow sits inside `apply`'s admit→persist span. `apply` reads one
  snapshot (`src/commands/design.rs:1224`) and the only pre-write recheck covers
  the authored watermark before `write_snapshot` (`:1305-1315` — recheck at 1311,
  fault hook 1313, write 1314). That window is microseconds today because nothing
  slow is in it, and a subprocess would blow it open, so no subprocess goes there.

**Gate-time re-verification obeys the same rule.** Required verifiers are
re-evaluated shell-side before the stage act is submitted, and their results ride
the same derived-input path into `advance`. So a step that verified early cannot
have silently regressed by the time the run advances, and still no process runs
inside the pure core.

*(The pre-existing narrowness of the admit→persist window is untouched by this
phase and is not this phase's to fix — filed separately as a latent issue.)*

### 4.2 · The verify family

**Doctrine's own checks are a public family at a durable address.** Verifier argv
lands in committed TOML, so the address is API: moving a shipped check later
breaks every downstream runbook. Top-level `doctrine verify <key>`, confirmed
free of name collision at head, and `verify` is already this CLI's established
verb-word (`coverage verify`, `slice verify-vt`, `memory verify`,
`review verify`).

**That name is free but not costless, and there are two integration points, not
one.** Every visible top-level command must join `FAMILIES`, and
`families_partition_the_visible_command_tree` (`src/commands/cli.rs:1917`) panics
on an unclassified command. **It also asserts a hard census** —
`assert_eq!(visible.len(), 55)` at `:1953` — which a 56th command reds
independently of classification. Help and boot-map goldens derive from the same
taxonomy. PHASE-16 owns all of that explicitly rather than discovering it.

**Do not borrow exit codes from verbs that exist for other reasons.** `doctrine
slice research <id>` exits for reasons unrelated to any runbook contract, and the
day that changes, runbooks break silently. The verify family's *only* contract is
exit code plus explanatory output.

**Validation is owned, not inherited.** Revision 1 claimed runbook TOML "inherits
validation" from `doctrine prompt check`. It does not: that verb loads
`.doctrine/hymns` plus embedded hymns and checks the replaces graph, stage
vocabulary, seal integrity and hymn markers (`src/commands/prompt.rs:215-221`,
`check_corpus` at `:301-350`). It knows nothing about `design-prompts/*.toml`.
Reusing the verb is reasonable; it requires a **second corpus loader and
validation domain**, whose surface is: unknown fields rejected, non-empty step
list, non-empty text, id grammar and bounds, duplicate ids rejected, empty-argv
`verify` rejected, `mode` in the closed set, placeholders known.

## 5 · Progression is attestation

The progression API has no bare "next". **You advance by attesting a named step**,
and there is no verb that means "move on" without a name attached to why.

**`sequence` is the only mode PHASE-16 ships.** The run holds a cursor; the
discharge act must name the step **at the cursor**; naming any other is refused;
discharging advances the cursor. The gate clears when every `required` step is
discharged.

**`set` mode is deferred**, and the reason is §0's posture applied honestly.
PHASE-16 ships exactly one runbook and it is a sequence. `set` would land with no
shipped user, and — because ruling (b) removed the project-path seam (§8) — with
no end-to-end test able to reach it. That is speculative generality plus an
unexercisable criterion, which is precisely what shipping-to-learn is supposed to
avoid.

> **Reassigned 2026-08-01** (owner ruling; RV-325 round 5 F-11, contest re-fixed
> at round 6). This paragraph continued: *"PHASE-08 converts the two genuinely
> set-shaped checklists (present-design's fifteen content items, the adversarial
> pass's attack list) and will have two real instances to design the mode
> against."* **Both halves lapsed.** D14 reclassified those checklists as stage
> framing, so PHASE-08 converts neither; F-5's settlement then produced **five
> order-independent steps** under `sequence` across edges 2 and 4, so the
> *real instances* precondition is **met by different instances**. The
> no-shipped-user argument above was true **of PHASE-16** and is **not** a
> warrant for deferring past it — that claim is withdrawn wherever it appears.
>
> **`set` now belongs to IMP-373, not to PHASE-08.** What defers it is the
> **render**: a cursorless runbook has no rendering rule under `EX-14`'s token
> bound, and that question is unsketched and outside PHASE-08 `EN-2`'s scope.
> DEC-101 carries the governing amendment in both tiers; PHASE-16 `EX-7`/`EX-14`
> are annotated with their discharged obligations preserved.

The reasoning survives intact as the argument for adding it *eventually*:
explore's five steps are a genuine sequence (the triage needs the canon sweep
done), whereas a coverage set has no order to impose and imposing one is fake
determinism. Revision 2 was right about the distinction and premature about the
delivery. That distinction now has live instances rather than hypothetical ones —
the five order-independent steps shipping under `sequence` on edges 2 and 4 — and
the imposed order on them is **conceded and judged cheap**, not defended.

Outcomes, in the shipped mode:

- a step **with** a verifier: the attestation is corroborated by an exit code, and
  the verifier's output is recorded with it;
- a step **without** one: the attestation stands alone — recorded, revision-bound,
  digest-bound, auditable, and *believed*;
- a step that cannot be done: **discharge-with-reason**, reason required,
  rendered in the envelope and kept in the record.

That last case is not a nicety. Once the gate blocks on required steps, a project
whose runbook contains a step its agent cannot satisfy would otherwise wedge the
run. Disclosed deviation beats a wedged machine, and it beats a silent skip.

## 6 · The authoring rule

Written down deliberately, because without it the runbook accretes
invariant-restatements — which is exactly how the four shipped fragments ended up
in the wrong register.

> **Could a project legitimately do this differently?**
> **Yes** → runbook step. Overridable, verifier substitutable.
> **No** → engine invariant. Enforced by `apply` / `advance`. Never a step.

True invariants stay out of runbooks altogether. A user cannot override "every
mutation compare-and-swaps against the run's revision" — the code does that
regardless, so an override would not be *different*, it would be *false*.

This is the same line that dissolves the customization fork (§8), which is fair
evidence it is the real one.

## 7 · What this does not claim

`/canon` and `/retrieve-memory` have no possible verifier. No exit code proves an
agent read something.

So the runner's guarantee is **sequenced, gated, and auditable**, and *some* steps
are additionally **verified**. Steps without a verifier are best-effort and
trust-reliant, and the record says so per step rather than in a footnote.

This is stated at this volume on purpose. This slice has been bitten more than
once by a surface claiming detection it did not have (DEC-100's watermark
contract; RV-321 F-3's over-claim, and then the same shape one level up in the
*record* of F-3's remedy). PHASE-09's rubric must score adherence-under-attestation
separately from verified discharge rather than collapsing both into one number
that flatters the mechanism.

**Not claimed, second:** the runbook is not yet *overridable by a project* — §8.

### 7.1 · The null-result problem, and why it is deferred

A verifier that checks a step's *outputs* — "the slice captured at least one
governance finding" — is wrong by construction. In a repo with few or no
applicable governance documents, "none applicable" is both the likely truth and
the signature of not having looked. Make the check blocking and it is wrong in a
clean repo; make it advisory and it proves nothing.

The check that works points at **inputs**: was the sweep performed over the
corpus that exists? That is mechanically decidable, and it separates the cases —
no record at all means the work was not done and the gate blocks; a discharge
carrying the scope it swept is a positive, bounded, *perishable* claim that goes
stale when the corpus grows.

What it still cannot do is prove the agent thought. §7's limit is unchanged.

**Deferred, deliberately.** Binding a swept scope needs either a fourth outcome
arm on the record or a scope declaration on the step, and per §0 the record's
shape hardens on ship while the right shape is exactly the sort of thing the
running mechanism answers better than prose. So PHASE-16 ships the version tag
(§2.3) that makes the arm cheap to add, records the reasoning here, and does not
guess. A verifier that wishes to emit swept scope in its captured output may
already do so — the record keeps verifier output — it simply is not structured.

## 8 · Customization — the fork this dissolves, and the seam that does not exist

The open fork was whether the four `design-prompts/*` and `hymns/stage/design.md`
— the only five `customization = "fixed"` entries in `publication/manifest.toml`
(confirmed at `:88-122` and `:250-257`) — should be reversed to customizable.

They should not, and the reversal is not needed. The principled line is §6's:

- **Invariants stay sealed**, because overriding them would make them *false*.
- **Craft ships overridable**, because a project legitimately disagrees. The live
  proof is `plugins/doctrine/skills/design/SKILL.md:98` ("prefer multiple choice
  questions when possible") against this repo's own `CLAUDE.md`, which says the
  opposite for design loops.

DEC-077 **deferred** override semantics rather than forbidding them, so the
decision **narrows** DEC-077 for one asset class instead of superseding a ruling.
ADR-019 does not move: it mandates only that every projected asset *declare* a
customization, and names no hymn.

**But `customization` has no execution seam behind it today.** The field is parsed
and displayed; its only production consumer outside `publication.rs` is the
library view (`src/commands/library.rs:111`). Design fragments are fetched
straight from the embed via `install::asset_text` → `asset_source::read_text`
(`src/commands/design.rs:1633-1638`). **Declaring a runbook `customizable` would
therefore make it no more overridable than anything else.**

Owner's ruling, 2026-07-31: **option (b)** — the runbook ships embedded like every
other design asset, and the override seam is **identified and deferred**, not
delivered.

**Revision 3 sweeps the consequences revision 2 left behind.** Having conceded
that no override seam exists, revision 2 went on speaking in the present tense
about project ownership: that a project "may substitute" a verifier, that argv
"lands in project-authored, committed TOML", that the trust surface includes
"whatever a project vendors into its own tree". `plan.md` carried the same
tense. All of it is now future-conditional on the seam landing. Two consequences
follow and are recorded rather than absorbed:

- **§4's argv argument loses one of its two legs** — the `{repo_root}` leg
  carries it alone (§4).
- **Two of `VT-1`'s named tests were unreachable as written.** Mutating a step
  definition and exercising `set` mode both require a runbook the phase neither
  ships nor can vary, because the only source is the embed. `set` mode leaves the
  phase (§5), which dissolves one. The digest-staleness test remains and is
  reachable **only** through an in-memory fixture over the `#[path]`-included
  pure model — the idiom already used at `tests/e2e_design_delegation.rs:36-43`.
  `VT-1` now says so rather than implying a CLI end-to-end test that cannot
  exist. The digest binding is the one thing that must not ship unverified.

What survives intact: the *reasoning* about which assets are sealed and which are
craft, and the concrete win that the craft has moved out of skill prose into data.
What is honestly downgraded: the delivery claim. The `SKILL.md:98` contradiction
is fixed for this repo and not yet for anyone else, because overriding still
requires a rebuild.

## 9 · Phase boundary

PHASE-16 (appended; ids are immutable, so it executes out of numeric order —
which this plan already does, its order being 01→…→11→12→10→07→16→08→09).

**PHASE-16 ships the mechanism plus one runbook** — the guard on the edge out of
`Exploring`, the sharpest instance: the state with the most explicit ordered
checklist, whose incumbent guard turns out to be two unimplemented placeholders
(§1.1).

**PHASE-08 does the remaining prose work** as part of the body rewrite it was
already scoped as. Prose is cheaper than mechanism, and it balances the two
phases' load.

> **Reassigned 2026-08-01** (owner ruling; RV-325 round 7, F-11 second contest).
> This paragraph read: *"PHASE-08 rewrites the remaining two prose checklists
> into runbooks … present-design's fifteen content items and the adversarial
> pass's attack list. It also **lands `set` mode** (§5), with two real instances
> to design against."* **§5's amendment block withdrew both claims and this
> restatement was missed** — the same defect one section apart. D14 reclassified
> both checklists as stage framing, so neither conversion happens, and `set`
> belongs to **IMP-373, not PHASE-08**. See §5.

PHASE-16 runs **before** PHASE-08. The dependency is real: PHASE-08 `EX-1` says
the workflow machine's deterministic mechanism now lives in Doctrine, and without
the runner the rewrite can only drop the checklists (losing them) or keep them
(staying fat, failing `VA-4`'s anti-theatre size comparison). The runner is what
lets PHASE-08's rewrite be honest.

## 10 · Trust boundary — decided, disclosed

An executable in a runbook means `doctrine design` runs project-authored code.

**Accepted.** The runbook is committed alongside the repo's own `justfile` and CI
configuration, at exactly that trust level — an agent already runs those. Owner's
ruling: *"if an attacker can write hymns, all agents are compromised and we're
fucked anyway."* Gating custom executables behind a `doctrine.toml` opt-in was
considered and rejected as theatre: the attacker who can write your runbook can
write your justfile.

At v1 the surface is narrower still: §8 defers the override seam, so the only
runbook that runs is the **shipped embedded** one. The ruling is unchanged and
will hold when the seam lands and a project can vendor its own.

## 11 · Open — for `/phase-plan`, not for now

- **The seventh act's wire key.** `discharge` is the working name. `attest` reads
  closer to the §5 semantics but collides with the existing content-bound section
  *attestation*. A §0 hardens-on-ship item: it is a wire key on `ApplyRequest`.
  Naming call at phase-plan.
- **The canonical encoding of the step-definition digest** (§3). Also
  hardens-on-ship; must be settled at phase-plan and carry a version tag.
- **Which shipped checks `doctrine verify` launches with.** `research-current` is
  certain; the rest should follow the runbooks that need them rather than be
  speculated into existence.
- **`Locked` yields no fragment by design**, but state 8 (transition to planning)
  still has work to instruct. Under §2.1's edge-keying the lock transition is a
  natural home; out of scope for PHASE-16.

*Closed since revision 1:* runbook drift — the answer is the per-step definition
digest (§3).

*Closed since revision 2:* **the stale-after-clear rule.** Owner's ruling,
2026-07-31: a discharge whose definition changed after its stage cleared
**warns**; it does not block and does not un-advance. `EX-16` already stated the
rendering half; §2.6 now states the evaluation half it rested on. Nothing remains
to decide.

## 12 · What the adversarial reviews changed

### First pass — over revision 1

External prose review, codex, read-only, 2026-07-31. Deliberately **not** an RV
ledger — the owner declined a fifteen-round disposition cycle for a
pre-implementation catch. Ten findings; nine stood on independent verification.

| # | finding | outcome |
|---|---|---|
| F5 | `customization` has no execution seam; the override claim is false | **confirmed** → §8, owner ruling (b) |
| F2 | `can_advance` is edge-membership, not clearance | **confirmed** → clause moves to `advance` |
| F1 | nothing rechecks the revision before `write_snapshot` | **confirmed**; resolved by keeping subprocesses out of the span (§4.1) |
| F3 | step id binds reference, not equivalence | **confirmed** → §3 rewritten |
| F4 | `sequence`/`set` are two transition semantics behind one flag | **confirmed** → distinguished; `set` since deferred (§5) |
| F7 | placeholder interpolation into a shell string | **confirmed** → §4, argv array |
| F9 | the `WRITER_ACTS` comment does not anticipate a seventh member | **confirmed** → §2.4 |
| F10 | a new top-level command must join `FAMILIES` | **confirmed** → §4.2 |
| F6 | `prompt check` does not inherit validation for this asset class | **confirmed** → §4.2 |
| F8 | PHASE-16's verification is underpowered; `VT-2` is theatre | **confirmed** → `plan.toml` |

Refinement on F1: admission *does* compare-and-swap. What is true is that the
admit→persist span has no recheck, so a subprocess in it would open a window that
is currently negligible.

### Second pass — over revision 2

A **fresh** codex session, read-only, 2026-07-31, deliberately not the agent that
produced the first pass. Every load-bearing claim was verified against source
before acceptance; nothing was taken on deference.

| # | finding | outcome |
|---|---|---|
| G1 | `exploring.toml` has no selector — `Fragment` is closed and has no exploring variant | **confirmed** → §2.1, edge-keyed guard |
| G2 | `Condition` cannot express a per-obligation gate — payload-free, existential `satisfies`, refusal has no room | **confirmed** → §2.5, third derived input |
| G3 | verifier-results-as-payload does not exist; `DerivedDesignFacts` is snapshot state, and `request.evidence` is refused | **confirmed** → §4.1, `DerivedInput` path |
| G4 | present-tense project-substitution language survives ruling (b); two `VT-1` tests unreachable | **confirmed** → §8, §5 |
| G5 | the section-attestation precedent is a false citation; canonical encoding unspecified | **confirmed** → §3 |
| G6 | the stale-after-clear rule is listed open but `EX-16` already settled it | **confirmed** → §11 |
| G7 | the eight `VT-1` tests do not inherently collapse; `VA-NC`'s risk is misdiagnosed | **confirmed** → sequenced by seam at phase-plan |
| G8 | `FAMILIES` claim incomplete — the hard census at `cli.rs:1953` also reds | **confirmed** → §4.2 |

Two findings were the reviewer's and two were the adjudicator's: G1 and G2 were
reached independently by both; §1.1 (the incumbent guard is an unimplemented
stub) and the `DESIGN_ID_BYTES` bound on a new condition name (§2.5) came out of
verification rather than out of the review.

**Accurate as repaired, and re-verified:** the `WRITER_ACTS` seventh-act account
and its second table; `prompt check`'s loader scope; `library.rs:111`; embedded
asset resolution; `verify`'s freedom from collision.

## Provenance

- Owner ruling 2026-07-30 replacing PHASE-08 `EN-2`'s procedure; forks recorded
  in `notes.md` `## Harvest → Open` at `0c57cbea`.
- Fork 1 settled 2026-07-31: the generic ordered-obligation primitive, contents
  as asset data.
- Fork A settled 2026-07-31: TOML sibling, not a Markdown convention.
- Fork B settled 2026-07-31: insert the work now rather than ship a fat design
  skill and defer.
- §4's verifier-as-executable model is the owner's, replacing an earlier
  closed-predicate-vocabulary proposal.
- §5's attestation-as-progression is the owner's.
- §10 is the owner's ruling, verbatim above.
- §8's option (b) is the owner's ruling, 2026-07-31, after the first review.
- §0's ship-and-learn posture, §2.1's guards-belong-on-edges framing, §2.6's
  warn-not-block rule, and the rejection of derived truth flowing from
  user-customisable material into a closed vocabulary (§2.5) are the owner's,
  2026-07-31, after the second review.
