# Review RV-370 — design of SL-246

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

### Subject

`SL-246` — *Entity reads carry their knowledge records*. The design is
`.doctrine/slice/246/design.md` (nine sections, ~1,086 lines), materialised at
design-run revision 35. Read it with `doctrine slice show SL-246` for scope and
the file directly for the design. The slice's durable rulings are `DEC-145`
through `DEC-151` plus `DEC-260` — read each with
`doctrine knowledge show DEC-NNN`; the design's §7.1 table is a map to them, not
a restatement, and **the records are normative where the two differ**.

### Round and bar

**Round 1 — the first adversarial pass over this design.** The full bar applies:
any severity, any section. This is not a late regression round, so a marginal
finding is worth raising if it is real.

What makes a finding admissible:

- It is actionable **against the design as written**. "Consider also…" with no
  defect named is noise.
- It cites the thing it attacks — a design section, a `DEC` record, a governance
  id, or a file:line in the tree.
- It distinguishes **observation from proposed repair**. Raise the defect; a
  suggested fix is welcome but is a separate claim and will be adjudicated
  separately.

What is out of scope:

- Re-litigating a `DEC` whose *conclusion* holds on an argument the design has
  already repaired (see `F1` below) — attack the repair, not the retired
  argument.
- Corpus hygiene (`IMP-403`), the `SPEC-019` four-of-seven record-kind gap
  (`ISS-316`), and the `IMP-457` rehome. All are explicitly deferred with
  carriers; a finding that they *should not* have been deferred is in scope, a
  finding that they are unfixed is not.
- Implementation-level code review. No code exists yet.

**A round that raises nothing is a successful outcome and should be stated
plainly.** Do not manufacture findings to fill a quota.

### Lines of attack — where the bodies are likely buried

Ranked. The first three are where the author's own confidence most exceeds the
evidence.

**1. §3 `F1` is an unreviewed claim by the design's author, and three things
rest on it.** The design asserts that `scan_entities` calls `outbound_for` per
entity (`src/catalog/scan.rs:192`), which for a record prefix dispatches to
`knowledge::relation_edges` (`:65`) → `read_record`, and therefore that **every
corpus scan already reads, parses and validates all ~360 knowledge records
(~964 KB of prose) and keeps only their relation edges**. That claim is used to
(a) falsify `DEC-146`'s stated cost, (b) falsify `DEC-146`'s rejection argument
for the scan-carried alternative, and (c) justify `IMP-459` and part of
`IMP-460`. **Verify it against the tree.** If the dispatch is conditional, if
`ScanMode` gates the record read, if `relation_edges` does not reach
`read_record`, or if the body is not in fact read at scan time, then §3 `F1`,
§7.3's first bullet, and the repaired rationale all fall — and `DEC-146`'s
original argument would have to be reinstated rather than replaced.

**2. `DEC-260` has had no adversarial pass at all.** It was taken during
drafting, off a mid-session framing correction, and was never an inquiry node.
Everything it rules is unreviewed:

- siting the read at `slice design show <SLICE>` (three-level grammar) rather
  than any alternative;
- retiring the deprecated `slice design <ID>` leaf and `scaffold_design_doc`;
- the claim (§5.6, §2.5) that retiring the leaf **deletes** the residual
  dispatch arm at `src/commands/cli.rs:1531` and removes a documented `ADR-001`
  cycle workaround. Check that against the tree — if that arm has another
  caller, the payoff claimed in §5.6 is wrong.
- `R5`'s claim that the removal "completes a deprecation rather than starting
  one" — is the deprecation actually announced anywhere a scripter would see it?

**3. §5.2's signatures are unwritten code asserted to compile and to be total.**
Three assertions to attack:

- `render_block` is **total** — never returns `Err` — while reading files.
  Does every failure mode inside it actually have a disclosure path, or is
  there one (a panic, a poisoned lock, a non-UTF-8 path) that totality cannot
  absorb?
- `SelectedRecord` is defined in `knowledge` so that `relation_graph →
  knowledge` stays acyclic. **Check the current direction of that edge**; the
  design claims `knowledge` imports nothing from `relation_graph` today.
- `Facets` never opens the `.md`, yet the unfilled-facet marker carries a prose
  **size hint** ("6.7 KB of prose") obtained from a `metadata()` call. Is that
  actually reachable without a read, and is the byte count it yields the number
  the marker claims?
- `format_facet(facet, tier, empty)` gaining two policy parameters while
  `knowledge show` stays byte-identical (`C2`) — is that compatible with every
  existing caller, and is `facet_json` left inconsistent with it?

**4. §7.2's four drafting decisions (`D1`–`D4`) bypassed the inquiry entirely.**
They are settled by assertion in the same document that proposes them. Attack
each on its merits: the JSON object shape (`D1`), verbatim document render
(`D2`), the stated empty set (`D3`), and returning the pointer line to
`IMP-398` (`D4`).

**5. Governance application, not governance citation.** §3.1 names `ADR-004`,
`ADR-001`, `SPEC-013`, `SPEC-018`, `SPEC-019`, `STD-001`, `STD-002`, `STD-003`.
Check the *application*, not the list: does `slice design show` actually satisfy
`SPEC-013`'s grammar clause (the design defends three levels by the `slice
selector` precedent — is that precedent real?); is `STD-003`'s disclosure
obligation genuinely discharged by `I5` alone; does `STD-001` bind every string
the design introduces (three markers, three level names, the tier annotation,
the empty-set line, the stale-run note in `OQ-1`).

**6. The seam claim (`C6`, `DEC-147`, objective 3).** The design promises the
transitive closure (`IMP-398` S5) will **extend** this, not replace it. Is that
true of the shape actually proposed? `select_knowledge(view: &InspectView)` is
typed on a one-hop view — can a depth-N selection produce `Vec<SelectedRecord>`
without changing that signature or that type? If not, the seam is asserted and
not built, and objective 3 is unmet.

**7. Verification adequacy (§9).** `VT` is twelve named cases over a synthetic
fixture corpus. Does that set actually cover every invariant `I1`–`I7` and every
edge case `X1`–`X7`? Specifically: `I4` permutation-invariance past id 999 is
claimed but the named case does not obviously reach it; `X3` is routed to an
open question and so is unverified; `X7` (malformed `[facet]` table) has a
fixture but no named case. And §9.5 declines a live-corpus invariant test — is
the argument for declining sound, or does it leave `C1` proven only on synthetic
bytes?

**8. Open questions that may not be optional.** §6 carries `OQ-1` (disclose a
stale `design.md`, `QUE-223`), `OQ-2` (per-kind levels), `OQ-3` (`Full` vs
`knowledge show`). Each has a recommendation. Is any of them actually blocking —
i.e. would answering it the other way change §5's structure rather than a call
site? `OQ-3` in particular: if `Full` does **not** delegate to `format_show`,
`C4` is violated and that is a design fork, not the "code-reading question" §6
calls it.

**9. Prose and diagrams.** Does the reader have to reconstruct the design from
identifiers? Do the three mermaid diagrams (§2, §5.1, §5.4) agree with the
prose, and does each carry the load-bearing edges rather than decorate an
inventory? Is anything in the design only intelligible to someone who sat
through the design run?

**10. The slice mints a carrier that already existed.** During drafting this
design minted `IMP-457` ("Design-run verbs occupy the document-render verb
slots") as the carrier for the information-architecture defect that forced
`DEC-260`'s siting. `IMP-393` ("Reader-facing design render for review", opened
2026-08-03, six weeks earlier, originating from `SL-244`) already states the
same complaint in the same terms — `show` means *render the entity's document*
everywhere else, `design show` is the writer's turn envelope, and the fix is
either reverting `show` to the convention or rehoming the envelope to its own
verb. `IMP-393` goes further: it also specifies the composed reader-facing
render this slice partially builds. Read both.

Three things to rule on:

- Is `IMP-457` a duplicate of `IMP-393`, and if so which survives? The project
  bans parallel implementation; parallel *carriers* for one defect are the same
  failure in the backlog.
- `R6`'s mitigation is "the intent is recorded on `IMP-457` with the collapse
  named". If the older, broader item is the real carrier, that mitigation points
  at the wrong row and `R6`'s residual is understated.
- `SL-246` now carries `fulfils IMP-393 --degree partial` (added after the
  design was written, at the user's observation — the design does not mention
  `IMP-393` anywhere). Does §1's boundary statement, §2.5, and `DEC-260`'s
  reasoning survive contact with an item that had already scoped this ground?
  In particular `IMP-393` raises a resolution option `DEC-260` never considered:
  the envelope's own `--format` slot already spells the writer rendering
  `prompt`, so making the reader rendering the default may be smaller than
  `DEC-260` assumed.

### Standing constraint on the pass

The design **must stand alone**. It may not require the review chronology, the
design run's state, or locally-invented terminology to be understood or
implemented. A place where it does is itself a finding.

---

## Round 2 bar — added 2026-09-18, after integration

Round 1 raised `F-1`..`F-8`; all eight are disposed and the baton is back with
the raiser. The design was then rewritten: eight of its nine sections moved
(`sec-4` alone is untouched), adopted at run revision 38.

### Two jobs, and they are different

**1. Adjudicate the eight dispositions.** Each finding is `answered`. Verify it
(the disposition is accepted, terminal) or contest it (hands it back). Three
dispositions make claims worth testing rather than accepting:

- `F-1`, `F-2`, `F-8` are disposed `dissolved-by-dec-261` — the defect is gone
  because its premise is gone, not because it was repaired. Test the premise:
  is `doctrine design show <SLICE>` actually two levels and actually conformant;
  does the handler now sit somewhere `ADR-001` permits; is `guard.rs`'s
  `SliceCommand::Design` row genuinely deleted by the leaf retirement rather
  than needing a nested classification.
- `F-3`'s disposition **corrects your finding** — it says `run_inspect` already
  writes scan diagnostics to stderr, so the repair is deleting an unreachable
  second disclosure rather than building a first one. Check that correction. If
  it is wrong, contest.
- `F-1`'s disposition **rejects part of your prescription** — revising SPEC-013
  before implementation inverts this project's mechanism (a `REV` is minted
  after design and applied at reconcile). That is the user's ruling, not a
  negotiating position; contest the *reasoning* if it is wrong, not the ruling.

**2. Attack the new material, which has been read by nobody.** This is the
larger job. Round 1's bar and lines of attack still apply to it. The genuinely
new text:

- **§2.6** — wholly new. The claim that `design show --format [default: prompt]`
  makes reclaiming `show` a one-default move.
- **§3.1** — rewritten. Now claims SL-246 *conforms* to SPEC-013 and needs no
  governance to move. Also asserts six named three-level groups exist under
  numbered entity kinds; verify the list.
- **§3.3 `F5`** — rewritten, and it asserts a measured migration cost of 14
  references with a specific breakdown. Verify the count and the breakdown.
- **§5.2** — the `facet_fields` structured projection is new design, not a
  repair of old text. Attack it as unreviewed: does it actually let `C2` hold
  (both existing renders byte-identical) while making the tier filter reach
  both arms? Is `FacetValue` sufficient for every kind's fields? Does the JSON
  `Full` entry as now specified actually match `show_json`'s payload?
- **§5.2 / `I5` / `X7` / §9.2** — the split of the disclosure population into
  scan-pruned (stderr) and race (in-block). Is the split exhaustive? Is the
  race case actually reachable in a test without fabricating it?
- **§5.6** — rewritten table plus three notes. Is the table now complete?
- **§6** — `OQ-3` closed and moved to `D5`; `OQ-1` and `OQ-2` re-argued.
- **§7.2 `D5`** — new decision, taken at review, never adversarially read.
- **§8 `R6`** — wholly replaced. The old ossification risk is gone; the new
  migration risk is unreviewed.
- **§9.2** — six new named cases and three new fixtures.

### Bar

Anything real, any severity — the new material is first-pass, not late-round.
But hold the round-1 admissibility rules: cite what you attack, separate
observation from prescription, and do not manufacture findings. **Raising
nothing on the new material, while verifying or contesting the eight, is a
legitimate and complete outcome.**

### Out of scope, unchanged

Corpus hygiene (`IMP-403`), the `SPEC-019` record-kind gap (`ISS-316`), and
implementation-level code review — there is still no code. `IMP-457` is closed
as a duplicate of `IMP-393`; do not re-raise the overlap.

---

## Round 3 bar — added 2026-09-18, at run revision 45

This is the round that decides whether the pass can be disposed `conducted`. The
run's `review-disposition-attested` contract will not take a pass while
dispositions stand unadjudicated, so the ten below are the gate, not the
formality.

Round 2 read **revision 38**. The design is now **revision 42** content, adopted
after integration, with five sections moved — `sec-3`, `sec-5`, `sec-7`, `sec-8`,
`sec-9`, roughly 200 added lines. Read the current text, not your recollection of
it: `.doctrine/slice/246/design.md`.

### Job 1 — adjudicate the ten dispositions

`F-4` and `F-9`..`F-17` are all `answered`. Verify each (the disposition is
accepted and terminal) or contest it (hands it back). Four make claims worth
testing rather than accepting:

- **`F-14` changed the verification mode, not the design.** Its disposition
  retires the fixture and the named case, and asserts that `I5`'s in-block half
  is verified **by construction** — `render_block` returns `String`, not
  `Result`, so no path exists on which an unreadable selected record aborts the
  block or vanishes from it. That is a claim about a signature that does not
  exist yet. Attack it directly: does totality actually foreclose the failure
  `I5` names, or does it only foreclose the *abort* half and leave the
  *vanishes-from-it* half unwitnessed? A function returning `String` can still
  return a string with the record missing from it.
- **`F-13`'s repair puts the marker inside `facet` as `{"marker": …}`** at both
  levels, and claims this keeps `Full`'s entry **exactly** `show_json`'s twelve
  keys plus `caption`. Check that against `show_json`'s actual payload in
  `src/knowledge.rs`, not against the design's description of it.
- **`F-17`'s disposition rules `DEC-150` unsuperseded** on the grounds that
  `facet_fields` *is* `DEC-150`'s per-kind match rewritten, not the separate
  constant `DEC-150` rejected on drift grounds. That distinction is the whole
  ruling. Read `DEC-150` (`doctrine knowledge show DEC-150`) and test it: if
  `facet_fields` is a new table that a future field addition can leave stale
  independently of `format_facet`, the drift argument `DEC-150` rejected is back
  and the record needs superseding after all.
- **`F-4`/`F-12` reduced `D1`** to "the two arms agree, mechanised by a shared
  `facet_fields` under a shared `EmptyPolicy`", deferring the entry shape to
  `D5` and § 5.2. Confirm `D1`, `D5` and § 5.2 now state one contract and not
  two.

`F-9`'s disposition records four user rulings; `F-10`'s rescopes `C1` and `I1`.
Both are legitimate targets under Job 2 rather than here — the disposition is
that the design changed, and what it changed to is new material.

### Job 2 — attack the new material, which no reviewer has read

Round 1's and round 2's bars still apply to it. The genuinely new or rewritten
text at revision 42:

- **§ 5.2 command grammar.** `--format` gains `document` **and defaults to it**;
  `--json` is refused alongside an explicit `--format`; `--full` is refused on
  `document`. Three refusals settled as user rulings during integration, so the
  grammar they produce has been attacked by nobody. Is the refusal set complete
  and are the three mutually consistent? Is a default-carrying `--format` on a
  verb whose old default was a different rendering actually a move rather than a
  removal, as the prose claims?
- **§ 5.2 `facet_fields` reaching `facet_json`.** `facet_json` now takes
  `EmptyPolicy` alongside `TierFilter`. Does `C2` — both existing renders
  byte-identical — still hold once the JSON arm grows a policy parameter, and is
  every existing caller compatible?
- **§ 3.2 `C1` and § 5.5 `I1`, as rescoped by `F-10`.** The rescoping is honest
  or it is the constraint defined down to what the design happens to satisfy.
  Decide which.
- **§ 5.6's table and its four notes**, especially the intended-red whitelist and
  the claim that a red suite not named there is the `C1` alarm. Is the whitelist
  exhaustive of the intended red?
- **§ 3.3 `F5`'s population table** — new at round 2. Its four emitted-string
  sites were re-derived independently and agree exactly, so the count is sound.
  What is unread is the table's *completeness*: are those four populations the
  whole migration?
- **§ 7.2 `D5`** — a decision taken at review and never adversarially read.
- **§ 8 `R6`** — rewritten twice now, once per round.
- **§ 9.2's split cases and § 9.5's by-construction argument.**

### Job 3 — the post-round-2 artefacts, unread by anyone

Made at run revisions 43-45, after the round-2 integration:

- **`slice-246.md`'s *Affected surface*, rewritten.** It had listed
  `src/commands/design.rs` under *Dropped by the inquiry* — contradicting
  `DEC-261`, which sites the design read in that file. Does the scope now assert
  nothing the accepted decisions contradict, and omit nothing they add? Read it
  with `doctrine slice show SL-246`.
- **The design-target selector set, re-pointed** off `DEC-260`'s siting
  (`doctrine slice selector list SL-246`). Does it match § 5.6's commitments?
- **`notes.md`'s third-pass section**, which is this bar's source and may be
  wrong about what is worth probing.

### Bar

Anything real, any severity. Hold the standing admissibility rules: cite what you
attack, separate observation from prescription, and do not manufacture findings.
**Raising nothing on the new material, while verifying or contesting the ten, is
a legitimate and complete outcome** — and given two rounds have already run over
this text, it is a plausible one.

### Standing constraint

The design **must stand alone**. It may not require the review chronology, the
design run's state, or locally-invented terminology to be understood or
implemented. A place where it does is itself a finding.

### Out of scope, unchanged

Corpus hygiene (`IMP-403`), the `SPEC-019` record-kind gap (`ISS-316`), the
`IMP-457`/`IMP-393` overlap (closed as a duplicate), and implementation-level
code review — there is still no code.

## Round 4 bar — added 2026-09-18, at run revision 47

Rounds 1–3 were read by one reviewer. **This round's raiser is a different model**
— Claude Opus, not the incumbent. You inherit the raiser role and this ledger
whole; you did not write the earlier findings and you are not bound to agree with
them. Where you think an earlier finding was wrong, say so as a new finding
rather than reopening a terminal one.

Round 3 read **revision 45**. The design is now **revision 47** content, adopted
after integration, with five sections moved — `sec-3`, `sec-5`, `sec-7`, `sec-8`,
`sec-9`, roughly 345 changed lines. Read the current text, not the diff's story
about it: `.doctrine/slice/246/design.md`.

The integration's own account is `git show 63516fa9e` — useful for locating what
moved, **not** as evidence that it moved correctly.

### Job 1 — adjudicate the five dispositions

`F-14`, `F-18`, `F-20`, `F-21`, `F-22` are all `answered`. Verify each (accepted,
terminal) or contest it (hands it back). Four are worth testing rather than
accepting:

- **`F-14`'s contest was *upheld*, so the thing to adjudicate is the repair, not
  the concession.** The disposition grants that `-> String` foreclosed only
  `I5`'s abort clause, and re-grounds the other two: the *map shape* forbids the
  drop, and `render_record` / `record_value`'s **never-empty contract** forbids
  the empty render (§ 5.2, § 9.5's table). Attack the second ground directly. A
  map is a shape a reader can check; a "contracted never-empty" function is a
  doc comment. If the never-empty clause is enforced by nothing but the
  implementer's intention, then the repair moved one clause from the type system
  to prose and called it by-construction — which is the same defect `F-14`
  named, one layer along. Decide whether that is so.
- **`F-18`'s repair gives the JSON arm `record_value` and `knowledge_value`.**
  `knowledge_value` at `Skip` **omits the key** rather than emitting an empty
  array, justified by `C1` byte-identity. Check that against `inspect --json`'s
  actual payload and against what `C1` says: is an absent key byte-identical to
  today, and does any existing consumer of that payload distinguish absent from
  empty?
- **`F-20`'s repair splits the three markers across two layers** — the by-design
  marker stays on `format_facet` / `facet_json` under `Marked`; the unfilled and
  unreadable markers move to `render_record` / `record_value`. The stated reason
  is that `metadata()` behind `format_facet` would put disk in a pure layer that
  `knowledge show` also calls. Test the premise: is `format_facet` actually in a
  layer that is pure today, and does `knowledge show`'s call path make that
  claim true? If the facet renderers already sit above the disk, the repair's
  justification is wrong even if its shape is right.
- **`F-21`'s repair adds `--known-revision` to the partition**, on the ground
  that the draft "enumerated four flags from its own prose rather than from
  `ShowArgs`, where there are five". **Count them yourself** in
  `src/commands/design.rs`. If `ShowArgs` carries a sixth, the partition is
  still not a partition and the repair repeated the original error.
- **`F-22`** repaired two counts and the integration's own sweep claims two more
  (`slice-246.md`'s second "14 references", § 5.6's "~4 sites"). Re-check all
  four against the tree. A count sweep that stopped one short is the finding.

### Job 2 — attack the new material, which no reviewer has read

Rounds 1–3's bars still apply to it. `D6` and `D7` are **decisions taken at
review**, written by the responder, and read adversarially by nobody:

- **§ 5.2's five signatures** — `render_record`, `record_value`, `render_block`,
  `knowledge_value`, `show_value`. Unwritten code asserted to compile, to be
  total, and to be never-empty. Attack each assertion separately.
- **`show_value` split out of `show_json`** (`src/knowledge.rs:1949`). The claim
  is that `show_json` builds that twelve-key map inline, so naming it leaves
  `show_json`'s envelope, `Result` and bytes unchanged (`C2`). Read the function.
  If the map is not inline, or if the split changes what `show_json` can return,
  `C2` is at risk and `Full`'s entry has no single projection.
- **The layering rule itself** — "the per-record producers may touch the disk,
  the facet renderers stay pure". Check it against `ADR-001` (module layering,
  no cycles) and against the pure/imperative split `AGENTS.md` states. A rule
  that is right in this design and wrong in the codebase is a finding.
- **`D7`'s machinery argument.** It claims the round-2 rule would have forced
  the implementation to read clap's `ValueSource` to tell a defaulted `document`
  from a written one, and that restating the refusal over the *rendering*
  removes that. Verify the second half: is the new rule enforceable from the
  resolved `--format` value alone?
- **§ 9.2's replacement case**,
  `json_is_legal_at_the_document_rendering_and_refused_at_every_envelope_one`.
  Its stated point is that it asserts a **pair**, so it cannot pass under the
  round-2 rule it replaces. Check that: does the legal half actually fail under
  the retired rule, or does the retired rule also permit `--json` alone?
- **§ 5.1's flowchart node and § 5.4's sequence**, both rewritten to carry the
  map. Do they now agree with § 5.2 — and does § 5.4's three-outcome `alt` cover
  the states § 5.2 names, no more and no fewer?
- **§ 3.1's `STD-001` restatement** — "one of the three is a bare constant, the
  other two are templates over a constant frame". Does `STD-001` actually admit
  a template as a single-source constant, or is that a widening of the standard
  written to fit the design? Read `STD-001`.
- **§ 8 `R6`**, now rewritten three times, once per round.

### Job 3 — the post-round-3 artefacts, unread by anyone

- **`slice-246.md`, rewritten again at the round-3 integration** (14 lines).
  Read it with `doctrine slice show SL-246`. Does the scope now assert nothing
  the accepted decisions contradict, and omit nothing `D6` / `D7` add?
- **`notes.md`, its harvest rewritten through round 3** (`git show ff0fdb02a`).
  It is this bar's source and may be wrong about what is worth probing.

### Bar

Anything real, any severity. Hold the standing admissibility rules: cite what you
attack, separate observation from prescription, and do not manufacture findings.

Round yields have gone **8 → 9 → 4**. That is convergence, and it means the
honest expected outcome of this round is *small*. **Raising nothing on the new
material, while verifying or contesting the five, is a legitimate and complete
outcome** and should be stated plainly if it is what you find. A fourth round
that manufactures four findings to match the third is worse than a fourth round
that raises none.

### Standing constraint

The design **must stand alone**. It may not require the review chronology, the
design run's state, or locally-invented terminology to be understood or
implemented. A place where it does is itself a finding.

### Out of scope, unchanged

Corpus hygiene (`IMP-403`), the `SPEC-019` record-kind gap (`ISS-316`), the
`IMP-457`/`IMP-393` overlap (closed as a duplicate), and implementation-level
code review — there is still no code.

## Round 5 bar — added 2026-09-18, at run revision 49

**Adjudication only.** Round 4's five findings are disposed and the baton is
back with the raiser. This round exists because the `review-disposition-attested`
contract will not take a pass while dispositions stand unadjudicated — it is the
mechanism closing, not a fifth adversarial pass.

Round 4 read **revision 47**. The design is now **revision 49**, four sections
moved — `sec-3`, `sec-5`, `sec-7`, `sec-9`. Read the current text, not the
disposition's account of it: `.doctrine/slice/246/design.md`.

### The job

`F-23`, `F-24`, `F-25`, `F-26`, `F-27` are `answered`. Verify each, or contest
it. Two are worth more than a read-through:

- **`F-23`'s repair is not the one you prescribed.** You offered threading
  `(TierFilter, EmptyPolicy)` up through `show_value` and the `format_metadata`
  chain, or giving the per-record layer all three markers at the cost of
  re-arguing `D6`. The second was taken and extended: `EmptyPolicy` is withdrawn
  outright, and `facet_fields`' return shape decides the empty state — `[]` is
  by-design, all-`Absent` is unfilled. Attack that. Does the shape actually
  distinguish the two states for every one of the seven record kinds, or is
  there a kind whose `facet_fields` is `[]` for a reason other than 'no facet by
  design'? And does anything outside the composed read now lose a marker it
  previously had?
- **`F-27`'s repair widens a rule rather than adding a case.** The partition is
  now scoped to flags that select content or projection. Check that the scope
  qualifier does not quietly excuse a flag that *should* be ruled.

The other three are prescription-taken repairs; read them against what you
raised.

### Bar

Raising new findings is **in scope but not the purpose**. If the repairs
introduced a defect, say so — rounds 3 and 4 both found that they had. If they
did not, adjudicating the five and saying the text is clean is the complete and
expected outcome.

### Standing constraint and out of scope

Unchanged from round 4.

## Synthesis

Seven rounds, 31 findings, all terminal — 29 verified, one withdrawn as a
mis-raise (`F-19`, re-raised as `F-20`), one contest upheld (`F-14`). Two
reviewers: rounds 1 and part of 2 on a GPT raiser, rounds 2-7 on Opus after
codex ran out of credits. Yields 8, 9, 4, 5, 2, 1.

### The closure story

The draft was sound in its decisions and wrong in its evidence. Round 1's two
blockers (`F-1`, `F-2`) were dissolved not by repair but by `DEC-261`, which
re-sited the whole read from `slice design show` onto `doctrine design show` —
the review's largest single change, and it came from attacking a governance
claim (`SPEC-013`'s two-level grammar) rather than the design's logic.

What the middle rounds found was a different genus: **the design kept asserting
mechanisms it had not established**. `F-14` claimed a guarantee from a return
type that carried one of three clauses. `F-18` specified a JSON shape with no
producer. `F-20` sited an empty-state policy on functions that could not see its
inputs. Those three were one defect — there was no per-record layer — and
dependency-ordering the triage is what surfaced it. `D6` is the result.

`D6` then produced `F-23`, because it carried the new policy parameters down to
the leaf renderers and to nothing above them, leaving the by-design marker
reachable at `Facets` and unreachable at `Full`. The repair withdrew
`EmptyPolicy` outright rather than threading it up — fewer moving parts than the
reviewer prescribed, and it made `C2` structural: `knowledge show` cannot reach
a layer that marks. That is the shape of the whole review in miniature — the
finding was right, its prescription was not the best repair, and the two were
adjudicated separately.

### The standing risk, and it is not in the design

**A repair inherits the finding's scope.** Four times a repair satisfied the arm
the finding named and left its twin — `F-13`/`F-18`, `F-23`, `F-30`, `F-31` —
and none was carelessness: every one was verified against the tree before its
disposition was written. The class was named in `F-30`'s disposition and *still*
not swept, which is how `F-31` was found: a reviewer took that sentence at its
word and asked where the fourth was. `X5` and `I6` were then fixed by sweeping
rather than by a further round.

The residual is that the sweep is the only evidence the class is closed. It was
performed at revision 59 over every edge case and invariant in §5.5; nothing
structural makes a two-arm claim state both arms, and nothing will notice if a
later edit states one. Recorded as
`mem.pattern.review.repair-inherits-finding-scope`.

Two process gaps were found and left unfixed, both recorded rather than
absorbed: the slice scope has no staleness signal against its design
(`IMP-461`, four instances in this review alone), and the reviewing runbook
stays discharged across adopt + materialise, so `review.passes` reported clear
while its own text — "written after the last pass" — was false.

### Tradeoffs consciously accepted

- **`I5`'s in-block half is verified by construction, not by test** (§9.5). The
  race it covers is unreachable in a black-box golden, and an injection seam
  would exist only to prove the shape. Three clauses, three separate grounds —
  the map forbids the drop, the never-empty contract forbids the empty render,
  the return type forbids the abort. The never-empty contract is prose; its two
  reachable floors are witnessed by §9.2 and only the scan/render race rests on
  intention. That is stated rather than papered over.
- **An `EVD` at `Only(Tier::Argument)` falls to the unfilled marker** — a
  wrong-ish message for a state nothing enters, since the filter is constructed
  nowhere. A fourth empty state would cost more than the named edge; the edge is
  named in §5.2 so a future caller is told where it is rather than finding it.
- **`knowledge show`'s concealing behaviour is preserved deliberately** (`C2`),
  and `format_facet`/`facet_json`'s existing disagreement about absent fields
  with it. Both are `IMP-403`'s.
- **The five committed memories documenting `design show` as the envelope read
  are routed to `/reviewing-memory`** (`CHR-073`), not to a phase — re-attesting
  a memory is its own verb and the corpus is not code. They must land with or
  before the code: a stale memory is injected into agent context.

### What this review did not reach

`notes.md`'s harvest was never audited on its own terms — it was read as a
source for round bars and its claims tested against the tree, which is not the
same check. §3.1's six three-level governance groups were verified at round 2
and not re-run. And nine section attestations remain outstanding under
`review_policy = human-only`; no round of this ledger discharges any of them,
and each binds a revision the adversarial lane never read.
