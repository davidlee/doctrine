# IMP-317: History-stable scope surface for validate and retrieve

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

> **Heading corrected 2026-07-27 (SL-232 design).** This H1 still read "Route
> retrieve staleness through the shared claim-surface constructor" — the model
> RV-307 F-27 rejected — after F-39 limb 2 retitled the item and moved the slug.
> The retitle updated the queried `title` field and the slug symlink and missed
> the body's own title-shaped heading. F-39 named three sweep tiers (normative
> prose, routing-record bodies, queried metadata); a body H1 that mirrors a
> queried field is a **fourth**, and it is the one a reader of the *body* sees.

## Gate — OQ-2 is answered `yes` (2026-07-27, SL-232 design)

**QUE-175 / OQ-2** asked whether claim-surface drift feeds retrieve-side
staleness ranking or only `validate`. Measured on the live corpus at HEAD
`9f8cf40b`, 389 memories:

| | count | staleness mode |
|---|---|---|
| path-scoped + attested | 30 | commit mode — ranked by drift |
| **glob-only + attested** | **13** | **time mode — ranked by days since `reviewed`** |

**13 of 43 scoped-and-attested memories (30%)** are ranked on a 30-day timer
rather than by commits touching what they claim about, and they scope exactly the
fast-moving surfaces — `src/**`, `plugins/**`, `tests/**`,
`.claude/skills/dispatch*/**`, `src/worktree/**`. So this item does **not** close
`wont-do`, and SL-230's R7 is not restated as permanent.

## What remains here — the dataflow limb only

The measurement split this item in two. **SL-232 takes the cheap limb; this item
keeps the expensive one.**

**Moved to SL-232 (objective 4):** pass `scope.globs` alongside `scope.paths` and
neutralise pathspec magic before either reaches `commits_touching`. It needs no
`dir`, no provenance and no `collect_all` change — it is a two-argument change
that fixes the 13 mis-moded memories and closes the F-18 injection route into the
historical seam. It rides SL-232 objective 7, which is already at those call
sites.

**Retained here:** **own-directory drift** in the historical seam — counting
commits touching a memory's *own item directory* since `verified_sha`. This is
the limb that genuinely needs the dataflow change, and the two were bundled,
which is why adoption looked uniformly expensive.

## Why the retained limb is expensive (RV-307 F-28)

SL-230 round 5 claimed adoption would be a call-site swap because the constructor
takes `(root, memory, dir)`. F-28 falsified that: neither consumer has a `dir`.
`memory_health_findings(root, &[Memory], today)` (`src/memory.rs:3400`) receives
none, `run_validate` discards `_dir` (`:3480-3483`), and `collect_all`
(`:2826-2834`) unions `items/` and `shipped/` into one `Vec<Memory>` — so the
row's origin is not merely absent but unrecoverable from `uid`, which is why
`read_body` (`:2788-2797`) probes both roots. `retrieve::git_facts` and its caller
(`src/retrieve.rs:628`) are the same.

So the retained limb requires threading item-directory provenance through
`collect_all` and `memory_health_findings` — a dataflow change, not two arguments.

## The constraint that governs any surface built here (RV-307 F-27)

`verify`'s claim surface **must not** be reused. `verify` asks *is this evidence
dirty now*, where symlink resolution is mandatory — a pathspec naming a symlink
reads clean while its target is modified. `validate` and `retrieve` ask *how many
commits touched this since `verified_sha`*, where resolution is actively wrong: it
resolves against today's checkout and then queries yesterday-to-today history, so
a committed symlink retarget counts 1 through the declared path and 0 through the
resolved one (measured, git 2.54.0).

Note this constraint survives SL-232's **DEC-053** pivot unchanged. DEC-053
replaces `verify`'s construction with an index-first rule, but the reason the two
verbs cannot share a surface is about *history versus now*, not about which oracle
does the resolving.

## Provenance

Raised as RV-307 F-24 against SL-230's design; scope and cost corrected by F-27
and F-28; title corrected by F-39 limb 2; heading corrected and scope split by
SL-232's design round after OQ-2 was measured.
