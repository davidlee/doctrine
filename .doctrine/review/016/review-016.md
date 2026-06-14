# Review RV-016 — reconciliation of SL-056

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

A code-review workflow fan-out (5 dimensions + a codex pass, adversarially
verified) probed the SL-056 dispatch-funnel surface at `sl056-coord@0feb672`,
holding it to its own thesis: the import belt is the malice-containment seam, and
governance/config files must not ride back into the coordination index.

## Synthesis

**Headline — the two belt blockers (F-3, F-4).** The slice's whole thesis is the
import belt: a pure prefix-match (`.doctrine/`/`.claude/`) over the `B..fork`
name-only diff. Both blockers were the *same shape* — an unhardened diff
invocation let a malicious worker smuggle a governance write past the belt:
F-3 via git's default `core.quotePath=true` (a non-ASCII/space path is C-quoted,
so the leading `"` defeats `starts_with`), F-4 via default rename detection (a
governance deletion paired with a same-content add elsewhere collapses to the
destination line, hiding the `.doctrine/` source). Fix was two flags
(`-c core.quotePath=false`, `--no-renames`) + two reproduction tests; the apply
patch took `--no-renames` too to keep its view consistent with the belt's.

**The rest.** F-5/F-7 hardened gc honesty (a `--dry-run` that lied `landed ✓`
over a `--force`/branch-gone reap; a target base resolved from the coordination
root instead of the fork dir). F-8/F-9 closed two narrow correctness gaps
(false rollback debris; a re-entrant stamp re-provisioning an already-marked
worktree). F-6/F-14 were prose-vs-verb drift (the slice's own fail-mode) in a
skill + the README. F-10 was design-wrong, not code-wrong: §5/§9/§11/§12 still
named the dropped `create-fork` path as the build target — reconciled to the
shipped SubagentStart-stamp. F-11/F-12/F-13 (test proxies + a hook-merge
fail-open) were captured as IMP-054/IMP-055/ISS-011. F-15/F-16 confirmed aligned
(F-15 carries an owner VH sign-off at /close on the §5 hook-mint exemption).

**False-red note (F-1, F-2, withdrawn).** Both were artifacts of the coord
worktree sharing `CARGO_TARGET_DIR` with main (the jail redirect): test binaries
built from main's source shadowed coord's, producing a deterministic + a flaky
RED that vanished on a fresh per-suite compile (`touch` the test file, run suites
individually, `env -u DOCTRINE_WORKER`). Worth a memory — a cousin of
`mem.pattern.dispatch.worktree-removal-stale-manifest-dir-false-red`.
