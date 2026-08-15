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
