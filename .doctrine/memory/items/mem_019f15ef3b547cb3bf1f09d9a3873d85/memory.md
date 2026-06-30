# Public git history rewrite (2026-06-30)

The public `davidlee/doctrine` GitHub repo was **deleted and recreated from a
`git filter-repo`-scrubbed history** on 2026-06-30 (release-decoupling campaign,
final step). This severs the old SHA line entirely.

> This memory is tracked and may reach public `origin`. It deliberately does
> **not** restate the scrubbed backend-name strings — naming them here would
> re-leak what the rewrite removed. The exact replacement mapping + purged paths
> live only in the gitignored runbook `scratch/REWRITE.runbook.md`.

## What changed

- **Old SHAs are dead on origin.** Pre-rewrite tip `a4026a98` and every ancestor
  no longer exist on `origin`. New public scrubbed tip: `e920c2f5`. The rewrite
  touched **only** historical blobs + commit/tag messages — each HEAD tree is
  byte-identical to its pre-rewrite counterpart, so source builds unchanged.
- **What was scrubbed:** backend-name leak tells (string-replaced) plus six
  privatized-master blobs purged outright. Specifics in the private runbook.
- **Frozen contract — never touch:** the `forget.*` wire-tag VALUES in
  `src/git.rs` (`forget.remote.v1`, `forget.checkout.v1`, `forget.repo.*`) are a
  byte-frozen first-party contract, NOT a leak. Any future scrub must scope to
  the backend-name word-forms only and leave bare `forget` / the wire tags
  untouched.

## dogma = dirty backup (hard rule)

The **`dogma` remote** (`git@github.com:davidlee/dogma.git`) holds the full
UN-scrubbed pre-surgery history. It is the recovery point. HARD RULE: it stays
private forever; **never seed/restore a public repo from it; never make it a
push target during any future rewrite.**

Locally, old objects survive only because `refs/remotes/dogma/*` pins them — by
design. No local branch or `origin/*` ref reaches the old history, so accidental
re-leak via a branch push is not possible. If a future agent sees the live repo
holding objects absent from `origin`, that is the intended dogma backup, not
drift.
