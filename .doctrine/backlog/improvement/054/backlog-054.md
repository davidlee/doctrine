# IMP-054: SL-056 gc orchestrator env-leg test asserts dual-cause token not the named verb (proxy)

Source: RV-016 finding F-11 (reconciliation review of SL-056), severity minor / follow-up.

## Detail

`tests/e2e_worktree_gc.rs:602` — the exhaustive Orchestrator env-set refusal test checks
the dual-cause message token but does NOT assert the refused verb is named
(`fork`/`import`/`land`/`gc`). Per
`mem.pattern.review.guard-test-asserts-property-not-proxy` the test should assert the
PROPERTY (this specific verb refused + named), not the shared token proxy: a regression
that names the wrong verb would still pass.

## Fix

Assert the backtick-delimited verb name per member.

## Fixed 2026-07-29

Both refusal loops in `tests/e2e_worktree_gc.rs` now assert the verb as the guard
renders it — backtick-delimited — through one `assert_refusal_names_verb` helper
layered on the file's existing `assert_refusal`.

**The marker leg was the same defect, unreported.** RV-016 F-11 named only the env
leg, but `every_orchestrator_verb_refused_from_a_marked_linked_worktree` asserted a
bare `contains(verb)`, and its own message prose defeats that for the `fork`
member: the refusal opens `worker fork (signal: marker):`, so `"fork"` is present
even when a different verb is named. Fixed in the same pass.

**Verified by mutation, not just by going green.** Shadowing `verb` with a constant
`"gc"` in `worker_guard` (`src/commands/guard.rs`) fails both legs. The marker-leg
failure output — `worker fork (signal: marker): refusing authored write \`gc\`` —
is direct evidence the old assertion passed on a wrong-verb message.

**Defect-class sweep, clean.** `tests/e2e_worker_guard.rs:149,202` already asserted
`` contains(&format!("`{verb}`")) `` for this same guard, so the correct form was
house style and only the gc file's two loops had drifted. The other
`assert_refusal` call sites across the worktree e2e files pass unique cause tokens
(`not-landed`, `tree-unclean`, `merge-conflict`) which discriminate per case — not
proxies.

Noted in passing, not fixed: `assert_refusal` is copy-declared in five
`tests/e2e_worktree_*.rs` files while `tests/common/` exists. Out of scope here.
