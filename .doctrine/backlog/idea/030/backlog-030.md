# IDE-030: Client-repo doc currency under write-if-absent install

## Idea

Give doctrine a path to **refresh shipped reference docs in an existing client**
whose local `.doctrine/*.md` copies have gone stale relative to a newer doctrine
release.

## Origin

Surfaced by the SL-144 (ADR-005 docs-IA) external design review. That slice's
reachability model is **shipped ∧ pointed-at**, but "shipped" via `build_plan`
step 2 is **write-if-absent**: re-install/upgrade does **not** overwrite a doc
that already exists in the client. So a client installed against an older
doctrine keeps its old `glossary.md` / `using-doctrine.md` forever, even after
upgrading the binary. SL-144 audits *what ships from this repo* (fresh install)
and explicitly scopes stale-client currency **out**; this is that carved-out
concern.

## The hazard

- Distribution reachability (doc arrives on fresh install) ≠ semantic currency
  (the arrived copy matches the current release).
- Write-if-absent is correct for user-editable hooks (`governance.md`,
  onboarding seed) — you must not clobber user edits. It is *wrong* for docs the
  user is not meant to edit (`glossary.md`, `routing-process.md`,
  `review-ledger.md`), which silently rot.

## Shape (not yet designed)

1. **Classify the ship set** — which `install/*` files are user-editable (never
   overwrite) vs framework-owned (safe/desirable to refresh). The hymns
   `seal`/`expose` split in `manifest.toml` is prior art for a per-asset policy
   axis.
2. **A refresh verb** — likely `doctrine reseat` (already exists) grows a
   `--docs` / framework-owned mode that re-materialises framework docs while
   leaving user-editable hooks and any locally-diverged file untouched (or
   3-way / `.orig` on divergence).
3. **Divergence detection** — compare the installed copy against the embedded
   original (RustEmbed content hash) to tell "stale but pristine" (safe to
   overwrite) from "user-edited" (needs care).

## Why eventually worth executing

- Closes the currency gap SL-144 can only *name*: an upgraded client currently
  runs on stale doctrine docs with no signal and no remedy.
- Rides existing machinery (`reseat`, the manifest per-asset policy pattern,
  RustEmbed originals) rather than a new subsystem.

## Dependencies / relations

- Concerns the install/reseat surface and `install/manifest.toml`.
- Downstream of SL-144 (which establishes the framework-owned vs user-editable
  doc distinction the classification needs).

## Not this

- Not clobbering user-editable hooks — write-if-absent stays correct there.
- Not a SL-144 deliverable — that slice is docs-IA, no install-mechanism code.
