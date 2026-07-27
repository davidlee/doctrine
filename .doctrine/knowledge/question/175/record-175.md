# QUE-175: Should claim-surface drift feed retrieve-side staleness ranking

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The question

SL-230 builds a claim-surface constructor for `verify` — the canonicalised uid
directory plus a memory's declared, sanitised scope entries. Should the two
*historical* consumers of a memory's scope — `validate`'s staleness check and
`retrieve::git_facts`'s ranking input — be given an equivalent, **history-stable**
surface?

This is SL-230 design **OQ-2**, promoted to a durable record because it gates
more than one decision:

1. the original: should own-directory drift feed retrieve-side `staleness`, or
   only `validate`?
2. added by RV-307 F-24: should `retrieve::git_facts` (`src/retrieve.rs:556-557`)
   leave the raw scope seam at all?
3. added by RV-307 F-27: `validate` is in the same position, so this is now a
   question about *both* historical consumers, not just `retrieve`.

They are one question by several routes — each reclassifies a large fraction of
the corpus at once and shifts staleness or retrieval ordering broadly.

**Corrected by RV-307 F-27/F-28/F-34.** This record originally described SL-230 as
adopting *one shared constructor* across `verify` and `validate`, and adoption
here as a cheap call-site swap. Both were wrong and the design has been re-cut:

- `verify` asks *is this evidence dirty now*, where canonicalisation is
  mandatory. A historical query asks *what commits touched it since*, where
  canonicalising against today's checkout **erases** a committed symlink retarget
  (measured, git 2.54.0: `rev-list -- link` → 1, over the resolved target → 0).
  So the answer is not "reuse `verify`'s surface" — it is "build a second,
  history-stable one".
- adoption is **not** a call-site swap. Neither consumer has an item directory to
  pass, and `collect_all` (`src/memory.rs:2826-2834`) unions `items/` and
  `shipped/`, so the row's origin is unrecoverable from `uid`. It needs a dataflow
  change through `collect_all` and `memory_health_findings`.

## Why it is open rather than deferred-and-forgotten

`git_facts` today gates on `m.scope.paths.is_empty()` and passes `scope.paths`
raw: no globs, no canonicalisation, no pathspec-magic neutralisation, no uid
path. Every defect RV-307 found in `verify`'s surface is still live there. So
"leave it alone" is not a neutral choice — it means two notions of scoped drift
coexist and **the weaker one drives ranking** (SL-230 R7).

The same is now true of `validate`, which SL-230 round 4 intended to repair and
round 6 returned to the raw seam (RV-307 F-27). So the unrepaired population is
two consumers, not one.

SL-230 declined to answer it inside a body-write slice: converting either
consumer would smuggle a staleness/ordering change in under cover of a bug fix.
The bound is honest about its cost rather than cheap — see the correction above.

## What answering it decides

- **Yes** → implement **IMP-317**; R7 closes.
- **No** → close IMP-317 as `wont-do` and restate R7 as *intended and permanent*
  rather than provisional, with the divergence documented at the `git_facts`
  call site so the next reviewer does not re-raise F-24.

Either answer is fine; leaving it unanswered is the failure mode, because the
gap currently reads as an oversight rather than a decision.

## Evidence to gather before answering

- how many memories change retrieval rank if drift is measured over a repaired
  surface (the SL-230 census machinery answers this: of 417 addressable memories
  and 482 path/glob declarations, 404 are observable and 43 do not resolve);
- whether glob-only memories — invisible to `git_facts` today — are a material
  population;
- whether ranking already treats `Staleness::Unknown` conservatively enough that
  the change is small in practice.

Related: SL-230 (design OQ-2, D11, R7), IMP-317, QUE-173 (the digest-based
alternative, which would make the whole question git-independent).

---

## Answered — `yes` (2026-07-27, SL-232 design round)

Answered on measurement, not argument. Corpus at HEAD `9f8cf40b`, 389 memories,
59 attested:

| | count | staleness mode |
|---|---|---|
| path-scoped + attested | 30 | commit mode — ranked by drift |
| **glob-only + attested** | **13** | **time mode — days since `reviewed`** |

**13 of 43 scoped-and-attested memories — 30% — are ranked by calendar rather
than by commits touching their evidence**, and they scope the fastest-moving
surfaces in the repo: `src/**`, `plugins/**`, `tests/**`, `src/worktree/**`,
`.claude/skills/dispatch*/**`. So the answer to the record's second evidence
question ("are glob-only memories a material population?") is **yes**, and the
"No" branch — close IMP-317 `wont-do`, restate R7 as permanent — is refuted.

### The third evidence question is answered too, and it came out the other way

*"Whether ranking already treats `Staleness::Unknown` conservatively enough that
the change is small in practice."* Checked directly, expecting to find that
`GitFacts::default()` ("never asked") and a failed probe ("cannot determine")
collide as the same `None`. **They do not.** `staleness()` branch 1
(`src/retrieve.rs:371`) is guarded by `!m.scope.paths.is_empty() &&
!verified_sha.is_empty()` — the *same* predicate `git_facts` gates on — so a
default `None` can never reach the branch that reads `None` as `Unknown`.
`retrieve` is correct on this axis, and so is `src/coverage.rs:150-166`
(`None => Unknown`).

That leaves **`validate` as the only one of three consumers** that mishandles the
seam, which is ISS-257 / RV-307 F-36, absorbed into SL-232 (see DEC-054). It also
supplies that work's continuation policy from precedent rather than invention:
`git_facts`'s documented contract is *per-candidate failure, never a query abort*
(review B18).

### The answer splits IMP-317 rather than triggering it whole

The measurement showed the item bundled two changes of very different cost:

- **(a) taken in SL-232, objective 4** — pass `scope.globs` alongside
  `scope.paths` and neutralise pathspec magic before either reaches
  `commits_touching`. No `dir`, no provenance, no `collect_all` change. Fixes the
  13 mis-moded memories and closes the F-18 injection route into the historical
  seam.
- **(b) retained as IMP-317** — own-directory drift, which is the limb that
  genuinely needs item-directory provenance threaded through `collect_all` and
  `memory_health_findings` (F-28's dataflow change).

**R7 therefore closes partially, not wholly**, and the F-27 constraint governs
(a): `verify`'s surface is still not reused, because the two verbs differ on
*history versus now*. That constraint survives SL-232's DEC-053 index-first pivot
unchanged — DEC-053 changes which oracle resolves, not whether resolution belongs
in a historical query.

The record's first evidence question — how many memories change *retrieval rank*
— is deliberately **not** answered here. It is a property of an implementation
that does not exist yet, and answering it against a mock would be the
design-time-absolute trap RV-313 F-1 caught (a design figure of 11-of-30 that
failed to reproduce as 3-of-48 purely through corpus growth). It belongs in
SL-232's verification evidence, measured against the real change.
