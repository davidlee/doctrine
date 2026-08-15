# IMP-432: next lacks kind, tag and status filters

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`doctrine next` has no filters. Its whole surface is `--format`, `--json`,
`--path`, `--columns`, `--limit`, `--offset`, `--page`, `--verbose`. `kind` is
available as a *column*, but there is no way to say "the actionable frontier,
issues and improvements only".

Compare `backlog list`, which carries `--kind`, `-t/--tag`, `-s/--status`,
`-f/--filter`, `-r/--regexp`, and `-a/--all` (the `CommonListArgs` spine plus
backlog-local additions).

## Why it matters

Surfaced during SL-238's design (2026-08-15) as the cause of a design pressure,
not merely as a missing convenience.

`backlog list --by sequence` composes a backlog work order from `needs`/`after`
edges but does not gate on them, while `next` gates but cannot be narrowed to a
kind. That gap made it tempting to push the gating role onto `--by sequence` —
which would have made it a worse `next` and destroyed the one thing it uniquely
does (show blocked work *in dependency position*, which `next` structurally
cannot, because it drops exactly those rows).

SL-238 settled the division of labour instead:

- `next` — actionable frontier: cross-kind, score-ranked, drops blocked rows.
- `backlog list --by sequence` — backlog-scoped work order: shows everything
  including blocked rows, in dependency position.

That split is only complete once `next` can be narrowed. Until then, "the
actionable backlog" is not expressible in one command, and the pressure to
overload `--by sequence` recurs.

## Scope

- `--kind` on `next`, accepting cross-kind prefixes (not just the five backlog
  kinds) since `next` is a cross-kind surface — so the value vocabulary is the
  `KINDS` table, not `ItemKind`.
- `--tag` and `--status`, for parity with the shared list spine.
- Decide whether these ride `CommonListArgs` (`src/main.rs:143-184`) or stay
  local to `next`. `next` is not a `listing.rs` consumer today, so this is a
  real design question, not a mechanical lift — it is the reason this is an
  improvement rather than a chore.

## Out of scope

Anything about `backlog list --by sequence`'s own semantics — that is SL-238.
