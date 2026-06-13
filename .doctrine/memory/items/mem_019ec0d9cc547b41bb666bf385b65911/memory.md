# Relation surface is FULLY wired in source (SL-048): the 'half-wired' symptoms were a STALE installed jail binary; verify against a fresh dev build

**CORRECTION (supersedes the original "half-wired" claim, which was wrong).**
The relation surface is wired end to end in source. The symptoms that looked
like "relations are write-only / half-wired" were entirely an artifact of the
**stale read-only installed `~/.cargo/bin/doctrine`** in the jail, which predates
SL-048 — it renders no relations and has no `link` verb. Every diagnosis run
against it was false.

What is true in current source:

- Tier-1 entity relations are uniform `[[relation]]` rows (`label` + `target`),
  NOT the legacy typed `[relationships]` table. Author them structurally.
- Legal slice labels (source = slice): `specs` (→ SPEC-NNN **or** PRD-NNN),
  `requirements` (→ REQ-NNN), `supersedes` (→ SL-NNN), `governed_by` (→
  ADR/POL/STD). Governance `related` is governance-source-only (SameKind); there
  is **no** slice→slice `related`. The legal `(source, label)` set lives in
  `RELATION_RULES` (`src/relation.rs`), read by `read_block`. Row form:

  ```toml
  [[relation]]
  label = "specs"
  target = "SPEC-002"
  ```

- **Read side renders.** `inspect <ID>` (outbound) and `slice show` (table +
  json `relationships`) display authored tier-1 rows. Verified on SL-057/SL-058
  with the dev binary.
- **Write side exists.** `doctrine link <ID> <label> <target>` / `unlink` are
  top-level verbs over `append_edge`/`remove_edge`, write-strict + idempotent
  (SL-048 §5.4). Exercised end to end.

**The trap (the real durable lesson).** In the jail, `~/.cargo/bin/doctrine` is
read-only and can lag source by many slices. NEVER diagnose CLI behaviour against
it. `cargo build` (writes to `~/.cargo/doctrine-target-jail/debug/doctrine` — see
[[mem.pattern.build.jail-target-redirect]]) and run THAT binary. The same trap
bites test binaries via a baked `CARGO_MANIFEST_DIR` pointing at leftover
`.worktrees/` copies (see
[[mem.pattern.dispatch.worktree-removal-stale-manifest-dir-false-red]]) — touch
the test `.rs` to force a re-bake before trusting a corpus-walk result.

**The one real remaining gap (SL-058 / ISS-009).** The scaffold templates
(`install/templates/{slice,adr,backlog}.toml`) still emit the migrated tier-1
keys as typed `[relationships]` slots (slice: whole table; adr: `related`;
backlog: `slices`/`specs`/`drift`). Entities born from them carry stale keys —
ADR-011 tripped `e2e_relation_migration_storage` (fixed in `138038c`); backlog
009/010/045-049 are latent (their `[relationships]` header's inline comment slips
past the test's exact-match parser). IMP-048 and ISS-010 were closed as
already-shipped phantoms.

Verification that hand-authored rows are legal: `doctrine validate` (read-tolerant,
reports illegal/dangling, never rewrites). It confirms legality but does not prove
rendering — use `slice show` / `inspect` for that.

Related: [[mem.pattern.review.superseded-by-is-adr004-carveout]],
[[mem.pattern.entity.numbered-kind-identity-table]],
[[mem.pattern.build.jail-target-redirect]],
[[mem.pattern.dispatch.worktree-removal-stale-manifest-dir-false-red]].
