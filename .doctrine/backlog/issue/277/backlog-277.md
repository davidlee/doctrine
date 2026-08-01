# ISS-277: reseat cannot renumber a review — strict Meta read requires derived status

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

```
$ doctrine reseat RV-320 --to 322
Error: Failed to parse .doctrine/review/320/review-320.toml
  0: RV-320: TOML parse failed
  1: TOML parse error at line 1, column 1
     missing field `status`
```

Structural, not a data fault: **`reseat` can never renumber any review.**

## Cause

`reseat` reads the alias slug via the **strict** meta reader —
`meta::read_meta` (`src/integrity.rs:295`) → `dtoml::parse_entity_toml`, which
requires a `status` field. A review's status is **derived from its findings**
(ADR-007 D-C8) and is therefore deliberately never stored; the review toml says
so in a comment.

The fix is narrow: `reseat` needs only `.slug`, which reviews **do** carry.

## The telling part

`src/integrity.rs:132-133` already records the same problem being solved for a
different verb:

> …reader so review's intentionally status-less toml scans cleanly, while the
> strict `Meta` (status-bearing readers) is untouched.

SL-151 gave `scan_kind` (behind `validate`) an id-only reader for exactly this
reason and **left `reseat` on the strict path**. So the class was known, the
remedy was known, and one call site was missed — a one-verb blind spot rather
than an unrecognised defect.

## Suggested fix

Read the slug through the same lenient/id-only reader `scan_kind` uses, or make
the status field optional for kinds that derive it. Add a reseat test over a
status-less kind — `scan_kind_reads_a_review_statusless_toml`
(`src/integrity.rs:563`) is the existing pattern to mirror.

## Impact / workaround

Hit while renumbering SL-237's design review after an id collision (RV-320 was
allocated concurrently in two worktrees — see obs `019fa925-c961`). Worked around
by hand: move the entity dir, rename both files, edit `id` + the `.md` heading,
replace the alias symlink, move the gitignored `.doctrine/state/review/<id>/`,
and re-point citing artefacts.

**The hand path is exactly what `reseat` exists to de-risk.** It requires
distinguishing citations of *this* entity from citations of a *different* entity
that happened to share the id — here, a memory
(`mem_019fa97aa54671f2b572e83ae2923dc4`, sourced `RV-320` + `SL-233`) and three
observation records all referred to the *other* RV-320 and had to be left alone.
A naive global replace would have corrupted them silently.

## Adjacent defect: the dangler report is `.md`-only

Separable from the parse failure, and it would bite even once the strict-read fix
lands. `scan_danglers` (`src/integrity.rs`) globs `.doctrine/**/*.md`, so the
inbound citations it reports are prose only. **Relation edges live in `.toml`**
and are never scanned: rehoming `RV-323` → `RV-340` (2026-08-01, second instance
of this issue) had to sweep six memories carrying `[[source]] kind = "review"` /
`ref = "RV-323"` and one `plan.toml` entry by hand, none of which `reseat` would
have named. Since the report is the *only* thing standing between the operator
and a silent dangling reference, an `.md`-only sweep understates the work in a
way that reads as completeness.

Cheap fix: glob both extensions in `scan_danglers`, or drive the sweep off the
relation graph for the `.toml` half and keep `line_cites` for prose.
