# DEC-053: Claim surface is built from the index, never the filesystem

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The decision

`memory verify`'s claim surface is constructed **entirely from the git index**.
The declared scope entry is emitted as a pathspec magic-prefixed by **the field
it came from** (`scope.paths` → `:(literal)`, `scope.globs` → `:(glob)`),
expanded with `git ls-files -s -z`, and every match of mode `120000` has its
target read from the **index blob** and re-expanded. No `realpath`, no
filesystem `stat`, and no classification of an entry by its characters.

This **replaces** SL-230's ordered algorithm (design § 5.2 steps 1–5). It is not
a fourth repair of it.

Taken during SL-232's design round, after reproducing RV-307 F-37's three routes.

## Why — three routes, one defect

F-37 reported three ways an entry contributes while bypassing canonicalisation
and reading clean against a dirty target. All three reproduced on git 2.54.0.
They are **not three bugs**:

| route | why the old rule missed it |
|---|---|
| `missing/../link` | git normalises `..` **lexically**; `realpath -e` requires the intermediate directory to exist |
| sparse checkout / `skip-worktree` | git matches the **index**; `realpath` requires the working tree |
| a literal filename containing `*` | git's `:(literal)` reads `*` as a character; step 2 read it as a wildcard |

The single statement covering all three: **the design canonicalised with a
filesystem oracle and then asked git an index question.** A shape matrix shows
the two instruments are uncorrelated in *both* directions — `missing/../link`
and a sparse entry fail `realpath -e` and still contribute; `linkdir/target.txt`
resolves cleanly and contributes nothing.

`cat-file blob :<path>` returning a link target **while the file is absent from
the working tree** is the load-bearing fact: it is why index semantics closes the
sparse-checkout route that no filesystem-based rule can.

## What this retires

- **Character-based shape classification** (`*`, `?`, `[` → "pattern"). The
  schema already records path-vs-glob in the field name; step 2 discarded it and
  re-derived it unreliably. This is F-37's structural correction and the root
  answer to RV-307 F-32's returned contest, which the round-7 fix addressed only
  at the split point.
- **The whole-component-prefix rule.** Nothing is resolved before emission, so
  there is no prefix to split.
- **E13's stated basis.** Its justification was mechanical necessity — git
  aborts `exit 128` rather than returning a verdict. Escaping and absolute-outside
  entries are now rejected **lexically before emission**, so the abort is
  unreachable rather than handled, and E13 must be re-justified or folded into
  E7's reporting.
- **Most of R-A's enumerate-then-probe burden.** The obligation was discharged in
  method (a shape matrix over `..` aliases, sparse checkout, `skip-worktree`,
  `assume-unchanged`, glob metacharacters, symlink chains and topology,
  `core.quotePath`, `core.ignoreCase`, control chars). But the taxonomy is now
  **non-load-bearing**: shapes are no longer classified by us, so a fourth
  unenumerated shape has nothing to break. That is the substantive reason to
  prefer this over a repair — the previous three totality claims (F-26, F-32,
  F-37) each failed by asserting a rule over an under-enumerated domain, and this
  rule has no domain to under-enumerate.

## Measured, so it is not overclaimed

Census at HEAD `9f8cf40b`, 440 scope entries across 389 memories:

- symlink resolution adds **zero** coverage for declared scopes today — 25
  entries match symlinks, all self-covering; **0** entries are symlink-*rooted*,
  which confirms the inherited "no glob declaration is symlink-rooted" claim
  rather than refuting it;
- it *is* live for the **uid directory base**, reached through one of 347 key
  symlinks (F-15);
- the full resolve pass over `.doctrine/**` — 7,670 matches, 2,071 symlinks —
  costs **7ms**, so scale is not a constraint.

So the value of this decision is **totality by construction**, not live defect
count. Stating it the other way round would be the overclaim RV-307 F-25/F-33
punished.

## Residuals it does not close

- **Index bits suppress the measurement itself.** `skip-worktree` (`S`) and
  `assume-unchanged` (`h`) make a modified tracked file read `diff-index` exit 0.
  No pathspec-based approach can close this; it is detectable via
  `git ls-files -v` and affects the source leg too. Live population: **0** rows.
- **Case-insensitive collision is unmeasured.** `core.ignoreCase` alone did not
  flip pathspec matching on ext4; a genuinely case-insensitive filesystem could
  not be probed. Carried as unmeasured, not cleared.
- **`-z` is mandatory**, not stylistic: `core.quotePath=true` renders `ünï.txt`
  as `"\303\274n\303\257.txt"` and corrupts any parsed output.

## Relations

- SL-232 (objective 2), RV-307 F-37 / F-26 / F-32, DEC-027 (the split), DEC-020
  (non-contribution is reported, never classified — untouched by this).
- Invariant I8 is untouched: entries remain magic-prefixed, so nothing a memory
  declares can subtract from what it is measured against (F-18).
