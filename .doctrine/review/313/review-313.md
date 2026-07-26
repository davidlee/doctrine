# Review RV-313 — reconciliation of SL-230

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed.** Not the raw evidence refs. This audit ran against the
candidate interaction branch `candidate/230/review-001` = **no-ff 3-way merge of
base `refs/heads/main` @ `c2e9191a2` (carrying landed SL-228) with source
`refs/heads/review/230` @ `66d478cc2`**, merge oid `8eb22ea80`, admitted to this
review as `review_surface`. The base is deliberately post-SL-228: the operator
asked that SL-230 be audited against the trunk it will actually land on, not the
13-commit-older trunk it forked from. Both slices touch `src/mcp_server/tools.rs`
and the merge is clean.

The surface then advanced to **`18fc9961`** — the F-3 fix-now repair, committed on
the candidate (never on `review/*` or `phase/*`, which stay immutable per I9/R2)
and re-admitted. `admit` permits this by design: it requires the recorded merge be
an *ancestor of* the admitted tip, not equal to it.

**Lines of attack.**

1. **The four residuals the slice disclosed about itself** — weigh them, do not
   merely re-narrate them. A slice that discloses its own weak points has earned
   the benefit of the doubt on candour, not on correctness.
2. **I10 and I13, hardest.** Both are load-bearing and both were introduced *by
   review rounds* (RV-307 F-3 and the confirming pass) rather than by the original
   design — that is exactly the profile of an invariant asserted but not pinned.
   Neither is accepted on the strength of a passing test: each is mutation-checked
   here, and a mutation that does not compile does not count as a check.
3. **Does the shipped check actually fire?** PHASE-06's own falsifier cannot
   distinguish "correctly silent" from "silently inert". Re-measure the live
   corpus independently rather than reading the worker's figure.
4. **Criterion arithmetic.** VA-1 carries an absolute figure from a corpus that
   has since quadrupled. Recompute; treat a number that no longer reproduces as
   something to restate, never to reinterpret at read time.
5. **The governing spec, read directly.** SPEC-007 governs this slice
   (`references(implements)`). Read its staleness contract against the shipped
   behaviour rather than assuming phase criteria exhaust the obligations.
6. **Mechanical conformance first** (`slice conformance`), so prose reading is
   spent only where the algebra cannot reach.

## Synthesis

**SL-230 is sound and closeable.** Six findings, all terminal: one fixed in
audit, one routed to owned work, one to governance, one accepted with a forced
rationale, one restated, one upheld as correctly decided. No blockers.

The implementation matches its design unusually faithfully — `run_edit` is the
ordered function § 5.4 describes, step for step; the MCP arm really is argument
mapping only, gaining no call to `write_body`/`resolve_body`; Check 4 really is an
independent block guarded on `verified_sha` alone with a repo-relative
`memory.md` pathspec. `slice conformance` is clean in all three cells (0
undeclared, 0 undelivered, 6 conformant), so there is no scope creep and no
dropped declared work.

**Evidence gathered, not inherited.** `doctrine check gate` ran against the slice
code for the first time in its life — the drive's standing gates were per-phase
`check prove` / `check regression diff`. It is green: **4928 passed, 0 failed,
102 suites, zero compiler or clippy diagnostics** (the 19 `warning:` lines in the
log are doctrine's own runtime warning *strings*, emitted by tests exercising
warning paths — none carries a `-->` source pointer; a naive grep would have
misreported the gate as dirty, and did on the first pass).

Both hard invariants are now mutation-verified rather than asserted:

| invariant | mutation applied | result |
|---|---|---|
| **I10** — rejected edit leaves both tiers byte-identical | hoist `write_body` ahead of `apply_edit` (the pre-RV-307-F-3 order) | **T40 red** — `clobbered\n` on disk |
| **I13** — a stamp's own commit is never counted as drift | pathspec `memory.md` → item directory | **T42 red *alone***; the other two validate tests stayed green |

The "red *alone*" property is what separates a real falsifier from one that fires
on any perturbation, and it held.

**VA-1's property survives; its number does not.** Re-measured on the primary
tree (HEAD `46c4eac83`, 2026-07-27): the shipped `memory.md` pathspec flags **3 of
48** reachable-anchored memories; the rejected item-directory form flags **25 of
48** — an **8.3×** discrimination (the drive reported 7.7× over 44). Saturation,
the falsifier the design feared, is decisively absent. The criterion's *absolute*
expectation (~11 of 30) does not reproduce, in the safe direction, because the
anchored corpus grew 30 → 115 between design and execution. `validate` on the
candidate binary emits exactly 3 own-body findings, matching the independent
count — the check is live, not inert.

**One audit hypothesis failed honestly and is recorded as such.** Check 4 is
guarded on `verified_sha` alone where Check 2 also requires non-empty
`scope.paths`, so it necessarily runs `commits_touching` over strictly more rows;
audit predicted a measurable cost. Measured: **74s pre-slice vs 72s candidate**.
No cost. The 74s baseline is pre-existing and belongs to neither this slice nor
this audit.

### The one thing the slice did not find about itself

Every one of the four carried residuals was real, and all four are dispositioned.
But the audit's most consequential finding came from reading the *governing spec*
rather than the slice's disclosures. SPEC-007 § "Git-anchored staleness" promises
that every undecidable reachability case — **naming "non-ancestor anchor"
explicitly** — resolves to an explicit state, "never a silent hide or a silent
over-trust". Both staleness checks do the forbidden thing on 67 of 115 anchored
memories, and the product already implements the correct shape one module away
(`src/coverage.rs:150-166`, `IsStale::{Fresh,Stale,Unknown}` with a documented
`None ⇒ Unknown` contract over the *same* seam).

That reframes F-2 from "an unstated reach limit" to a probable spec-conformance
gap, and it is why F-2 (→ ISS-257) and F-6 (→ REV) are carried as two findings
against two write surfaces rather than collapsed into one. The defect is *not* in
`commits_touching`'s ancestry guard, which is correct and must stay — it is in the
two call sites, where `let Some(n) = … && n > 0` lets `None` fall out and emit
nothing.

### Standing risks and consciously accepted tradeoffs

- **Corpus reach is ~42%, not 100%.** Owned by ISS-257; the conformance question
  by the SPEC-007 REV. Pre-existing in Check 2 — SL-230 inherited it into Check 4
  rather than introducing it — which is why it does not block this slice.
- **`clear_verification` is tolerant where `stamp_verification` refuses** (F-4).
  Accepted, and the rationale is *forced* rather than convenient: step 4 sits
  after the body write, so a fallible clear would reintroduce exactly the
  mutation-on-failure that I10 forbids and that this audit proved T40 catches.
  The narrow residual — a hand-corrupted `[review]`/`[git]` table silently keeping
  a stale attestation — is now stated rather than latent.
- **`memory edit` is not atomic across its two tiers.** A crash between the body
  write and the TOML write leaves a changed body with a stale `updated`, never the
  reverse (R1). Design § 5.4 states this and rejects full two-tier atomicity as
  disproportionate; audit agrees and re-endorses it explicitly.
- **The MCP adapter rejects no unknown fields anywhere** — `deny_unknown_fields`
  appears zero times in `src/mcp_server/tools.rs`. The F-3 fix closes the one
  instance that mattered (`body_mode` on `record`) without closing the class. Left
  unfiled pending an operator decision; noted here so it is not lost.
- **DEC-027, OQ-4, OQ-7, R4** carried forward unsettled, deliberately.

### For `/close` — one non-obvious sourcing constraint

**The close candidate must NOT be sourced from `review/230`.** The F-3 repair
(`18fc9961`) lives only on `candidate/230/review-001`; `review/230` is still
`66d478cc2` and does not carry it. A `close_target` candidate created with the
default source would silently ship without the fix. Source it from the admitted
review surface, and expect the known authored divergence — `slice-230.toml` reads
`started` on the evidence refs and has since advanced on `edge` — to need
primary-wins at integration.

## Reconciliation Brief

Handoff to `/reconcile`. Grouped by the write surface reconcile will actually
touch. Every non-aligned finding that touches design or governance appears here.

### Per-slice (direct edit)

- **`design.md` § 5.4 — restate the D5 pathspec measurement (F-1).** The section
  carries "fires on **30 of 30**" (directory form) and "drops it to **11 of 30**"
  (shipped form), measured against a 30-memory anchored corpus. Both figures are
  stale. Replace with the audit measurement, carrying its denominator, date and
  tree so it cannot silently rot again: on primary HEAD `46c4eac83` (2026-07-27),
  over 48 reachable-anchored memories, the shipped `memory.md` form flags **3**
  and the item-directory form flags **25** — **8.3×**. Add the sentence that does
  the durable work: *the property D5 relies on is the discrimination ratio, not
  the absolute count* — the count is a function of corpus age and mix.
  **Do NOT edit `plan.toml`.** VA-1 (and every `EN-`/`EX-`/`VT-` id) is
  immutable-append; the stale prose VA-1 derives from is the design, and that is
  the only surface this item touches.

- **`design.md` § 5.5 — append an edge case for `clear_verification`'s tolerance
  (F-4).** § 5.4 asserts the attestation is cleared "iff a claim field genuinely
  changed"; that holds only for well-formed TOML. Record that a missing or
  non-table `[review]`/`[git]` is silently skipped, so a hand-corrupted file keeps
  a stale attestation through a genuine claim change — and that the tolerance is
  *forced* by the step order I10 depends on, not chosen. Use the **next free id
  `E15`** (per `notes.md` § Open); `E1`–`E14` are immutable and moved ids are
  never reused.

- **`design.md` § 5.4 — state the corpus reach limit (F-2).** § 1's closure
  promise reads corpus-wide, and nothing in the slice qualifies it. Add the
  measured cap — 67 of 115 anchored memories carry a `verified_sha` that is not an
  ancestor of HEAD, so both staleness checks are silently inert on them and
  reach is ~42% — and point to **ISS-257** as its owner. This is a *stated
  residual*, in the same voice § 5.4 already uses for the TOML-hand-edit residual.

- **`notes.md` § Harvest — bring current.** Sinks: RV-313 (6 findings, all
  terminal) under Produced; **ISS-257** minted; the F-3 fix at `18fc9961` and the
  fact that it lives on the candidate, not `review/230`; `check gate` green for
  the first time against slice code (4928/0); the I10 and I13 mutation results.
  § Next is now stale — its step 1 (remove the coord worktree, flip to audit) is
  done.

### Governance/spec (REV)

- **SPEC-007 § "Git-anchored staleness" — adjudicate the explicit-state guarantee
  against `memory validate` (F-6).** The sentence at
  `.doctrine/spec/tech/007/spec-007.md:106-116` is currently unqualified and names
  "non-ancestor anchor" explicitly, while `memory_health_findings` Checks 2 and 4
  emit nothing in that case. Two mutually exclusive resolutions; the REV must pick
  one, and audit deliberately did not.

  **Audit's recommendation, revised on a closer reading: resolve the ambiguity,
  do not adjudicate conformance.** The first reading of this finding leaned to (b)
  below on the strength of the sentence being unqualified and naming the exact
  failing case. Two textual facts weaken that and are recorded here so reconcile
  weighs them rather than inheriting the stronger claim:
  - the state vocabulary the sentence promises — `fresh / stale / unknown /
    **unanchored** / **reference**` — is `retrieve`/render vocabulary. `validate`
    emits *findings*, not states, and has no `unanchored`/`reference` to render.
  - "never a silent **hide**" most naturally denotes hiding a memory *from
    results*, which is a retrieval concern. Only "silent over-trust" is
    surface-independent.

  What still supports (b) is structural, not lexical: "Git-anchored staleness" is
  a **peer section under `## Responsibilities`**, not a subsection of "The
  scope-aware reader" that precedes it — so it reads as an engine-wide
  responsibility rather than a property of the reader.

  The honest conclusion is that **the sentence does not determine its own scope**,
  and that ambiguity is the defect worth fixing. Preferred REV: amend SPEC-007 to
  state explicitly which surfaces the guarantee binds. That is cheap, uncontested,
  and leaves **ISS-257 standing on its own merits** — a silent "no drift" on 58% of
  the anchored corpus is worth fixing whether or not it is formally non-conformant.

  The two readings, if the REV prefers to settle rather than clarify:
  - **(b)** — the guarantee binds any git-anchored staleness computation.
    `validate` is recorded as non-conformant and **ISS-257 becomes a conformance
    fix** rather than an improvement. Note the cost: this retro-labels shipped
    behaviour non-conformant on a contested reading.
  - **(a)** — the guarantee is scoped to the `find`/`retrieve` axis. SPEC-007
    gains the qualifier it currently lacks, and ISS-257 stays an improvement.

  Under all three routes the *code* outcome is identical — ISS-257 is the fix, and
  the conforming shape already exists over the same seam at
  `src/coverage.rs:150-166`. What actually moves is ISS-257's priority and the
  spec's clarity, not what gets written.

- **No REQ status changes identified.** No requirement was verified or falsified
  by this slice's evidence; recorded explicitly so reconcile does not re-derive it.

### Not in scope for reconcile

- The MCP adapter's blanket unknown-field tolerance (Synthesis, standing risks) —
  a backlog item if the operator wants it, not a reconcile write.
- **DEC-027 / OQ-4 / OQ-7 / R4** — carried forward unsettled by explicit
  instruction. Reconcile must not settle them.
