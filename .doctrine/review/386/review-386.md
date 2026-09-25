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

## Third pass (2026-09-25)

Read the repairs in commits `0dca75b6f` and `c2bd07a6d` against the current
design, accepted `DEC-300`/`DEC-301`/`DEC-302`/`DEC-121`/`DEC-126`, the ten
design-run modules named in the review request, and the stored run snapshots.
The current snapshot set has 22 files; 20 hold `BlockingSetDeclared`. Normal
recording replaces declarations by act kind (`snapshot.rs:470-490`), so the
multiple declaration records in a run are different kinds, not competing
blocking sets. All observed stored `graph-reviewed` confirmation digests match
their corresponding declaration fingerprint; the stale-digest case still needs
the fixture in `VT-6`.

| finding | third-pass verdict | evidence |
| --- | --- | --- |
| `F-1` | Substance resolved; ledger remains `verified` | The accepted `DEC-301` choice and consequences now scope re-attestation to covered nodes. `design.md:14-20,276-278` agrees. No reopening verb exists; no TOML hand edit. |
| `F-6` | Verified | `design.md:48-70` gives sufficiency `InquiryMap` over carried material and concerns `ReviewedGraph` with full blocking membership. The two cumulative rows are distinct at `gate.rs:585-634`. `F-15` attacks the new projection, not this split. |
| `F-7` | Verified | `design.md:176-193` retains the deserialised legacy variants and `ConfirmationStale`, allowing the stored snapshots to parse. `F-16` addresses the separate write boundary this creates. |
| `F-8` | Verified | `design.md:195-203` resolves `Some(bool)` first, then the legacy set per unjudged node, preserving the open blockers in the `SL-252` snapshot after another node is judged. |
| `F-9` | Contested | The two kind/state rows in `design.md:113-120` solve admission of a value, but `payload_contract.rs:1398-1402` has one unqualified optional key and no way to render both homes; `submission.rs:168-171,521-524` still collapses `blocking: null` and omission into `Option::None` on an inquiry update. The repair specifies neither a context-aware contract representation nor a lossless wire read for the promised null refusal. |
| `F-10` | Verified | `design.md:132-140` seeds both import routes in `run.rs:1058-1101` with `blocking: true`, through constructors that require a judgement. |
| `F-11` | Contested | The revised slice still says `ActKind::BlockingSetDeclared` and `Cause::ConfirmationStale` retire at `slice-264.md:156-158`, then says the opposite at `:162-165`; its `R4` discussion at `:184-190` still explains the retired declared-set guard as current. |
| `F-12` | Contested | The accepted facets and consequences were amended, but `record-300.md:1` still says the derived re-declaration condition is required and `record-301.md:1` still states an unscoped move rule. `knowledge show` publishes these contradictory lines beside the accepted amendments. |
| `F-13` | Verified | `design.md:72-79,244-248` routes active acts through rule-specific `CoveredSet::moved`, so the gate and change log can agree for non-blocking additions. `F-17` addresses stored legacy acts that have no rule to look up. |
| `F-14` | Verified | `DEC-121`'s accepted choice and dated amendment now authorize one recorded user act with the agent's per-node judgement; `DEC-126` has the matching amendment. `design.md:166-174` states the departure explicitly. |

New findings on the repair: `F-15` (blocker, resolved blockers make
`ReviewedGraph` stale and can revive an unseen addition), `F-16` (blocker,
legacy blocking-set act remains writable), `F-17` (major, rule-based
`live_acts` has no coverage for a legacy declaration), and `F-18` (minor,
condition-row count contradicts the generated table). Their fixed details and
failure paths are in the ledger.

The carried confirmation check can be deterministic without another shell
digest: `CheckpointAct::confirms` and `AgentDeclaration::fingerprint` are stored
values (`attestation.rs:646-679`), while `DerivedInput.declaration_fingerprint`
is needed only when writing a declaration (`run.rs:160-169,584-607`). A stale
legacy confirmation stays stale until a new `GraphReviewed` replaces it; the
new single-act rule records no confirmation (`run.rs:654-670`). The old
confirmation also detects a legacy-set change made after an old act, whereas a
matching stored fingerprint permits the per-node fallback to use the current
set. `VT-6` needs its stated stale-digest fixture to pin that read.

The other named criteria have useful failure signals: `VT-1` distinguishes the
two coverage rules and the derived open-blocker check but misses `F-15`'s
add-then-resolve sequence; `VT-2` tests clearing and per-edge rows; `VT-3`
pins the old result before it flips; `VT-4` distinguishes covered from late
node moves; `VT-5` will expose the null-refusal gap if exercised through JSON;
`VT-6` covers parse and mixed legacy fallback but not the new write boundary;
`VT-7` covers both import creation routes. A covered resolved node whose
`blocking` mark is flipped also re-faces the user. That is an acceptable cost
of changing material the user reviewed (`design.md:123-129`), distinct from the
ordinary lifecycle transition in `F-15`.

## Response to the third pass (2026-09-25)

All seven points accepted on evidence and answered on the ledger.

- **`F-15`.** One effective judgement, two reads: the *marks* (any lifecycle),
  which `ReviewedGraph` compares, and the *open blockers*, which the derived
  row counts. Answering a question neither stales the review nor revives it.
- **`F-16`, `F-17` — one class.** The design had assumed every `ActKind` has a
  contract row. Legacy kinds are now named (`ActKind::is_legacy`), the
  one-row-per-kind test states the exception, the write path refuses them
  (`RetiredAct`), and `live_acts` excludes them by stated rule.
- **`F-9`.** `blocking` parses as `Sparse<bool>`; `KeyContract` carries its
  home, so the contract renders the key once per home.
- **`F-11`, `F-12`, `F-18`.** Slice document, decision prose and the condition
  count brought in line (nine rows; the dropped proposal would have been the
  tenth). The decision-prose edits bring summaries into line with amendments
  the user already assented to; no new position is recorded.
