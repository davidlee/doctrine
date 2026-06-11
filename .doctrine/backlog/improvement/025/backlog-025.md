# IMP-025: Promote content-hashed-path-set to shared primitive

SL-040 builds a standalone leaf module (`src/contentset.rs`) for the D-C10
warm-cache staleness key: a `Set<(path, hash)>` over a root, with an impure
`compute(root, paths)` (reuses `git.rs:300 sha256`) and a pure
`diff(stored, current) → {changed, added, removed}` / `is_stale`. Built
consumer-agnostic and liftable on purpose, but **not** generalised — only one
real consumer exists today (the warm-cache).

Promote it to a shared primitive when a **second real consumer** lands, so the
shared interface is designed against actual tension rather than guesses.

Candidate future consumers (all hypothetical at authoring):
- **Drift Ledger (IMP-022)** — likeliest second consumer; observed-vs-normative
  divergence is content-hash-of-sources at the core.
- **Tech-spec source-binding** — bind a spec to the source paths it describes,
  flag on drift.
- **Architectural triggers** — want **globs**, a matching layer *above* this
  core; the core stays the same, the glob-resolution sits on top.

Each consumer shapes the *layers above* the core differently (curated literal
set vs typed sources vs globs), which is exactly why the shared interface should
wait for a second real consumer.

Source decision: SL-040 `/design`, R1 follow-up (warm-cache domain_map = curated
`(path, hash)` set, single parent root).
