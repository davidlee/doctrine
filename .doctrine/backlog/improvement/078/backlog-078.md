# IMP-078: dispatch sync --integrate is silent about its trunk/worktree outcome

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Two DX papercuts hit during the SL-068 `/close` (the first real candidate-integrate
landing). Both are about `dispatch sync --integrate` under-reporting what it did:

**1. `--integrate` without `--trunk`/`--edge` silently lands nothing.** The verb
replays the journal and prints `integrate: N ref(s) replayed` — identical output
whether or not trunk moved. A closer who omits `--trunk refs/heads/main` (easy: the
handover prose and the `/close` skill both render it as optional `[--trunk <ref>]`)
gets a success line and an unmoved trunk, with no signal. Fix: the success line
should state the trunk/edge outcome explicitly (e.g. `trunk refs/heads/main
7f8692f→12dda24 (ff)` or `trunk: untouched (no --trunk)`), and ideally warn when an
admitted `close_target` exists but no `--trunk` was given (a probable no-op landing).

**2. `--integrate --trunk <ref>` moves the ref but leaves a checked-out working tree
and index stale.** When trunk is the currently-checked-out branch, the plumbing ref
move is not reflected in the work tree — `git status` then shows every just-landed
file as deleted/modified-away, and the closer must hand-run
`git restore --source=HEAD --staged --worktree <landed paths>` (taking care not to
disturb unrelated WIP). Fix: detect that the projected `--trunk` is the checked-out
HEAD and either (a) print a resync hint naming the landed paths, or (b) advise
running from a detached/parent tree, consistent with the "runs from parent/root
after the coordination worktree is removed" guidance already in the help.

Scope note: usage-carriage of the close-via-candidate-integrate workflow into
skills / shipped memories is already owned by CHR-009 → SL-069; this item is the
*tooling smoothing* half. The `--trunk`-required nuance is also captured in
[[mem.pattern.dispatch.close-lands-via-candidate-integrate-trunk]].

Cross-ref: SL-068 (close), CHR-009, `src/dispatch.rs` (integrate path).
