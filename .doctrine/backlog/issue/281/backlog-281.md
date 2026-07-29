# ISS-281: vt9 rootless premise is environmental: a marked TMPDIR ancestor makes it pass for the wrong reason

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found 2026-07-29 while closing [[ISS-220]] — a new test asserting "an unmarked
anchor discovers no root" reddened with `left: Some("/tmp")`.

## The defect

`memory::ambient_surface_tests::vt9_no_discoverable_root_emits_nothing` states its
premise in its own doc comment: "a resolvable cwd with no root marker above it".
It builds that premise with a bare `tempfile::tempdir()`.

A tempdir has no marker *in* it, but `root::find_from` walks **up**, and
`default_markers()` is `.git` / `.jj` / `.project` / `Cargo.toml`. So the premise
holds only if no ancestor of `TMPDIR` carries one of those — which is not a
property the test controls.

On this host it does not hold. A stray `/tmp/.git` (a bare-repo skeleton:
`config`, `description`, `HEAD`; created 2026-07-24, provenance unknown — likely
an agent accident) makes `/tmp` itself resolve as a root:

```
discover_surface_root(Some(<tempdir>), _) => Some("/tmp")
```

vt9 nevertheless **passes** — with `root = /tmp`, `retrieve_rows` finds no memory
corpus there, so the emitted block is empty and the `out.is_empty()` assert holds.
It passes on the emptiness of `/tmp`, not on the absence of a root.

## Why it matters

This is the same failure *class* ISS-220 was about, one layer out: a test whose
stated premise is not what the assertion exercises, passing for an ambient reason.
ISS-220's original masking was a stale seen-set; this one is a marker above
`TMPDIR`. Both make a green test carry less signal than it appears to.

Concretely: no test covers "a root was discovered but the corpus yielded nothing"
versus "no root was discovered" — vt9 claims the second and, here, exercises the
first. The `find_from` miss path is unasserted.

## Not urgent

ISS-220's fix pinned the arms that actually matter directly and hermetically
(`resolvable_cwd_wins_over_the_env_anchor`,
`no_usable_anchor_on_either_arm_yields_none` — the latter fails canonicalization
on both arms and never reaches the walk). So the seam has real coverage now and
this is a fidelity gap in one end-to-end test, not an uncovered behaviour.

## Fix directions

- Assert the *reason* rather than the byte-emptiness — have vt9 check that
  discovery returned `None`, not merely that nothing was emitted. Cheapest, and
  it converts an ambient pass into a real one.
- Or give the rootless case an anchor whose ancestry the test owns: a tempdir
  plus a walk ceiling, if `find_from` grows one.
- Not a fix: deleting `/tmp/.git`. It makes the test pass here today and says
  nothing about the next host. (Worth deleting anyway as host hygiene — it is
  outside the repo, so out of scope for this item.)

Related: [[ISS-220]] (the ambient-env false-red this was found while closing);
[[IMP-196]] (`cluster:testing-goldens` — golden hermeticity lint; the same
"test reads ambient state" family, at a different altitude).

## Resolution

Fixed 2026-07-29 on `edge`, `2217fc2d`. Three changes, and the largest was not
in the fix directions above.

**1. `root.rs` had no tests at all.** `find_from` — the walk this whole issue is
about — was unasserted at its own altitude. That is where the miss path belongs,
and it turns out to be *hermetically* assertable there for a reason neither fix
direction spotted: `find_from(start, markers)` takes its **markers as a
parameter**. A test can inject `.iss281-test-marker`, which no real ancestor
carries, and own its entire tree — no `TMPDIR` ancestry assumption, no
`marker_free_base` candidate scan, no flake surface. Six tests now cover
start-inclusive, walks-up, nearest-wins, the miss path, any-of-many markers
including a file marker (`.git` is a file in a linked worktree), and the empty
marker set.

**2. The env read moved up one more level.** `run_surface_to` now takes
`env_project_dir`; `run_surface` — the impure boundary that already reads stdin —
reads the env and passes it down. This mattered because ISS-220's fix stopped one
level short: `run_surface_to` still read process env, so *any* test driving it
with an unresolvable cwd fell through to the ambient anchor. The 12 test call
sites route through a documented `surface` helper passing `None`, so the module is
now **structurally** unable to read process env rather than incidentally not
doing so.

**3. vt9 asserts its premise** — unresolvable cwd, no ambient anchor, both arms
failing at canonicalization before the walk is reached. This is fix direction 1,
and it is the smallest of the three.

Mutation-verified rather than assumed. Reintroducing an internal ambient env read
in `discover_surface_root` reds vt9's premise assert and
`no_usable_anchor_on_either_arm_yields_none`, naming the leaked root — the exact
regression this closes. Skipping the start dir in `find_from` reds
`start_dir_itself_is_a_candidate`; returning the outermost hit instead of the
nearest reds `nearest_marked_ancestor_wins`.

**Honest limit:** the reintroduced-env-read mutation is only observable when
`CLAUDE_PROJECT_DIR` is actually set during the run. That is the worker-fork / CI
condition where the bug bit, not a bare local run — so the guard holds where it
matters but a purely local suite would not catch that specific regression.

`doctrine check gate` green (clippy zero warnings); ambient module 19, root 6.

`/tmp/.git` was **not** deleted — out of scope as recorded above, and still worth
deleting as host hygiene.

Durable capture: [[mem.pattern.testing.no-root-find-walk]] already recorded this
hazard and the `marker_free_base` workaround; it now also carries the
injected-marker technique, the canonicalize-fails option, when each applies, and
the recurrence. It could not be re-attested — `memory verify` refuses on a dirty
tree and this worktree carries other agents' uncommitted work (friction recorded;
work homed at [[IMP-221]]).
