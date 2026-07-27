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
| `closure.sh` | DEC-080 — RV-314 F-15/F-16/F-20. Index-conditioned emission drops a detached and a never-tracked symlink target; a derived string read as pathspec magic narrows the measured set; an unmatched `:(literal)` spec is inert and masks nothing. Four falsifiers, all held. |
| `attributes.sh` | DEC-087 / CON-002 — RV-314 F-19. Committed attributes defeat all three legs on both the `eol` and clean-filter routes; `--attr-source=<empty tree>` restores them; the empty-tree oid differs by hash algorithm; the capability probe discriminates by exit status. Runs **both** `diff-index` forms side by side so the round-3 acquittal cannot recur. Five falsifiers, all held. |

## Rounds 2–3: figures re-authored, not re-derived

`closure.sh` and `attributes.sh` were written in round 4 to recover measurements
that rounds 2 and 3 had taken in a scratch directory since deleted. The figures
were durable — in the RV-314 ledger, `design.md` and `notes.md` § Harvest — but
the *scripts* were not, which is precisely the state `probes/` exists to prevent
(RV-313 F-1). Every falsifier registered in the two new scripts holds on
re-run, and the exact oids reproduce.

**Absolute byte counts do not, and should not be read as though they do.**
`design.md` quotes 145 / 152 / 143 / 156 / 172 bytes from the original fixtures;
these scripts produce different numbers from their own. A `git diff HEAD
--binary` byte count is a function of path names and file content, so a fixture
difference moves it. What re-runs — and what the falsifiers are written against
— is the *discrimination*: zero versus non-zero, exit 0 versus exit 1. Treat the
quoted absolutes as provenance for a measurement that happened, not as values to
re-assert.

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
