# Review RV-211 — code-review of IMP-191

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

IMP-191 adds a read-only query form to `slice status <ID>` (the bare-no-STATE
path), a `legal_moves` helper, and an `Option<SliceStatus>` widening of
`run_status`. Two files changed: `src/slice.rs` (+70/-28), `src/reconcile.rs`
(+4/-4 call-site wraps).

Lines of attack:
1. **Test coverage** — is the new `None`-state branch tested, or does the
   existing suite only exercise `Some(...)`? The `legal_moves` helper is pure
   — is it tested?
2. **Status-variant duplication** — `legal_moves` enumerates all 9 status
   variants as string literals. If a new variant joins `SliceStatus`, this
   array must be manually synchronised. Does this risk silent miscounts during
   future maintenance?
3. **Flag interaction** — `--note` is silently ignored when STATE is absent.
   Should this error or warn?
4. **Call-site audit** — the only external call site is `reconcile.rs` which
   always wraps `Some(...)`. Are there others?

## Synthesis

**Overall:** solid

**Synopsis:** IMP-191 delivers exactly what it promised — a read-only query
form for `slice status <ID>` that prints lifecycle state, phase rollup, legal
transitions, and the divergence marker, all without touching the authored
status. The implementation is parsimonious: an `Option<SliceStatus>` widening
in the existing `run_status` shell with a clean early-return branch, a pure
`legal_moves` helper derived from the existing `classify` FSM, and a single
call-site wrap in `reconcile.rs`. No new dependencies, no new modules.

Three findings raised and resolved in-session:

1. **Test coverage (blocker → fix-now, verified).** The read-only branch and
   `legal_moves` had zero dedicated tests. Added four tests covering the
   read-only smoke path, `--note`-without-STATE refusal, `legal_moves` from
   each status (including drifted/terminal), and a variant-coverage guard that
   fails if a new `SliceStatus` variant is added without updating the
   `legal_moves` array.

2. **Status-variant duplication (major → fix-now, verified).** The parallel
   string array in `legal_moves` is a maintenance hazard. The variant-coverage
   test (`legal_moves_covers_all_slice_status_variants`) provides a runtime
   guard — a new variant must pass through the `all` array or the test fails.
   Not a compile-time guard, but a practical catch at suite time.

3. **`--note` silently ignored (minor → fix-now, verified).** The read-only
   branch now errors with `--note requires a target STATE` when `--note` is
   passed without a target state, catching the "forgot the STATE argument"
   user error.

**Standing risks:** the sibling verbs (`revision status`, `adr status`,
`policy status`, `standard status`, `rfc status`) all share the same
reader/writer overload pattern. IMP-191's body scoped that as follow-up work —
not part of this change. The pattern is proven; replication is mechanical.

**Haiku:**

```
bare status query —
the FSM tells its secrets
no write, just the truth
```
