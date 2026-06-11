# CHR-001: RV ledger robustness: baton-note lock scope, domain_map path-component guard, unknown-severity fail-safe close-gate

Three small robustness hardenings surfaced by SL-040 reconciliation (RV-001):

- **F-1 (major) — baton-note lock scope.** `src/review.rs:1516-1522` appends the
  handoff `--note` to `baton.toml` after `with_turn` drops the lock — an unlocked
  RMW outside the lock + CAS. Self-heals via the next entry-CAS (baton is
  regenerable, D-C2), so impact is cosmetic, but it breaks the every-baton-write-
  under-the-lock invariant. Move the note write inside the `with_turn` closure.
- **F-2 (minor) — path-traversal guard.** `src/review.rs:1769`
  `validate_domain_map` uses `path.contains("..")`, false-positives legit paths
  (`src/a..b.rs`). Test `Path::components()` for `Component::ParentDir` instead.
- **F-4 (nit) — close-gate fail-safe.** `src/review.rs:724-737` drops a blocker
  with a hand-corrupted (out-of-vocab) severity from the close-gate; a close-gate
  should fail safe and gate on an unknown severity.
