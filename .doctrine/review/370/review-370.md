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
