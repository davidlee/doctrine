# DEC-413: codex_mcp execute: implementation deltas from the SL-286 design

Deltas adopted while implementing SL-286 PHASE-03 (`src/codex_mcp.rs`,
`src/fsutil.rs`; commits 40f2c57d0..49e58e65e). Reconcile folds each into
`design.md`.

1. **Project path parameter** — `execute(plan, consent, project_config, cli, out)`:
   the plan carries no project root, so the caller passes `.codex/config.toml`.
2. **Removal re-checks at write time** — "the existing edit-preserving path" is a
   new pure `remove_wrapper(text)` using boot's toml_edit idiom (boot has no
   removal fn). It re-reads the file and removes only an entry `classify_project`
   still calls `OwnedWrapper`; a file changed since the plan is refused.
3. **Empty table** — an implicit `mcp_servers` table goes with its last server; an
   explicit (user-written) one stays; the file is never deleted.
4. **Consent** is `Granted | Declined | NotAsked`; with a proposal, anything but
   `Granted` takes the declined path. Outcome is `Done(Vec<PathBuf>) | Refused(String)`.
5. **Symlink write-through is opt-in** — `fsutil::write_atomic_through_link`, used
   only by the wrapper removal. `write_atomic` keeps replacing a link, so a
   repo-committed symlink cannot redirect engine writes out of the tree (review F1;
   the phase objective's "fsutil writes through a symlink" read narrowly, per the
   design's file map).
6. **Removal fails after the user write** — reason is the F-6 line plus the cause
   indented, followed by the daemon hint (EX-6: every user write gets it).
   Removal-only failure: `wrapper_not_removed` with the cause, no hint.
7. **New report lines** (wording flagged to the user): `current`, `add_failed`,
   `wrapper_not_removed`, `project_malformed`, `project_unreadable`.
8. **Project-block refusals print the cause only** — no snippet: a manual user
   registration would still be shadowed by the project entry.
9. **Accepted asymmetry** — the user file is not re-read before a removal-only
   write; an edit during the consent prompt is the user's own act (review F6).
