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

What remains is everything the 80/20 deliberately deferred, plus the mechanism
the deletions need.

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

3. **CHR-043 prescribes the opposite remedy** — gitignore those nine, i.e. keep
   them as a projected/derived tree. Post-SL-227 they are neither authored nor
   derived. The two backlog items must be reconciled, not both actioned.

4. **`install/manifest.toml:56`** points readers at `install/using-doctrine.md` —
   an `install/` path, which exists only in this build repo.

5. **Nothing detects any of this.** `doctrine doctor` has no currency or liveness
   check for a reference-doc citation. Its only reference-doc awareness is
   `prose_cite` noise suppression for `glossary.md` (IMP-252). Without a check,
   the corpus re-rots on the next delivery change — this slice's own fixes
   included.

6. **No migration mechanism exists.** Deleting a client's stale `.doctrine/*.md`
   copies is not something `install` can do: install is write-if-absent and
   idempotent by contract, and a client's copy may carry local edits. A
   once-per-repo, version-keyed migration is the missing primitive, and this
   slice is where it gets established — deletion is its first customer.

### Sibling concerns, deliberately not merged

- **IDE-030** is the general client-side form: any framework-owned doc that
  arrived under write-if-absent install goes stale forever after an upgrade. The
  migration mechanism built here is its enabling primitive; the general sweep
  stays filed.
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

3. **Establish the migration primitive** — once-per-repo, version-keyed, isolated
   from doctrine's own dependencies so a migration that runs during an upgrade
   cannot be broken by the upgrade. Shape, invocation point, idempotence
   guarantee, and where the ledger of applied migrations lives are `/design`'s
   call. This is the precedent, so the bar is the contract, not the one script.

4. **Retire the orphans through that primitive** — the first migration deletes
   the framework-owned stale copies (never `governance.md`, which is user-owned;
   never a copy carrying local edits without saying so). Applies to this repo and
   to any client that installed before ADR-019.

5. **Add a doctor check** that fails on a reference-doc citation which cannot
   resolve — a bare `<name>.md` with no published counterpart, or any surviving
   `.doctrine/<name>.md` / `install/<name>.md` path assertion in a shipped
   surface. This is the regression control that keeps objectives 1–4 from
   re-rotting.

6. **Reconcile CHR-043** — close or rewrite it against the decision this slice
   makes, so the backlog stops carrying two contradictory remedies.

### In scope

- `memory/` — the shipped corpus: bodies and `scope.paths` of the affected
  memories, plus the sweep for others.
- `install/manifest.toml` — the `install/` path in the comment at line 56.
- The migration mechanism — its home, its ledger, its invocation seam, its first
  script.
- `src/doctor_checks.rs` (+ wherever the check registry lives) — the new check.
- `.doctrine/*.md` — the nine orphans, deleted via the migration.
- DEC-010; CHR-043; IMP-315 (closes it).

### Out of scope

- **Reference-doc *content*.** Currency of `glossary.md` / `using-doctrine.md`,
  IA overlaps, restate-line compliance — SL-144 owns these. This slice does not
  edit what a reference doc says, only how it is reached and described.
- **The general write-if-absent currency sweep** across all framework-owned
  client assets — IDE-030, enabled but not executed here.
- **Changing the projection/publication split.** ADR-019 is accurate and is a
  premise, not a target. `governance.md` being created-on-need is settled.
- **`boot-footer.md`'s retirement** — SL-144 objective 2 owns the deletion of the
  dead asset. This slice deletes its stale *projected copy* along with the other
  orphans; whether the master survives is SL-144's call.
- **New entity kinds, CLI verbs, or memory-engine changes.**

## Non-Goals

- A general-purpose migration *framework* with rollback, dry-run matrices, and a
  DSL. The precedent should be the smallest thing that is safely repeatable.
- Rewriting every bare `<name>.md` citation across ~20 skills to carry the
  `library show` command. The boot sector states the rule once (`ddd138ab`);
  duplicating it per citation is the parallel-implementation failure.

## Affected Surface

- `memory/mem_019ec92b10037850817507044f0f99ef/` (`mem.signpost.doctrine.reference-docs`)
- `memory/mem_019f2b93f5e178009191711f607caff6/`
- `memory/mem_019e9a11cda27db19c0c75bafa453d5d/` (file map)
- `memory/mem_019ec92b0fff76e1935b222348938d7f/` (install signpost)
- `install/manifest.toml`
- `src/doctor_checks.rs`, and the check-registry seam it is wired through
- migration mechanism — home TBD at `/design`
- `.doctrine/glossary.md`, `.doctrine/using-doctrine.md`, `.doctrine/review-ledger.md`,
  `.doctrine/routing-process.md`, `.doctrine/dispatch-mechanics.md`,
  `.doctrine/model-band.md`, `.doctrine/harvest.md`, `.doctrine/boot-footer.md`
  (deleted; `.doctrine/governance.md` retained — user-owned)

## Risks & Assumptions

- **R1 — Deleting a client's local edits.** A client may have edited a projected
  copy in place. The migration must detect divergence from the published bytes
  and refuse-or-report rather than delete silently. This is the sharpest risk in
  the slice and the reason the primitive precedes the deletion.
- **R2 — Migration ordering against the upgrade itself.** A migration keyed to a
  version runs in a repo whose binary has already changed. Isolation from
  doctrine's own dependencies (the user's stated constraint) exists precisely so
  a mid-upgrade migration cannot be broken by the upgrade; the design must say
  what "isolated" concretely forbids.
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

1. **Does the migration mechanism warrant an ADR?** "How doctrine migrates client
   repos across versions" is durable, project-global, and binds every future
   release — the altitude test points at ADR, with this slice as its first
   application. Resolve at `/design`.
2. **Sequencing against SL-144.** SL-144 is `ready` with 0/5 phases and its scope
   reconciliation predates ADR-019; both slices touch `install/manifest.toml` and
   the reference-doc surface. Does SL-144 need a staleness pass before either
   runs, and which goes first?
3. **Where does the applied-migration ledger live?** Runtime state is
   `rm -rf`-able by contract, so a ledger there would re-run migrations after a
   state wipe; authored state makes migration history a reviewable diff. The
   storage rule has to be applied deliberately here.

## Verification / Closure Intent

"Done" means:

- No shipped surface — memory body, memory `scope.paths`, skill, install asset,
  or generated boot text — asserts that a reference doc exists on disk in an
  installed project. Verified by the new doctor check, not by grep alone.
- The doctor check fails on a reintroduced stale citation, proven by a test that
  reintroduces one.
- The migration primitive has a stated contract (keying, idempotence, isolation,
  ledger) and one exercised script; running it twice is a no-op, and a locally
  edited copy is reported rather than deleted.
- This repo's nine orphans are gone, `.doctrine/governance.md` retained.
- DEC-010 settled or superseded; CHR-043 reconciled; IMP-315 closed.

## Summary

## Follow-Ups
