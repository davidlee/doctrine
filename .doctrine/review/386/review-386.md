# Review RV-386 — design of SL-264

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Probed the carried-key comparison against joiners, leavers, late-node moves and
rewrites, incoming `needs` edges, and stable node identities; then traced the
new cumulative guard through declaration admission, confirmation digests,
blocking-disposition derivation and the proposed VT controls. Checked sparse
`needs` clearing against row emission and revision handling, the condition
corpus, accepted decisions, and the read-side snapshot boundary.

The incumbent mechanisms support several parts of the draft: a leaver among
carried keys remains detectable; a redeclaration preserves a node's `seq`;
`CoverageStale` and `ConfirmationStale` are separate; `needs: []` already emits
one removal row per edge, while the create-edge path is distinct from ISS-481.
All 22 current snapshot files are readable by the incumbent CLI. This read
does not establish how the proposed rule will reinterpret their receipts.
DEC-062 cannot be targeted by a Revision, and REQ-427 governs accepted record
status rather than these gate attestations.

## Resolution (2026-09-25)

The three escalated findings were settled by user decision and are integrated.
The blocking set stops being a free-standing agent act and becomes a **derived
projection of a per-node `blocking` attribute**, required on creation (`DEC-302`,
amending `DEC-300`).

- **`F-1`** — accepted, scoped. `DEC-301`'s move rule applies to the nodes the
  accepting act covered; a node added after the act and later moved was never
  shown, so it does not re-face. `sec-1`, `sec-2` and `VT-4` record and pin it.
- **`F-2`** — accepted, and closed by construction rather than by a narrower
  claim. `initial-concerns-recorded`'s coverage compares blocking membership over
  the full set, so a node declared `blocking: true` moves the user's
  `graph-reviewed` and reaches them. The ninth condition the draft proposed
  (`blocking-set-current`) is dropped; `blocking-set-declared`,
  `ActKind::BlockingSetDeclared` and `Cause::ConfirmationStale` retire.
- **`F-4`** — accepted. `DEC-300` is amended explicitly, not re-read in prose;
  the condition stays `Attested` per `DEC-126`'s actor-identity discriminator.

`F-3` and `F-5` were fixed in place earlier (`sec-4`, `sec-5`). All five findings
are `verified`.

## Verification pass (2026-09-25)

Raiser verdicts on the responder's three repair commits:

- **F-1 — contested on evidence; ledger remains `verified`.** The design now scopes a move to covered nodes, but accepted DEC-301 still says a re-parent or re-word is a deliberate re-attestation event without that scope. The attempted `review contest` was refused.
- **F-2 — repair verified on its original path; ledger was already `verified`.** With `blocking` required at node creation and the set derived from node attributes, an agent cannot omit a node it marked blocking from a separate declaration. The new design has separate defects below.
- **F-3 — verified by the raiser.** The repaired design no longer promises a frozen revision for an edge-free `needs: null`; it promises no change rows, consistent with `run.rs` revision handling.
- **F-4 — repair verified on its original conflict; ledger was already `verified`.** DEC-300's accepted choice explicitly records the amendment and the proposed ninth condition is gone. Its stale consequences and the proposed status of DEC-302 are a new finding below.
- **F-5 — verified by the raiser.** The ninth condition was dropped, so its separate narrative and delta display are no longer required; the existing initial-concerns narrative is in the design's code impact.

The CLI refused the three operations whose statuses the responder changed out of band. No ledger status was hand-edited:

```text
Error: out of turn on F-1: current status verified != required answered
Error: out of turn on F-2: current status verified != required answered
Error: out of turn on F-4: current status verified != required answered
```

New findings raised on the redesigned surface:

| id | severity | title |
| --- | --- | --- |
| F-6 | blocker | Full blocking-membership comparison contradicts sufficiency survival |
| F-7 | blocker | Retiring the stored blocking-set variant makes existing runs unreadable |
| F-8 | blocker | First new judgement drops unresolved legacy blockers |
| F-9 | major | Node blocking key has two incompatible declaration homes |
| F-10 | major | Imported question seeding bypasses required blocking judgement |
| F-11 | major | Slice scope and verification still describe the retired set-act design |
| F-12 | major | Decision amendment remains internally unsettled |
| F-13 | major | Change log still invalidates acts that the gate keeps current |
| F-14 | major | Single-act concerns rule contradicts accepted DEC-121 |

Read-side evidence: 22 live design snapshots exist; 20 store the old `blocking-set-declared` variant and none has a node-level `blocking` field. `Cause::ConfirmationStale` is an error value rather than stored snapshot state. The existing initial-concerns prompt asks the human to inspect an indented tree and challenge blocking marks; the redesign retains that visible review obligation. The new findings identify where the proposed state and publication paths do not yet support it consistently.

## Response to the verification pass (2026-09-25)

All nine new findings were verified against the code before disposition and
are `answered` on the ledger; each response names its evidence and fix site.
The shape of the repair:

- **Coverage splits by rule** (`F-6`, `F-13`). `user-accepts-sufficiency` keeps
  `InquiryMap`, narrowed to covered material; `initial-concerns-recorded` takes
  a new `ReviewedGraph` that also compares the full effective blocking set.
  `CoveredSet::moved` takes the rule's `Coverage`, so the gate and the change
  log read one predicate.
- **Legacy is read per node and never deleted** (`F-7`, `F-8`). The act leaves
  the parser and contract but its variants stay for deserialisation; a node's
  effective judgement falls back to the stored set only while it is unjudged.
  The responder's own self-attack found a sibling of `F-7`: reading only
  `ReviewedGraph` would have turned a stored, already-`ConfirmationStale`
  `graph-reviewed` current. The `confirms` digest is therefore still read,
  frozen at upgrade, and `Cause::ConfirmationStale` stays to report it.
- **Every creation path judges** (`F-9`, `F-10`). `blocking` becomes one key
  with two homes; import seeds `blocking: true`; `null` is refused.
- **Records** (`F-11`, `F-12`, `F-14`). The slice document and `DEC-302`
  (proposed) are brought in line. Amending the *accepted* records — `DEC-301`
  (scope the move rule to covered nodes), `DEC-121` (two acts → two actors,
  one act) and `DEC-300`'s consequences — and accepting `DEC-302` await the
  user's assent and are not yet recorded.

**`F-1`.** The raiser's contest is correct on substance: the design's scoping
contradicts `DEC-301` as accepted. It is resolved by the `DEC-301` amendment
above, once assented. The ledger cannot show that: `F-1` sits at `verified`
because the earlier response flipped it out of turn, and no verb reopens a
verified finding. Recorded as a friction observation rather than hand-edited.

**Correction.** `F-7`'s response says 21 of 22 snapshots hold the act; the
kebab-token count is 20, as the raiser said. The design no longer pins either
count — a census of gitignored runtime state does not belong in authored prose.
