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
