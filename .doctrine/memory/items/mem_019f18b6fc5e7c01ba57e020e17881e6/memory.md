# Audit dogfood: seed a live Failed coverage cell from the CLI

`coverage record` cannot directly create a `Failed`/`Blocked` cell — a VT recipe
leans `Planned` until verified, and `record`'s `--status` is honoured only for
VA/VH attestations. So a live-failure dogfood (closure-gate refuse, forget refuse)
needs the cell *derived* to Failed:

```bash
# 1. record a VT cell whose check is guaranteed to fail
doctrine coverage record --slice SL-179 --requirement REQ-113 --change SL-179 \
  --mode VT --command false \
  --matcher-source stdout --matcher-pattern PASS-SENTINEL-NEVER-PRINTED
# 2. re-derive — the failing check flips Planned→Failed
doctrine coverage verify SL-179        # → SL-179/REQ-113/SL-179/VT: Planned→Failed
doctrine coverage show REQ-113         # → verdict: Divergent: observed-failure
```

The cell now bites: `coverage forget` refuses it, and `slice status <id> done`
(from `reconcile`) refuses citing the req. **Cleanup:** `forget` refuses a Failed
cell by design, so don't try to forget it — if the slice had no prior
`coverage.toml` the file is wholly scratch and untracked; `rm` it to revert. If the
store pre-existed, hand-edit out the seeded `[[entry]]` (never `git checkout`).

Use the **candidate binary** (`./target/debug/doctrine`), not the PATH install, so
the dogfood exercises the code under audit. Pairs with the unit-test seam: tests
use a `seed_cell` fixture that upserts an exact stored status directly (see SL-179
PHASE-04 F-04b) — this is its CLI analogue for end-to-end VA dogfoods.
