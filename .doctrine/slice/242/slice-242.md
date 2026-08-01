# Retire the projected reference-doc model

## Context

SL-227 (governed by ADR-019) made projection minimal: `doctrine install` eagerly
projects only `.gitignore`, `doctrine.toml`, and `project-orientation.md`
(`install/manifest.toml` `[base] backings`). Every other embedded asset is
**published** — reachable on demand via `doctrine library show`, never copied to
disk.

The delivery mechanism changed; the corpus that describes it did not. Surfaced by
the SL-229 audit as IMP-315 (RV-306 F-7) and inventoried on 2026-08-01 across the
four surfaces that actually reach a client — shipped memory (`memory/`), skills
(`plugins/doctrine/skills/`), install assets (`install/**`), and boot generation.

The 80/20 landed in `ddd138ab` ahead of this slice: the boot sector now states the
resolution rule once (docs are cited bare as `<name>.md`, are published not
projected, and are read with `doctrine library show reference/<name>.md`); the two
skills asserting `.doctrine/review-ledger.md` were corrected; five templates
dropped the `.doctrine/` prefix from their `glossary.md` citation; and four master
headers stopped claiming an inert installed copy exists.

What remains is the corpus that still teaches the retired model, and the tracked
residue that model left behind in this repo.

### What is still wrong

1. **The shipped memory corpus still teaches the retired model.** It is embedded
   in the binary and pushed into every client's retrieval path, so it outranks
   the docs it describes.
   - `mem.signpost.doctrine.reference-docs` — the canonical "how to find a
     reference doc" signpost. Body: "ships two reference documents to every
     installed project under `.doctrine/` … they install once and stay inert
     unless the installer is re-run." False in every clause. Its own
     `scope.paths` are `.doctrine/using-doctrine.md`, `.doctrine/glossary.md` —
     retrieval anchors pointing at files that cannot exist.
   - `mem_019f2b93f5e178009191711f607caff6` — `scope.paths` `.doctrine/dispatch-mechanics.md`.
   - `mem_019e9a11cda27db19c0c75bafa453d5d` (file map) — body lists
     `.doctrine/using-doctrine.md` / `.doctrine/glossary.md` as shipped reference docs.
   - `mem_019ec92b0fff76e1935b222348938d7f` (install signpost) — `scope.paths` and
     body instruct the user to "add a reference in `.doctrine/governance.md`",
     which is now created-on-need rather than seeded.
   - SL-143 overhauled this corpus and is `done`; it predates ADR-019, so these
     have no current owner.

2. **Nine tracked orphans in this repo.** `.doctrine/*.md` copies are unprojected,
   unrefreshable, and drifting from their `install/` masters — measured
   2026-08-01: `dispatch-mechanics` 231 divergent lines, `using-doctrine` 93,
   `glossary` 70, `routing-process` 56, `review-ledger` 27, `model-band` 6,
   `governance` 127 (by design — user-owned), `harvest` and `boot-footer` current.
   `.doctrine/glossary.md` is missing the entire knowledge-record kind table.
   They are correct *enough* not to announce themselves as stale, which is the
   worst failure mode: the SL-229 auditor read one on the boot digest's
   instruction.

3. **The residue is wider than the nine, and CHR-043 already named it.** Every
   `.doctrine/` path CHR-043 lists is tracked and unignored today — the nine
   reference-doc copies plus `agents/`, `templates/`, `hymns/`, `workflows/`,
   `mod.just`, `doctrine.toml.example`, `rules/AGENTS.md`. These are install
   output committed as if authored, so they arrive in every diff and review as
   noise. (Three of CHR-043's entries — `.claude/workflows`, `.claude/agents`,
   `.pi/skills` — are already ignored; the item is partly stale.)

4. **`install/manifest.toml:56`** points readers at `install/using-doctrine.md` —
   an `install/` path, which exists only in this build repo.

5. **Nothing detects any of this.** `doctrine doctor` has no currency or liveness
   check for a reference-doc citation. Its only reference-doc awareness is
   `prose_cite` noise suppression for `glossary.md` (IMP-252). Without a check,
   the corpus re-rots on the next delivery change — this slice's own fixes
   included.

6. **Existing clients have no removal path — and are not this slice's problem.**
   A repo installed before ADR-019 keeps its stale copies forever: `install` is
   write-if-absent and idempotent by contract, so it will never remove them.
   Fixing that needs a once-per-repo, version-keyed migration primitive that
   doctrine does not have. **Explicitly deferred** (see Out of scope, IMP-378) —
   this slice cleans this repo and stops the corpus lying; it does not build a
   mechanism to reach back into installed clients.

### Sibling concerns, deliberately not merged

- **IDE-030** is the general client-side form: any framework-owned doc that
  arrived under write-if-absent install goes stale forever after an upgrade. It
  needs the migration primitive this slice declines to build, so it stays filed
  and unenabled.
- **SL-144** (`ready`, 0/5) audits the `install/*.md` set as an *information
  architecture* — content overlaps, gaps, restate-line compliance, currency of
  `glossary.md`/`using-doctrine.md`. That is about what the docs *say*; this
  slice is about how they are *reached*. Its scope reconciliation is dated
  2026-07-03 and therefore predates ADR-019. See OQ-2.
- **DEC-010** ("published set = full projection complement") is `proposed` and
  out of date with SL-227 — it must be settled or superseded here, since it is
  the record of the very question this slice answers.

## Scope & Objectives

### Objectives

1. **Re-anchor the shipped memory corpus on the published model.** Correct the
   four memories above — bodies *and* `scope.paths`, since a path anchored to a
   non-existent file silently degrades retrieval. Sweep the corpus for any other
   assertion that a reference doc lands on disk; the four are an inventory, not a
   proof of exhaustiveness.

2. **Settle DEC-010** against the shipped SL-227 behaviour — update or supersede,
   so the decision record matches `[base] backings`.

3. **Untrack the projection residue.** Freshen the gitignore contract so install
   output stops being committed as authored state, and untrack what is already in
   (`git rm --cached` — a gitignore entry alone does nothing to a tracked file).
   The nine stale reference-doc copies go with it; `governance.md` stays tracked,
   being user-owned and boot-read.

   The edit belongs in **`install/manifest.toml` `[gitignore] entries`** — the
   shipped contract every client inherits, which already carries
   `.doctrine/skills/*` — with this repo's `.gitignore` following from it. A
   local-only `.gitignore` edit would fix this repo and leave every client
   accumulating the same residue.

   Each path must be classified **derived vs user-overlay before** it is ignored
   — see R1.

4. **Add a doctor check** that fails on a reference-doc citation which cannot
   resolve — a bare `<name>.md` with no published counterpart, or any surviving
   `.doctrine/<name>.md` / `install/<name>.md` path assertion in a shipped
   surface. This is the regression control that keeps objectives 1–3 from
   re-rotting.

5. **Close CHR-043** — objective 3 actions it. Its three already-ignored entries
   are dropped as stale; the rest land in the shipped contract.

### In scope

- `memory/` — the shipped corpus: bodies and `scope.paths` of the affected
  memories, plus the sweep for others.
- `install/manifest.toml` — `[gitignore] entries` (the shipped contract), and the
  `install/` path in the comment at line 56.
- `.gitignore` — this repo's, following from the shipped contract.
- `src/doctor_checks.rs` (+ wherever the check registry lives) — the new check.
- The tracked projection residue under `.doctrine/` — untracked, not deleted from
  disk (it is regenerable install output; `governance.md` retained and tracked).
- DEC-010; CHR-043; IMP-315 (closes them).

### Out of scope

- **Any migration mechanism**, and with it any removal path for stale copies in
  *already-installed client repos*. A once-per-repo, version-keyed primitive is
  the right answer and is deliberately not built here — filed as **IMP-378**.
  This slice's untracking is a this-repo hygiene move plus a forward-looking
  contract change, not a retroactive fix.
- **Reference-doc *content*.** Currency of `glossary.md` / `using-doctrine.md`,
  IA overlaps, restate-line compliance — SL-144 owns these. This slice does not
  edit what a reference doc says, only how it is reached and described.
- **The general write-if-absent currency sweep** across all framework-owned
  client assets — IDE-030, still filed and now unenabled.
- **Changing the projection/publication split.** ADR-019 is accurate and is a
  premise, not a target. `governance.md` being created-on-need is settled.
- **`boot-footer.md`'s retirement** — SL-144 objective 2 owns the deletion of the
  dead asset. This slice untracks its *projected copy* along with the other
  residue; whether the master survives is SL-144's call.
- **New entity kinds, CLI verbs, or memory-engine changes.**

## Non-Goals

- Deleting the untracked residue from disk. Untracking removes it from review and
  from the authored tier; it stays regenerable install output. Anyone can `rm` it.
- Rewriting every bare `<name>.md` citation across ~20 skills to carry the
  `library show` command. The boot sector states the rule once (`ddd138ab`);
  duplicating it per citation is the parallel-implementation failure.

## Affected Surface

- `memory/mem_019ec92b10037850817507044f0f99ef/` (`mem.signpost.doctrine.reference-docs`)
- `memory/mem_019f2b93f5e178009191711f607caff6/`
- `memory/mem_019e9a11cda27db19c0c75bafa453d5d/` (file map)
- `memory/mem_019ec92b0fff76e1935b222348938d7f/` (install signpost)
- `install/manifest.toml` — `[gitignore] entries` + the line-56 comment
- `.gitignore`
- `src/doctor_checks.rs`, and the check-registry seam it is wired through
- The tracked residue under `.doctrine/` — the nine `*.md` copies plus `agents/`,
  `templates/`, `hymns/`, `workflows/`, `mod.just`, `doctrine.toml.example`,
  `rules/AGENTS.md`; untracked subject to the R1 verdict.
  `.doctrine/governance.md` retained and tracked — user-owned.

## Risks & Assumptions

- **R1 — Ignoring a user-overlay surface.** The residue is not uniformly derived.
  `.doctrine/hymns/` is documented as a *user overlay* of the prompt cascade, and
  `.doctrine/templates/` is plausibly customisable per project. Gitignoring a
  path a user is invited to edit silently makes their customisation untracked —
  they lose it on the next clean, with no diff to warn them. Every path in
  objective 3 needs a derived-vs-overlay verdict, sourced from the install leg
  that writes it, before it is ignored. This is the sharpest risk in the slice.
- **R2 — Untracking is not gitignoring.** A `.gitignore` entry has no effect on
  an already-tracked file. Without `git rm --cached`, objective 3 is a silent
  no-op for all sixteen paths — they are every one of them tracked today.
- **R3 — Doctor-check false positives.** A bare `<name>.md` in prose that is not
  a reference-doc citation (a filename in an example, a slice's own notes) must
  not fail the check. IMP-252 already shows this surface generates noise.
- **R4 — Re-embed footgun.** Edits under `memory/` and `install/` are invisible
  until the embedding crate recompiles (`touch src/asset_source.rs` / `src/install.rs`
  + `cargo build`), then `doctrine memory sync`. Verification that reads the
  embedded corpus must run after a rebuild, not after the edit.
- **A1 — ADR-019 is accurate and stays.** Confirmed with the user 2026-08-01.
- **A2 — The four memories are an inventory, not an exhaustive set.** The sweep
  is a scope item precisely because pre-enumerated maps are suggestive only.

## Open Questions

1. **Which residue paths are derived, and which are user-overlay?** R1's verdict,
   per path, sourced from the install legs that write them — not from CHR-043's
   list, which is a suggestion and already partly stale. Decides the gitignore
   set.
2. **Sequencing against SL-144.** SL-144 is `ready` with 0/5 phases and its scope
   reconciliation predates ADR-019; both slices touch `install/manifest.toml` and
   the reference-doc surface. Does SL-144 need a staleness pass before either
   runs, and which goes first?
3. **Does a shipped `[gitignore]` change reach existing clients?** The entries are
   additive on install, so a client that re-runs install picks them up — but
   nothing untracks what that client already committed. Worth confirming the
   forward contract behaves as assumed, and stating plainly in IMP-378 that
   existing clients keep both the residue and the tracking.

## Verification / Closure Intent

"Done" means:

- No shipped surface — memory body, memory `scope.paths`, skill, install asset,
  or generated boot text — asserts that a reference doc exists on disk in an
  installed project. Verified by the new doctor check, not by grep alone.
- The doctor check fails on a reintroduced stale citation, proven by a test that
  reintroduces one.
- Every residue path carries a derived-vs-overlay verdict with its install leg
  cited; the derived ones are ignored in the shipped contract **and** untracked
  here; `.doctrine/governance.md` remains tracked.
- `git status` is clean after a fresh `doctrine install` in this repo — install
  output no longer appears as pending authored changes.
- DEC-010 settled or superseded; CHR-043 closed; IMP-315 closed.

## Summary

## Follow-Ups

- **IMP-378** — stale copies in already-installed clients have no removal path,
  and this slice deliberately does not build one. Needs the once-per-repo,
  version-keyed migration primitive (isolated from doctrine's own dependencies so
  an upgrade cannot break the migration that runs during it). IDE-030 is the
  adjacent general form.
