# SL-232 design-round probes

Executable evidence behind this slice's design assertions. Committed so the next
reviewer can **re-run** rather than re-derive — RV-307 spent eight rounds on
claims that were reasoned about instead of measured, and its standing lesson is
that a tool property is a claim needing a falsifier, not a premise.

All measured on **git 2.54.0**. Census figures are stamped with the HEAD they
were taken at (`9f8cf40b`) because RV-313 F-1 caught a design-time absolute that
failed to reproduce at execution purely through corpus growth (11-of-30 →
3-of-48).

Each script **registers its falsifiers in a header comment before the probe
runs**. A probe that cannot distinguish the two outcomes proves nothing (RV-307
F-17, F-23 — committed three times on that ledger, twice by whoever had just
named the rule).

| script | what it establishes |
|---|---|
| `route1.sh` | RV-307 F-37 route 1 — `missing/../link`: git normalises `..` *lexically*, `realpath -e` requires existence. Contributes, reads clean against a dirty target. |
| `route2.sh` | F-37 route 2 — sparse checkout. Shows sparse-checkout *is* `skip-worktree` (`S` on the index row), so R-A's shape list double-counted one mechanism as two. |
| `route3.sh` | F-37 route 3 — a real filename containing `*`. The contrast line is the sharp one: read as `:(glob)` the entry matches a **different file set**, so the character-sniff changes what is measured, not just whether it resolves. |
| `shapes.sh` | The R-A enumeration. Lexical aliases, `.`/`//`/trailing slash, sparse + `skip-worktree` + `assume-unchanged`, glob metacharacters in literal names, symlink chains and topology, abort/outside shapes. **The result that matters: `realpath` and contribution are uncorrelated in *both* directions.** |
| `candidate.sh` | The index-first rule (DEC-053) against five registered falsifiers. FAL-2 is load-bearing: `cat-file blob :<path>` returns a link target *while the file is absent from the worktree*. FAL-4 and FAL-5 **failed** — recorded as such. |
| `residue.sh` | What the candidate does **not** close, plus `core.quotePath` (corrupts parsed output; `-z` defeats it), `core.ignoreCase` (**unmeasured** — did not flip matching on ext4), NUL/newline at the argv boundary, and the index-only ancestor walk that recovers FAL-4. |
| `census.py` | The live-corpus census: 440 scope entries / 389 memories. Delegates all pathspec matching to git rather than re-implementing it. |

## Findings that came out against the hypothesis

Recorded because a probe round that only confirms is a probe round that was not
falsifiable:

- **FAL-5 — index bits defeat the candidate too.** `skip-worktree` /
  `assume-unchanged` make a modified tracked file read `diff-index` exit 0. No
  pathspec approach closes this. Detectable via `ls-files -v`; live population
  **0** in this repo. Carried as slice risk R-E.
- **`core.ignoreCase` is unmeasured, not cleared.** It did not make pathspec
  matching case-insensitive here; a genuinely case-insensitive filesystem could
  not be probed from this jail. Carried as R-F.
- **The symlink-resolution step adds zero coverage for declared scopes today** —
  25 entries match symlinks, all self-covering; **0** are symlink-*rooted*. That
  *confirms* the inherited "no glob declaration is symlink-rooted" claim rather
  than refuting it. The pivot's value is totality by construction, not live
  defect count, and saying otherwise would be the overclaim RV-307 F-25/F-33
  punished.
