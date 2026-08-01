# ISS-292: reseat moves the entity but not its citations, and its dangler report misses the structured tier

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

`doctrine reseat` renames the entity directory, sets `id` in the TOML, and swaps
the alias symlink — then reports inbound citations and `bail!`s without
rewriting them (`src/integrity.rs::run_reseat`, deliberate per D4/R-3). That
part is by design. The defect is that **its report cannot be used as the
rewrite worklist**, and nothing says so.

Measured on a real move — `DEC-099 → DEC-105` in `dispatch/233`, 2026-08-02,
resolving one of the six `ISS-279` collisions. Reseat reported 26 citations
across 4 paths. The truth was 24 occurrences across 6 tracked files.

## Four independent faults in `scan_danglers`

1. **It scans `.doctrine/**/*.md` only, so the entire structured tier is
   invisible.** Missed on this move: `record-092.toml`'s `superseded_by` and
   `record-100.toml`'s `supersedes` — the two *relation* edges that actually
   bind the supersession chain — plus 7 in `review-324.toml` and 4 in
   `plan.toml`. A reseat followed faithfully by its own report leaves the
   relation graph pointing at a vacant id.

2. **It never leaves `.doctrine/`,** so `src/commands/design.rs:1496` was
   missed. Doctrine's own source cites decision ids in comments; so does
   `scripts/` on `edge` (`spike-capsule/lib/common.sh`,
   `fixtures/*/interpretation-surface.txt`).

3. **It double-counts through slug alias symlinks.** The glob walks both
   `.doctrine/slice/233/` and `.doctrine/slice/233-cli-managed-design-runs-and-inquiry-maps/`
   — the same files twice. 26 reported hits were 13 real ones. Doctrine mints
   these aliases itself, so every reseat report on any kind is inflated ~2×.

4. **The disposability filter is defeated by a symlink.** `is_disposable_prose`
   tests for adjacent `.doctrine` / `state` path *components*, but the runtime
   phase sheets are reached as `.doctrine/slice/233/phases/phase-15.md` through
   the gitignored `phases` symlink — no `state` component in that path. Ten
   gitignored, `rm -rf`-able runtime lines were reported as citations a human
   must rewrite. The comment above the function names exactly this tier as the
   thing it exists to suppress.

Faults 3 and 4 are worse than noise: they pad the list with items that need no
action, which trains the operator to skim the list that also silently omits the
load-bearing ones.

## What the operator must do instead

Build the worklist from `git grep -l -w '<REF>'` over tracked files, rewrite,
then use reseat's output only as a cross-check. That is what was done for
`DEC-099`.

## Fix directions (not a design)

- Widen the scan to the authored tiers that actually carry refs — `*.toml` as
  well as `*.md`, and the repo outside `.doctrine/` (source comments, scripts,
  shipped assets) — or state the scan's bound in the output so it is not read as
  exhaustive.
- Canonicalise paths and de-duplicate before reporting, so alias symlinks
  collapse.
- Make disposability a resolved-path test, not a component test, so the `phases`
  symlink does not defeat it.
- Consider whether the structured tier should be rewritten automatically:
  `supersedes` / `superseded_by` / relation edges are machine-written by
  `doctrine supersede` and `doctrine link`, so leaving them to a human contradicts
  the verb that wrote them. D4/R-3's "prose relations are outbound-only" is a
  reason not to rewrite *prose*; it does not cover the structured tier.

## Links

- `ISS-279` — the collision this reseat was resolving; reservation reach is
  local, so two trees allocate the same id unwarned.
- `ISS-277` — `reseat` cannot renumber a review at all (strict meta reader wants
  a `status` reviews never store, ADR-007 D-C8). Same command, different fault.
- `SL-233` `notes.md` `## Harvest` → `### Open` — the six-id ruling and the
  five moves still outstanding on `edge`.
