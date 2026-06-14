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

RV-026 (post-ship code-review of SL-040) surfaced two siblings of the above:

- **RV-026 F-7 == this item's F-1 (baton-note lock scope)** — same code, same
  fix (`run_raiser_transition` writes the handoff note via `read_baton`/`write_baton`
  after `with_turn` drops the lock). Already owned here.
- **RV-026 F-2 (read-path fail-safe sibling of F-4).** `parse_finding_status`
  (`src/review.rs:649`) defaults any unknown status string to `Open` with zero
  diagnostic — a hand-edited typo (`"verfied"`) leaves the review silently Active
  forever. Same unknown-enum-read class as F-4 (close-gate severity), different
  site (the general read path feeding `finding_states`/the verb gate/the
  close-gate). Fix alongside F-4: warn or hard-error on an out-of-vocab status
  rather than silently coercing. Low real-world risk (write path can only emit
  vocab statuses; default-Open is the safe direction — never silently *closes* a
  review), so minor, not a blocker.
