# Brief 04, thread A — verification note

Verification of `raw/thread-a-ownership.md` (pi-research, 2026-08-08) against
primary sources. The raw file is kept verbatim; corrections live here.

Study: brief 04 of the external design/proof pack
(`scratch/2026-08-08/04-proof-binding-adapters-brief.md`) — can Doctrine support
requirement → criterion → binding → evidence traceability without language or
test-framework semantics in core?

## Verdict

**Materially sound.** The strongest of the delegated threads so far: it ran the
positive controls it was asked for and reported them, and its central finding is
correct and was not on my list. Line cites drift ~10% (it cites `Step::material`
at 287-297, actual 324-335; `valid()` at 502-513, actual 536-547) — approximate,
but every fact I checked held.

## The central finding — confirmed, and it is the study's payload

**Doctrine already has definition-staleness. It is in the wrong subsystem.**

The runbook binds a discharge to the **digest of the step's definition**, not to
its id. `Step::material()` (`src/design_run/runbook.rs:324-335`):

```rust
framed(RUNBOOK_STEP_DIGEST_VERSION),   // "runbook-step.v1"  (:47)
framed(&self.id),
framed(&self.text),
framed(if self.required { "true" } else { "false" }),
framed(&argv.map_or(0, <[String]>::len).to_string()),
// … then each argv element, framed
```

`framed` is a netstring — `format!("{}:{value}", value.len())`, byte length
(`:340-342`) — chosen so no value can be mistaken for a delimiter, because `text`
may contain any byte and a separator join would let different definitions encode
identically. The encoding is version-tagged, so it can change later without
silently comparing incomparable bytes.

Result: edit a step's `text`, its `required` flag, or its verifier, and the
discharge goes **stale by construction**. `DEC-101`'s phrasing: *"An id solves
reference, not equivalence."*

**Coverage has no analogue.** Positive control: `digest` appears 34 times in
`src/design_run/runbook.rs` and **zero times in `src/coverage.rs`**.
`CoverageEntry` (`src/coverage.rs:55-74`) carries `key`, `status`, `git_anchor`,
`attested_date`, `touched_paths`, and `check: Option<VtCheck>` — the check recipe
is persisted **verbatim, undigested**.

So a coverage entry goes stale only when its *evidence's* git anchor ages against
`touched_paths`. **Change the binding definition — swap the matcher pattern,
change the command, retarget the alias — and the prior `Verified` status stands
unchallenged.**

That is exactly the correction case brief 04 § B asks about: *"replace the
criterion semantically and verify that old evidence does not automatically
satisfy the new active criterion."* Today it does.

## Also confirmed

- **`Verified` is unforgeable by the caller.** `DischargeOutcome` has three arms;
  a caller may say `attested` or `skipped` and never `verified`.
  `Discharge::verified()` is the sole route and requires an exit code and
  captured output — *"a verifier result is exactly the kind of fact Doctrine must
  derive rather than accept on a caller's word"* (`runbook.rs:71-73`). A step
  with no verifier therefore cannot render as verified. This is brief 04's
  VT-vs-VA/VH epistemic distinction, already built and enforced by construction
  rather than by convention.
- **argv, never a shell string** — *"`{repo_root}` can contain spaces and shell
  metacharacters, so substituting into a command string turns data into syntax"*.
  Placeholders are a closed four-element vocabulary interpolated per element so
  element count is preserved.
- **The scope fence is a typing argument.** Runbook steps are an OPEN vocabulary;
  `Condition` is CLOSED; *"narrowing a term from an OPEN vocabulary into a CLOSED
  one is a TYPE ERROR"*. `RunbookStanding` guards its own edge as a third derived
  input beside `ReviewStanding` and deliberately does not join
  `cumulative_conditions`. Directly relevant to whether project-supplied verifier
  adapters may satisfy Doctrine-owned criteria: on this precedent, they may not —
  they get their own standing instead.

## Independently established before the thread reported

Corroborating, reached separately:

1. **The phase-boundary gate executes nothing.** `src/vtgate.rs` does
   `source.contains(kw)` (`:130`) plus line-anchored regex. Positive control:
   `Command::new` appears in five other `src/` files, zero in `vtgate.rs`. So
   brief 04's *"what treats string presence as proof?"* is answered: the gate at
   the phase boundary does; the executing seam (SPEC-002) sits elsewhere, and the
   two share no identity wiring.

2. **The zero-test hole is real, scoped and flagged.** `derive_status`
   (`src/coverage.rs:481`): `Ran { exit_ok: true, matched: None } => Verified`.
   `valid()` (`:542`) makes a non-empty matcher mandatory **unless a literal
   command is set**, so literal-command entries may be exit-code-only and green
   on exit code alone; the verify report flags every such cell. Brief 04's
   constraint is therefore *partially* met with a bounded, documented residue —
   the study should propose closing the residue, not inventing the protection.

3. **SPEC-002's executable-verification requirements are title-only.**
   `REQ-254`–`REQ-257` carry no statement and no acceptance criteria, in either
   `spec req show` or the woven `spec show SPEC-002`. Positive control:
   `REQ-113`/`REQ-114`/`REQ-115` in the same spec carry full statements and 3–6
   acceptance criteria each. The thread reached this independently (its Unknown
   #2). **Brief 04's deliverable is a recommendation to reuse / extend / reject
   the SPEC-002 seam — and the seam's requirements presently assert nothing.**

4. **Two staleness models already exist; brief 04 proposes a third.** `REQ-115`:
   *"Decay VH/VA coverage attestations via the existing memory git-anchor
   staleness seam — surfaced, never auto-demoted."* Plus `DEC-101`'s digest
   binding. Any criterion/binding staleness proposal must reconcile with both.

## Consequence for brief 04's recommendation

The brief's framing — build a criterion/binding/evidence model — is the wrong
shape for what the corpus shows. Three of its four asks are already built:

| Brief 04 asks for | Incumbent |
|---|---|
| criterion identity distinct from test locator | `CoverageKey` 4-tuple vs `VtCheck` — already separate |
| binding revisable without criterion mutation | `check: Option<VtCheck>` is additive and replaceable |
| missing subject ≠ green | `Unobtainable → Blocked`; matcher mandatory except literal-command |
| VA/VH freshness distinct from VT | `REQ-115` git-anchor decay; `Verified` unforgeable |

What is genuinely absent is narrower and sharper than a new model:

1. **No digest binds a coverage entry to its check definition** — so binding
   edits do not invalidate evidence. The mechanism exists one subsystem over.
2. **The exit-code-only residue** on literal-command entries.
3. **No identity wiring between the two verification altitudes** — `vtgate`'s
   text mandate and coverage's executable check describe the same criterion and
   share no key.
4. **The governing requirements are empty**, so none of the above is
   conformance-checkable.

## Follow-up not yet captured

Finding 1 (no binding digest) and finding 3 (title-only `REQ-254`–`REQ-257`) are
each worth a backlog item. **Not minted yet** — a parallel thread is working the
same corpus and `backlog new` allocates the next free id, so concurrent mints can
race. File once the corpus has a single writer.
