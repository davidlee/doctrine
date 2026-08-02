`[ -x path ]` is true for a binary whose **ELF interpreter is absent**. On NixOS
that is an everyday state, not a corner case: a build made against one nix store
generation is `-x` in a jail mounting another, and **exits 127 on every call**.
`readelf -l <bin>` names the interpreter it wants; check that the path exists.

On a shared worktree this arrives without warning. A co-agent rebuilding
`target/debug/doctrine` silently replaced the apparatus another agent's
75-minute measurement run was using, mid-run.

**Two rules follow, and the second is the expensive one.**

1. **Probe runnability, don't test the bit.** A resolver ladder must exec its
   candidate (`"$bin" --version >/dev/null 2>&1`) before returning it. Otherwise
   it takes the dead rung and never reaches the working one — which in this repo
   is already provided: `flake.nix` ro-binds the crane build at
   `~/.cargo/bin/doctrine` and puts it on PATH.

2. **A tool that cannot be INVOKED is a defect of the harness, never a verdict
   about the subject.** `cmd … >/dev/null 2>&1 || refuse 'undeclared-path'`
   maps exit 127 onto a substantive refusal and blames the thing being measured.
   Key on 126/127 explicitly, raise a defect, and **keep the stderr** — the
   sentence `2>&1` discards (`bad interpreter: No such file or directory`) is
   the only one that says which failure it was. Cost when this was missed:
   32 reported failures, a wrong finding blaming the measurement vehicle, and a
   session boundary.

Same shape as [[mem.fact.git.clone-inherits-stale-commit-graph]]: a refusal
token standing for two causes with the distinguishing evidence thrown away.

**Signature:** every invocation exits 127 · the file is `-x` · the failure
vanishes with a different binary on the same inputs · reds cluster by
wall-clock, not by subject.


**Tension with [[mem.pattern.doctrine.worktree-target-and-stale-path-doctrine]]**
(`mem_019edf8f57d2726281fcddd36d5197b1`), which warns that PATH doctrine is
STALE. Both hold and they answer different questions: PATH may lag the tree
(currency), and may also be the only rung that RUNS (runnability). Falling
through to it is right; believing it blindly is not. **Check the version, and
check whether the verbs you depend on actually changed** — `git log <bump>..HEAD
-- <the source files that implement them>`, with a positive control so an empty
result is not mistaken for an answer.

That sibling's first clause is **stale**: worktrees no longer share
`CARGO_TARGET_DIR` (SL-156, ADR-008 D-B1) — each builds into its own in-tree
`target/`. Worth a `/reviewing-memory` pass.
