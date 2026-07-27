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

## Round 4 additions (RV-314 F-21…F-32, DEC-089 / DEC-090 / DEC-091)

All on git 2.54.0. Each script carries its falsifier in-header.

| script | question | headline result |
|---|---|---|
| `attr-sources.sh` | which attribute sources survive `--attr-source`? | **tree** closed by the flag; **`core.attributesFile`** closed only by `-c core.attributesFile=/dev/null`; **`$GIT_COMMON_DIR/info/attributes`** closed by *nothing* tested. Also: the flag flips a committed `merge=ours` to `unspecified`, which is F-23 |
| `freshness.sh` | does `core.fsmonitor` blind the legs, and does config neutralisation lift it? | blinds all three legs with `ls-files -v` reading `H`; `-c core.fsmonitor=false` restores them **even on a primed index and after re-priming** |
| `index-tags.sh` | can one `ls-files` discriminate every suppression? | `h` / `S` / `S` / `M` for assume-unchanged / skip-worktree / sparse / unmerged, `H` otherwise; **`-s -v -z` combine into one invocation** |
| `raw-bytes.sh` | is `untracked_fingerprint`'s `hash-object` filter-sensitive? | **yes** — two different untracked files collide to one oid under a `clean` filter. This falsified a claim in this design's own first draft |
| `no-filters.sh` | is `--no-filters` sufficient, or does eol conversion survive it? | sufficient for **both** the filter and the `text eol=crlf` routes |

## Round 5 additions and repairs (RV-314 F-33…F-42)

Round 5 raised nine findings, three of which (F-41) were against these artefacts
themselves. Repaired here, plus the two probes round 4 claimed and never wrote.

| script | question | headline result |
|---|---|---|
| `stat-cache.sh` | **new.** does a same-size, mtime-preserving edit stay invisible? | **yes, by two independent routes.** Route B: either `core.trustctime=false` or `core.checkStat=minimal` alone blinds all three legs. Route A (worse): the same blindness on **stock config**, separated from the control only by elapsed time. `ls-files -v` reads `H` throughout, so DEC-090 cannot see it. The git-internal predicate behind Route A is **unexplained and says so** |
| `non-utf8.sh` | **new** (T84's missing artefact, F-41 limb 2). is the non-UTF-8 symlink-target member reachable? | yes — a `120000` entry whose blob is `ff 74 61 72 67 65 74` returns **exit 0** from `cat-file blob`, so the rc carries no signal and the discrimination is the UTF-8 conversion, which is where DEC-090 puts the refusal |

**Repairs to existing probes (F-41):**

- `index-tags.sh` — `$?` after a pipeline read `tr`'s status, reporting `rc=0`
  for a `cat-file` that exits **128**, i.e. pinning DEC-090's load-bearing fact
  backwards. Now captures the rc from a non-pipelined invocation and asserts it.
- `attr-sources.sh` — the linked worktree was never cleared and the `worktree
  add` failure was suppressed, so every rerun printed `fatal: not a git
  repository: (null)` and silently measured nothing. Now cleared, `-B`, and
  loud on failure.
- Exec bits normalised across all scripts (by explicit filename — a previous
  `chmod +x probes/*.sh` flipped modes on six probes another agent authored).

**Re-run status:** all 18 scripts execute green as of this round, and the three
repaired ones were checked for **idempotence** (run twice, byte-identical output)
— which is the property `attr-sources.sh` had lost.

**Two instrument failures caught by controls, recorded because both are the
standing lesson recurring:**

- `non-utf8.sh` first used `iconv`, which is **absent in this jail**. It returned
  127 for every input, so the fixture *and* the control both read "INVALID UTF-8"
  and the probe appeared to confirm the design while measuring nothing. The
  control caught it; the checker is now verified against known-valid and
  known-invalid input before it is trusted.
- `stat-cache.sh`'s first fixture failed its own control, and the responder
  explained the failure away as a same-second artefact rather than following it.
  It was real signal — Route A above.

**The absolute-byte-count rule above applies to these too**, and round 4's
figures are likewise fixture-local. What reproduces is the discrimination and the
exact oids — `3c79cdb822b0…` (the collision), `81920715…` / `d004ceee…` (raw,
separated), `126799cc…` (raw CRLF) versus `0eabd516…` (LF-normalised).

### Findings that came out against the hypothesis, round 4

- **`FAL-5` / R-E is now CLOSED, not carried.** The note above says "no pathspec
  approach closes this. Detectable via `ls-files -v`". Both halves were right;
  the conclusion drawn from them was wrong. `ls-files -v` costs nothing where the
  expander already runs, and DEC-090 turns the detection into a refusal.
- ~~**F-22's stat-cache limb did NOT reproduce.** The tracked leg reported 101
  bytes under both the default config and the reviewer's weakened
  `core.trustctime=false` + `core.checkStat=minimal`. Recorded inside the finding
  rather than dropped; F-22 rests on its fsmonitor limb alone.~~
  **STRUCK — this counter-result is false (RV-314 F-33 / F-42).** No probe was
  ever persisted for it, so it could never be re-run or reviewed; it was a README
  assertion standing where evidence was claimed to be. `stat-cache.sh` now
  falsifies both halves: either config key alone blinds all three legs, **and** a
  second route needs no configuration at all. The likely cause of the original
  non-reproduction is a fixture that let mtime move — `core.checkStat=minimal`
  still compares mtime and size, so such a fixture cannot observe the blinding and
  reports a false negative. Kept visible rather than deleted, because the lesson
  is that an unpersisted counter-result silently narrowed a live finding for a
  full round.
- **`GIT_ATTR_NOSYSTEM` is the one unmeasured neutralisation.** `/etc/gitattributes`
  does not exist in this jail and constructing it means writing to a shared
  `/etc`. Declared unverified rather than assumed measured.
- **A negative grep is not evidence unless it could have gone positive.** The
  first sweep for attribute-sensitive reads inside `capture()` grepped for
  `run_git|git_stdin|Command::new` and returned empty; `capture()` uses the
  `git_bytes`/`git_opt`/`git_text` wrappers, so the query was incapable of a
  positive. The real defect was one probe away and nearly shipped.
