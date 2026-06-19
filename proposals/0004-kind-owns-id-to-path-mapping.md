---
seq: 0004
scope: backlog
target: IMP-067 (re-scope) + discovered corpus-wide duplication
confidence: high
reversible: yes (proposal only; no code or backlog transition performed)
---
## What
IMP-067 ("`dep_seq_for` SL arm should delegate slice toml path to the slice
module, mirror `outbound_for`") is correct but **under-scoped to one callsite**.
The actual defect is corpus-wide: the entity id→path formula
`<dir>/<NNN>/<stem>-<NNN>.{toml,md}` is rebuilt inline everywhere, with no kind
owning its own path mapping. Counted occurrences of just the `slice-`/`revision-`
literal form (`grep -Ec 'format!\("(slice|revision)-\{name\}\.(toml|md)"\)'`):

- `src/relation_graph.rs` — 2 (the SL arm `:57-64` **and** the REV arm `:73-77`;
  IMP-067 mentions only the first)
- `src/revision.rs` — 7 (`:393,495,504,608,835,852,933`)
- `src/slice.rs` — 8
- `src/review.rs` — 1
- `src/main.rs` — 6 (slice form)

≈24 hand-rolled copies of one formula, for two kinds — and the same shape recurs
for every other authored kind (adr/spec/policy/standard/…). Each copy is an
independent chance to drift the directory layout, and each is a place a new kind
must be remembered. This is the exact anti-pattern the codebase already named and
rejected elsewhere: `mem.pattern.entity.kind-is-data-not-trait` (cited in
`src/relation_graph.rs:6`), where `outbound_for`/`dep_seq_for` dispatch over
`integrity::KINDS` as *data*. Path construction never got the same treatment.

The data already exists: `integrity::KINDS` carries `dir` + `stem`
(`src/relation_graph.rs:307` comment; `entity::Kind`, `src/entity.rs:67`, carries
`dir`/`prefix`). A single data-driven helper closes all ≈24 sites at once —
strictly more leverage than IMP-067's one-arm fix, and it removes the REV arm
duplication IMP-067 doesn't even list.

## Options
1. **Execute IMP-067 as written** — delegate only the `dep_seq_for` SL arm to a
   `slice::toml_path`. Tradeoff: smallest diff, but leaves the REV arm and ≈22
   other copies untouched, and adds a *per-module* helper (slice has one, revision
   has one, …) — N helpers instead of one, a milder parallel-vocab smell.
2. **Re-scope IMP-067 → one corpus-wide `entity::id_path(kind, id, ext)`** driven
   by `integrity::KINDS` (dir+stem), then mechanically replace the ≈24 inline
   sites. Tradeoff: larger touch, but one source of truth for layout, kind-as-data
   consistent with `outbound_for`, and new kinds inherit pathing free. Slice-worthy.
3. **Leave IMP-067 narrow; file a separate "path-helper" improvement** for the
   corpus-wide version. Tradeoff: keeps IMP-067 honest to its title, but creates
   two overlapping items (the IMP-067 fix would be a strict subset of the new one) —
   the very duplication 0001 flagged in the backlog itself.

## Recommendation
Option 2: re-scope IMP-067 in place to the corpus-wide helper. The narrow version
is a strict subset; shipping it first means touching `dep_seq_for` twice (once for
the SL arm now, again when the helper lands) and leaving the REV arm — added later,
same smell — as a known untouched copy. One `entity::id_path` over `KINDS` is the
DRY-correct, kind-as-data move and matches the dispatch pattern already blessed in
this file.

Decisions deferred to YOU:
- (a) **re-scope IMP-067 vs. keep-narrow + new item** — i.e. does an existing
  backlog item get widened, or is its title contract sacred (file a sibling)?
- (b) **helper home & signature** — `entity::id_path(kind, id, Ext)` returning the
  relative or root-joined path; whether ext is an enum (`Toml`/`Md`) or `&str`.
- (c) **slice-worthy or quick-design** — ≈24 mechanical replacements + one helper +
  behaviour-preservation gate (existing suites stay green) reads slice-sized, but
  per boot.md Governance it could be a "small backlog item" quick-design.

## Next doctrine move
```
# confirm scope (read-only):
doctrine backlog show IMP-067
grep -rEn 'format!\("(slice|revision|adr|spec)-\{?name\}?\.(toml|md)"\)' src/

# then EITHER widen the existing item (re-scope, option 2) ...
doctrine backlog edit IMP-067 ...     # (verb shape per `doctrine backlog --help`)
# ... OR file the corpus-wide sibling (option 3):
doctrine backlog new improvement "Corpus-wide entity id->path helper \
  (entity::id_path over integrity::KINDS dir+stem) — absorb ~24 inline \
  <stem>-<NNN>.{toml,md} sites incl. dep_seq_for SL+REV arms (supersedes/widens \
  IMP-067)" --tag area:relations --tag area:entity --tag reuse
# if pursued as code:
/route                                # → /slice
```
(Verbs described, NOT executed — fence forbids backlog transition / authored edits.)

## Illustration (optional) — ILLUSTRATIVE, not applied
Hand-authored sketch of the helper shape (no worker spawned):
```rust
// src/entity.rs — one source of truth for the id->path formula.
pub(crate) enum Ext { Toml, Md }

pub(crate) fn id_path(root: &Path, kind: &Kind, id: u32, ext: Ext) -> PathBuf {
    let n = format!("{id:03}");
    let stem = kind.stem; // already on the KINDS row
    let file = match ext { Ext::Toml => format!("{stem}-{n}.toml"),
                           Ext::Md   => format!("{stem}-{n}.md") };
    root.join(kind.dir).join(&n).join(file)
}
// relation_graph.rs SL arm collapses to:
//   Ok((dep_seq::read(&entity::id_path(root, kind, id, Ext::Toml))?, false))
// and the REV arm becomes identical — the two arms differ only by `kind`.
```
