# Corpus-aware memory verify gate

## Context

Split out of **SL-230** by **DEC-027** at RV-307 round 8. SL-230 retains the
memory body-write seam and attestation invalidation; this slice takes the
`verify` gate and the SPEC-007 amendment its contract change requires.

The split is not a fresh start. This slice inherits a design that survived
**eight adversarial rounds and 29 findings** on the gate alone — the decisions,
the measured censuses, and the reasoning are carried over intact in `design.md`.
What it does not inherit is the constraint that made the last two rounds
unproductive: the gate was being designed inside a slice whose charter could not
accommodate the schema change its own decisions kept pointing at.

### The problem

`memory verify` refuses on any dirty working tree. Doing doctrine work means the
authored corpus is essentially always dirty, so verifying a memory is routinely
blocked by edits that say nothing about whether its claim still holds. In
practice agents hit the refusal and reach for `git stash` rather than
`--allow-dirty` (undiscoverable, and shipped after IMP-221 was filed).

The naive fix — ignore doctrine's own authored trees — is wrong, and RV-307 F-1
and F-6 proved it: memory items live at `.doctrine/memory/items/<key>/`, so a
blanket exclusion removes **the memory being verified**, stamping a commit that
provably lacks the attested body. And **81 items in this corpus declare
`.doctrine/**` scopes**, so an ADR a memory explicitly names is claim evidence
exactly as `src/` is.

So the gate must be **claim-aware**: exclude unclaimed dirt, never claimed
evidence. That requires constructing, per memory, the surface its attestation is
actually about — which is where the difficulty lives.

### Why this needed its own slice

Two round-8 blockers, neither a text defect, established that the claim surface
is a larger problem than a body-write slice could carry:

- **RV-307 F-36** — DEC-020 requires `validate` to raise every non-contributing
  scope entry, but D11 leaves `validate` with no contribution probe: it keeps a
  historical, `scope.paths`-gated seam that cannot implement it. Supplying one is
  an undesigned second per-entry git path plus a corpus-wide continuation policy.
- **RV-307 F-37** — the premise that a non-resolving entry contributes nothing is
  **false**. Reproduced on git 2.54.0 by three routes: `missing/../link`, a
  sparse checkout, and a `scope.paths` literal whose filename contains `*`. Each
  contributes while bypassing canonicalisation and reads clean against a dirty
  target, restoring the false attestation I9 exists to close.

Both were open at the split. **F-37 is now answered** (see objective 2); F-36 is
answered in shape by objective 7 and remains open in detail.

### What changed after the split

Three facts postdate DEC-027 and this scope's first draft. They are recorded here
because the inherited `design.md` predates all three:

- **REV-041 amended SPEC-007** (approved, done), on RV-313 F-6 from SL-230's
  reconciliation audit. The five-state resolution is now explicitly the *render
  contract*, binding `find`/`retrieve`; and the **prohibition on silent
  over-trust is surface-independent**, binding any git-anchored staleness
  computation "including `memory validate`'s health checks" — a surface emitting
  findings "discharges this by emitting a finding, not by falling silent."
- **ISS-257** (RV-313 F-2) — `memory_health_findings` Checks 2 and 4 bind
  `commits_touching` in a let-chain, so a non-ancestor `verified_sha` yields
  `None`, falls out, and emits nothing. *Cannot determine* renders as *no drift*
  on **67 of 115 anchored memories**, capping corpus reach at 41.7%. Pre-existing
  in Check 2; SL-230's Check 4 inherited it. **Absorbed into this slice** —
  `SL-232 fulfils ISS-257` — because it and F-36 are the same defect: `validate`
  not knowing something and saying nothing. See objective 7.
- **The census re-measured** (HEAD `9f8cf40b`), which retires several inherited
  assumptions. Figures are stamped with their HEAD deliberately: RV-313 F-1 was a
  denominator-drift finding, where a design-time absolute (11 of 30) failed to
  reproduce at execution (3 of 48) purely because the corpus grew.

## Scope & Objectives

### 1. Claim-aware dirty-tree gate for `verify`

- The clean/dirty decision ignores modifications confined to doctrine's own
  authored trees — `.doctrine/**` and the repo-root `memory/` tree — **except**
  the memory's own item directory and its declared scopes, which are the evidence
  the attestation is about.
- `verify` therefore asks two questions: is the **unclaimed** tree dirty, and is
  the **observable** claim committed?
- The exclusion set derives from doctrine-owned path constants, never a hardcoded
  literal list (POL-002, STD-001).
- A dirty **source** tree still refuses; `--allow-dirty` remains the escape hatch
  with its `checkout_state_id` stamping unchanged, taken from an *unexcluded*
  capture (RV-307 F-13).

### 2. The claim-surface constructor — index-first

The inherited ordered algorithm is **replaced, not patched**. It classified an
entry's shape from its *characters*, canonicalised it with `realpath`, and then
asked git a question about the index. F-37's three routes are all one defect:
**a filesystem oracle answering an index question.** Reproduced on git 2.54.0,
and the shape matrix shows the two instruments are uncorrelated in both
directions — `missing/../link` and a sparse-checkout entry fail `realpath -e` and
still contribute; `linkdir/target.txt` resolves cleanly and contributes nothing.

The replacement never leaves the index:

- **Emit by field of origin, never by character.** `scope.paths` → `:(literal)`,
  `scope.globs` → `:(glob)`. The schema already records the distinction that
  step 2 was re-deriving unreliably; this is F-37's structural correction and
  F-32's contest answered at the root rather than at the split point.
- **Expand against the index** — `git ls-files -s -z`. Non-empty match set →
  contributes; empty → non-contributing (objective 7's sink). `-z` is required,
  not stylistic: `core.quotePath=true` renders `ünï.txt` as `"\303\274n\303\257.txt"`
  and corrupts any parsed output.
- **Resolve symlinks from the index blob, not the filesystem** — every match of
  mode `120000` has its target read via `cat-file`, joined lexically, and
  re-expanded, bounded and cycle-checked. This is what closes the sparse-checkout
  route: `cat-file blob :<path>` returns the target *while the file is absent
  from the working tree*.
- **Guard aborts lexically, before emission.** Escaping and absolute-outside
  entries are rejected without git ever seeing them, so `exit 128` becomes
  unreachable rather than handled. This dissolves E13's stated basis — its
  justification was mechanical necessity, and the mechanism is now preventable.
- Scope entries remain **untrusted data, never pathspec syntax** (F-18); the
  magic prefix rule is unchanged and I8 is untouched.

**Stated honestly:** on this corpus the symlink-resolution step adds **zero**
coverage for declared scopes (25 entries match symlinks, all self-covering; 0
entries are symlink-*rooted*). It is insurance against a reproduced-but-not-live
hazard. It *is* live and load-bearing for the **uid directory base**, which is
reached through one of 347 key symlinks (F-15). The whole resolve pass over
`.doctrine/**` — 7,670 matches, 2,071 symlinks — costs **7ms**, so scale is not
a constraint.

### 3. Non-contribution reporting, and the declared-boundary question

- Per **DEC-020**, `verify` attests over every non-contributing entry and reports
  each on stderr; `validate` raises them as corpus-health findings. No
  classification: three derived instruments were refuted because each reads local
  repository state (F-21, F-25, F-31).
- **`validate` needs a contribution mechanism it does not have** (F-36). Designed
  under objective 7, unified with ISS-257.
- **The declared boundary** — a persisted signal marking evidence git is not
  expected to observe — is the stable answer DEC-020 deferred. This is the
  objective SL-230 structurally could not hold, and it is why this slice exists.

**The census gives it a measured population** (HEAD `9f8cf40b`): of 440 scope
entries across 389 memories, **59 are non-contributing**, and they sort exactly
along the boundary DEC-020 refuted three instruments for —

| bucket | count |
|---|---|
| ordinary path — moved, deleted, or never existed | 39 |
| harness / installed-tree (`.claude/**`, `.harness/**`) — git can never see these in a source checkout | 18 |
| runtime state, gitignored by design | 2 |

So the boundary is real and stable; it simply cannot be *derived* from local
repository state. Declaring it turns 59 undifferentiated reports into **39
actionable findings**.

**Shape:** a parallel assertion — entries stay in `scope.paths` / `scope.globs`
and are additionally named as expected-unobservable. Rejected: a sigil inside the
entry string (character-sniffing, the exact error objective 2 deletes, and it
collides with real filenames); a per-memory flag (too coarse — the typical
memory here declares several paths and one unobservable); a separate `external`
list (answers "is this part of the claim?" with *no*, which is wrong — if the
path ever becomes tracked it should be measured). The chosen shape is
**falsifiable**: a declaration that git then contradicts is itself a stale
declaration and a finding, so the boundary is self-policing rather than a
permanent silence. It never subtracts from the claim surface, so I8 holds.

**OQ-A is answered `no`.** DEC-020 argued the declared boundary, IMP-318's
attested coverage and QUE-173's body digest are one schema change. Having seen
the field shapes: the boundary is an **authored input**, the other two are
**machine-written outputs of a verify run**. Different writers, different
lifecycles, different validation. Sharing a TOML file is not sharing a change.
IMP-318 and QUE-173 stay routed and are **not** built here.

### 4. Historical scope consumers — decide, don't inherit

**Three `validate` concerns are distinct and the inherited text blurs them.**
Only the first two unify (objective 7); this objective is the third alone:

| concern | question | owner |
|---|---|---|
| ISS-257 — non-ancestor anchor renders as clean | *can I determine drift?* | objective 7 |
| F-36 — no contribution probe | *can I observe this evidence?* | objective 7 |
| R7 — raw, `paths`-gated scope seam | *which paths do I ask about?* | (a) here · (b) IMP-317 |

- `validate`'s staleness check and `retrieve::git_facts` both keep a raw
  `scope.paths`-gated seam (R7, F-19/F-24/F-27/F-28). They ask a *historical*
  question where canonicalising against today's checkout erases a committed
  symlink retarget (measured 1 → 0), so they need a **second, history-stable**
  surface rather than reuse of `verify`'s.
- **D11 and R7 are falsified as written.** D11 says `validate` "keeps its
  existing raw seam"; that cannot survive objective 7, which must touch the same
  two call sites. R7's four-defect enumeration is incomplete — the `None`-swallow
  is a fifth, the largest by population, and the only one that is
  **non-conformant** against amended SPEC-007 rather than merely weak. Both are
  restated in the design, not silently inherited.
**OQ-2 / QUE-175 is answered `yes`, on measurement** (HEAD `9f8cf40b`). Of 389
memories, 59 are attested; **30** are path-scoped (commit mode, ranked by drift)
and **13 are glob-only** — 30% of scoped-and-attested memories ranked on a 30-day
timer instead of by commits touching their evidence, and they scope precisely the
fast-moving surfaces (`src/**`, `plugins/**`, `tests/**`, `src/worktree/**`,
`.claude/skills/dispatch*/**`). IMP-317 therefore does **not** close `wont-do`,
and R7 is not restated as permanent.

The measurement also **splits IMP-317**, which bundled two changes of very
different cost:

- **(a) taken here** — pass `scope.globs` alongside `scope.paths` and neutralise
  pathspec magic before either reaches `commits_touching`. No `dir`, no
  provenance, no `collect_all` change: a two-argument change that fixes the 13
  mis-moded memories and closes the F-18 injection route into the historical
  seam. It rides objective 7, which is already at those call sites.
- **(b) routed to IMP-317** — own-directory drift in the historical seam. This is
  the limb that needs item-directory provenance threaded through `collect_all`
  and `memory_health_findings`: a dataflow change, not two arguments (F-28).

**The F-27 constraint governs (a) and survives DEC-053 unchanged**: `verify`'s
surface must not be reused, because the two verbs differ on *history versus now*,
not on which oracle resolves. (a) is deliberately *not* a shared surface — it
widens the raw seam's input and neutralises it, and resolves nothing.

### 5. SPEC-007 reconciliation — REV-034

The retired "clean working tree, refusing a dirty one" contract lives at three
sites: `spec-007.toml:22`, `spec-007.md:132-133`, and **REQ-147**, whose title is
that contract verbatim (RV-307 F-5). The implementation already diverged when
`--allow-dirty` shipped; this slice changes it further. Applied at close so spec
and code turn over together. Moved here from SL-230 by DEC-027 — the contract is
changed by the gate, not by body-write.

**The inventory was drawn before objective 7 existed and must be re-taken.**
`validate` appears in SPEC-007 exactly **once** — the sentence REV-041 added.
This slice gives the verb a contribution probe, a reporting contract and a
continuation policy against a spec surface that barely names it. Whether that is
an added REV-034 change row or a second revision is settled during design, but it
is not deferrable to close: REQ-147's title is the retired contract verbatim, and
the same trap (a queried surface asserting what the body replaced, RV-307 F-39)
applies to any requirement this objective leaves unamended.

### 6. Verify refusal names its escape hatch

The dirty-tree refusal names `--allow-dirty` instead of prescribing a commit.

### 7. `validate` renders every undeterminable explicitly (absorbs ISS-257)

**One mechanism, two unknowns.** `validate` currently has two ways of not knowing
something and saying nothing, and REV-041 forbids both:

| unknown | today | population |
|---|---|---|
| non-ancestor `verified_sha` → `commits_touching` = `None` | falls out of the let-chain, no finding | **67 of 115** anchored memories |
| scope entry contributes nothing | never probed at all | **59** entries across 55 memories |

Building these as two mechanisms would implement the same epistemic honesty
twice — the parallel implementation A1 rejected, one level up. The unifying
obligation is already approved governance: *"A surface that emits findings rather
than states discharges this by emitting a finding, not by falling silent."*

- **ISS-257's remedy** is a tri-state at the two call sites (`src/memory.rs`
  Checks 2 and 4) — drift / no-drift / undeterminable. The ancestry guard in
  `commits_touching` (`src/git.rs:2493`) is **correct and stays**; the defect is
  in how callers consume `None`. The correct shape already exists one module
  away: `src/coverage.rs:150-166` defines `IsStale{Fresh,Stale,Unknown}` over the
  *same* seam with the contract `None => Unknown`. Ride it; do not re-invent it.
- **F-36's remedy** is a per-entry contribution probe on a corpus-wide verb, plus
  the **continuation policy** F-29 identified — what the run does when one
  memory's surface cannot be probed. A corpus-wide verb must not abort the corpus
  for one bad row.
- Objective 3's declared boundary is what keeps this reporting *actionable*
  rather than 59 lines of noise.

**This makes SL-232 a `validate` slice as much as a `verify` slice.** Stated
plainly because the title does not say so.

## Non-Goals

- **The memory body-write verbs.** SL-230's, and already designed and reviewed
  there — this slice must not re-open them.
- **Attestation invalidation on claim-field edit** (D4/D8/D5). Also SL-230's.
  Note the coupling: SL-230 ships invalidation *without* this slice's relaxation,
  so R4 (mass re-verification friction) runs unmitigated until this lands. That
  is DEC-027's accepted tradeoff and the reason to sequence this next.
- **Loosening `thread_expiry`** or any retrieval-side gate — reviewed canon
  (SL-008 D6).
- **An MCP `memory_verify` tool.** SL-164's stated rationale dissolves once the
  clean-tree precondition relaxes, but re-litigating that exclusion is its own
  decision.
- **Masters coverage.** `verify` is items-only by construction (E2); masters are
  unanchored and `collect_all` never scans them (R5). QUE-173's digest is what
  would reach them, and it is not built here.
- **IMP-318 (persist attested coverage) and QUE-173 (body digest).** DEC-020
  argued these travel with the declared boundary; objective 3 answers that `no`
  on the authored-input/machine-output split. They stay routed. R8 — an
  attestation not recording what it covered — therefore remains **open** after
  this slice, and objective 3 does not close it.

## Risks, assumptions, open questions

- **R-A — discharged in method, narrowed in residue.** The enumerate-then-probe
  obligation was met: three routes reproduced, a shape matrix run over lexical
  aliases, sparse checkout, `skip-worktree`, `assume-unchanged`, literal glob
  metacharacters, symlink chains and topology, `core.quotePath`,
  `core.ignoreCase`, and control characters. Objective 2 then makes most of that
  taxonomy **non-load-bearing**: the shapes are no longer classified by us, so a
  fourth unenumerated shape has nothing to break. Two residuals survive and are
  named rather than closed by assertion — R-E and R-F below.
- **R-E — index bits suppress the measurement itself, and no pathspec approach
  can close it.** A tracked file marked `skip-worktree` (`S`) or
  `assume-unchanged` (`h`) reads `diff-index` exit 0 while modified on disk. No
  symlink, no pattern, no resolution — the instrument is locally disabled. It is
  **detectable** (`git ls-files -v` carries the flag on the row) and it affects
  the *source* leg as well as the claim surface, so it is wider than this slice.
  Live population in this repo: **0 rows** (HEAD `9f8cf40b`) — latent, not live,
  and pre-existing rather than introduced. Named because a slice whose purpose is
  closing false-attestation routes cannot leave a known one unstated.
- **R-F — case-insensitive collision is unmeasured, not cleared.** `core.ignoreCase`
  alone did not make pathspec matching case-insensitive on this filesystem
  (exit 1 either way). A genuinely case-insensitive filesystem (APFS, NTFS) could
  not be probed from here. Carried as unmeasured; not claimed closed.
- **R-C — R4 runs unmitigated** while SL-230 is shipped and this is not.
- **R-G — absorbing ISS-257 widens the blast radius to a seam no criterion of
  this slice originally governed.** `memory_health_findings` is consumed
  corpus-wide; the behaviour-preservation gate applies (existing suites green
  unchanged), and the tri-state must not convert a silent exemption into a noisy
  one for the 67 rows that were previously invisible.
- **OQ-A — answered `no`** (objective 3): the declared boundary is an authored
  input; IMP-318 and QUE-173 are machine-written outputs. Sequenced, not merged.
- **OQ-2 / QUE-175 — answered `yes` on measurement** (objective 4): 13 of 43
  scoped-and-attested memories are glob-only and ranked by calendar rather than
  drift. IMP-317 splits — (a) globs + magic-neutralisation taken here, (b) the
  own-directory dataflow change routed. QUE-175 is settled by this slice.
  (Cited by the design's own id; the pre-rescope text called this `OQ-B`, which
  collided with `design.md` § 6's `OQ-B` — a different question.)
- **OQ-B (design § 6)** — `validate`'s contribution mechanism and its
  continuation policy. Answered in shape by objective 7; open in detail.

## Summary

## Follow-Ups
