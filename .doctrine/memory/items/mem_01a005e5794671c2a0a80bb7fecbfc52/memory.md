# rtk-proxied grep output can silently rewrite source identifiers

Observed 2026-08-16 while verifying design claims against the tree. `rg` output
came back with a source identifier **substituted**: `fn status_and_title_for`
rendered as `fn status_and_n`, and every `title_for` in the same result set
became `n`. Separately, `tangle_baseline` rendered as `ln`.

There is no marker. The mangled line is well-formed text that looks exactly like
real source, so a claim read off it — a function's name, a field's name, a
config key — is wrong in a way nothing flags.

## Why it matters here

The hook rewrites bare `git`/`rg`/etc. through `rtk`, so this is the default
path for exactly the work most exposed to it: verifying a design or review claim
against source. An agent checking "does this function exist / what is it called"
is one substitution away from a confidently false finding.

## What to do

- **Never take an identifier's spelling from proxied grep output.** Confirm with
  a range read (`sed -n 'A,Bp'`) or the `Read` tool before asserting a name,
  signature, or arity.
- Suspect it when a name looks implausibly short (`n`, `l`) or when a
  multi-word identifier appears truncated mid-token.
- `rtk proxy <cmd>` is documented as the unfiltered escape hatch, but it was
  **not available** in this jail (`exit 127`), so the range-read fallback is the
  reliable one.

Captured as a friction observation under `.doctrine/observations/records/`.


## It corrupts LINE NUMBERS too, and non-deterministically

Observed 2026-08-16 while integrating `RV-360` on `SL-256`. `grep -n
"acts.record(checkpoint_act("` over `src/design_run/tests.rs` returned
`1081/1086/1091` early in a session and `1094/1099/1104` later — **the same
command, the same unchanged file, two different answers.** The Read tool and
`awk '{print NR}'` both agree with the second.

Multi-line `sed -n 'A,Bp'` is corrupted in a different way: lines are *elided*
from the output, so an 11-line range comes back as 8 lines and content can no
longer be aligned to the range's numbers. A range read that "looks right" can be
showing you content from tens of lines away.

Single-line `sed -n 'Np'` was accurate in every case checked.

## Why this is worse than the identifier case

An identifier substitution sometimes looks wrong (`n`, `ln`). A wrong line
number never does. It produced a confidently-verified claim that four source
anchors resolved on `edge` when two of them did not, and the error survived
until an external reviewer's independent anchors disagreed. Nothing else would
have caught it.

## Rule

**Never cite a line number read off proxied `grep -n` or a multi-line `sed`
range.** Derive it with `awk '/pattern/ {print NR": "$0}'` and confirm at least
one anchor with the Read tool before asserting a set of them. When a reviewer's
anchors disagree with yours by a consistent offset, assume yours are wrong and
re-derive — do not argue from the earlier output.

---

## CORRECTION 2026-08-17 — the mechanism named above was not present

**Superseded by [[mem.pattern.verification.suspect-transcription-before-tool]].
Do not act on the account above.**

`rtk` was real but had been **removed from this environment months before either
sighting recorded here**. Verified 2026-08-17: not on `PATH` in the jail; the
global `PreToolUse` hook array is empty; the project's only `Bash`-matched hook
runs `doctrine memory surface`, which injects context rather than rewriting
commands. There was no proxy in the path on 2026-08-16.

Re-running the line-number case the same way it was recorded — `grep -n`, `rg -n`
and an independent `awk NR` over `src/design_run/tests.rs` — returned
**1094/1099/1104** from all three, the value this memory records as correct.

The observations were most likely agent transcription errors, and this record is
the pattern it warns against: a documented mechanism sitting in context supplied a
diagnosis for a surprise nobody had traced. Retained for lineage only.
