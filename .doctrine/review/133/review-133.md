# Review RV-133 — reconciliation of SL-136

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit mode: conformance self-audit for SL-136 after dispatch completion.

Reviewed surface: `refs/heads/candidate/136/review-001` / worktree `.doctrine/state/dispatch/candidate/cand-136-review-001` at `898e6bd26da5fda745c6336e2e71478200ff3882`, created from `refs/heads/review/136` (`42c7691b0566`) onto `refs/heads/main` (`cda19d70a264`). Parent worktree dirt is intentionally excluded from evidence.

Lines of attack:

- Does the implementation follow SL-136 design D1/D2: one shared root-level tag write leaf, curated taggable set, and no write-only included kind?
- Does `doctrine tag` preserve backlog semantics, reject excluded/non-numbered kinds clearly, and keep memory tagging out of scope?
- Are all read surfaces wired for included kinds: `list --tag`, table `show`, and JSON output?
- Did the governance/RFC corpus and templates actually migrate away from `[relationships].tags` to root `tags`, including RFC fixtures and goldens?
- Does verification cover the phase VT criteria plus `just check`, and is the known D6 REV obligation for SPEC-005/SPEC-016/SPEC-018 explicit for `/reconcile`?
