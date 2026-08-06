# Review RV-346 — design of SL-248

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

### Round 3 — sec-8 and sec-9

**Subject.** `.doctrine/slice/248/design.md` at design-run revision 60. The
review target is `sec-8` (code impact and verification alignment) and `sec-9`
(risks, residuals, and what stays open), both newly written and neither
externally read. `sec-6` and `sec-7` have also never had an external pass;
findings there are in scope **where they bear on sec-8's or sec-9's
correctness**, and out of scope as a general sweep.

**Standing discipline for this slice.** Build-level and confinement claims are
verified by **execution on a minimal reproduction**, not by reading. Round 1's
`F-1` (bubblewrap dereferences a `--ro-bind` source path) was found by running a
probe against flags that read as if they said the opposite. A negative read is
not evidence here.

**Lines of attack.**

1. **Is the modified-paths table actually the whole diff?** `sec-8` claims it
   is, and `sec-6` invariant 2 asserts it. Candidates it does not name:
   `Cargo.lock`, `clippy.toml`, `.github/workflows/*`, the `[package] include`
   allow-list, `justfile` recipes, `.gitignore`, `flake.nix`. Each absence is
   either correct or a finding.
2. **The `default-members` ruling.** `sec-8` claims one key gets `cargo fmt`,
   `clippy`, `build` and `test` onto the new crate with no recipe edit, and
   leaves `crates/cordage` where it is. Verify by execution. Then check what
   else it moves: `cargo package` / publish, `cargo install --path .`,
   `cargo binstall`, the crane/nix build in `flake.nix`, and any recipe that
   assumes `cargo build` produces one binary.
3. **A package with both a bin and a lib.** `sec-6` adds `src/lib.rs` to a
   package whose bin is also named `doctrine`. Does that build, and does it
   need an explicit `[lib]` or `[[bin]]` section? `sec-6`'s three
   execution-verified claims (E0364 on re-exporting `pub(crate)`, same module
   in both targets, the `crate::kinds` E0432 under the library's own test
   build) are stated as confirmed — re-run them if cheap, and say so either way.
4. **`R7`, the published library API.** `sec-9` claims `src/lib.rs` falls inside
   the published `include` list and that the `doctrine` package therefore
   acquires public semver surface at its next release. Check the claim and the
   rejected alternative.
5. **The requirement closure table.** `sec-8` moves `REQ-449` and `REQ-461` to
   `satisfied` and holds `REQ-448`, `REQ-450`, `REQ-459` as contributing
   changes. Read the requirements' actual criteria against the evidence each
   section offers. Two specific doubts: `REQ-461`'s only executed leg may report
   *skipped*, and `sec-8` attributes `REQ-448`'s denial half to table A "rows
   1–5" — check that against what `sec-2` and `sec-7` actually say.
6. **`R4`'s retirement.** `sec-9` retires the `bwrap_core_argv` byte-parity risk
   on the ground that `src/worktree/` is unedited under `DEC-155`. Find anything
   the capsule backend needs that only `jail.rs` has.
7. **The `#[ignore]` versus `--skip` distinction.** `sec-8` argues a recipe-level
   test filter is lawful under `DEC-156` while `#[ignore]` is not. Is that a real
   distinction or a rationalisation of a convenient escape hatch?
8. **`sec-9` residual 1, the resolution-time race.** It is dismissed as not
   capsule-reachable on the strength of `sec-2` invariant 4. Check the invariant
   covers what the dismissal needs it to cover.
9. **Internal contradiction.** Anything `sec-8` or `sec-9` asserts that
   `sec-1`–`sec-7` contradicts. `sec-8` already corrects the slice's own scope in
   four places; look for a fifth it missed, and for a correction it got wrong.
10. **The indicative counts** in `sec-8`'s evidence table. Labelled indicative,
    but a count that is wrong by a lot is a phase-sizing defect.

### Round 4 — sec-2 and sec-7 as amended, and sec-6 unread

**Subject.** `.doctrine/slice/248/design.md` at design-run revision 67; section
bodies are unchanged since the round-3 remediation (`79d06645`). The design run
has moved to stage `reviewing`, so this is the pass that decides whether the
document locks.

**This round is deliberately narrow, and the narrowing is the point.** `sec-1`,
`sec-3`, `sec-4` and `sec-5` have been through rounds 1 and 2 and were not
amended by round 3. Findings there are welcome if they are real, but no effort
is asked for them. The three surfaces below are where the unread material is.

**Standing discipline, unchanged from round 3.** Build-level and confinement
claims are verified by **execution on a minimal reproduction**, not by reading.
A negative read is not evidence.

**One correction from round 3 that bears on how you work.** `F-20` reported
that this project's jail cannot create a nested network namespace, and it did
not survive its positive control: the exact profile and its three controls all
succeed one level down in this jail. What produced the failure was the
*reviewing agent's own sandbox* wrapping the jail, whose filter denied the
socket bubblewrap opens for loopback. If a probe you run fails, establish
whether the failure is the subject's or your cage's before raising it. The
general mechanism was kept and `sec-9` residual 3 now covers any such layer.

**Lines of attack.**

1. **Table A row 9 and its control (`sec-7`, amended).** Round 3's `F-19` added
   a ninth property — declared inputs bound *immutably*, distinct from the input
   set being *bounded* — because no earlier row ever wrote to something it had
   asked to be read-only, so a backend binding every declared input writable
   passed the whole table. Row 9 and its `InputsWritable` control have been read
   by nobody. Does the control remove the mechanism unique to row 9 and nothing
   else? Does the row's probe fail for the reason the row claims?
2. **`ConformanceBackend::execute_observed` and its `HostPid` callback
   (`sec-2`/`sec-7`, amended).** New surface, unread. `sec-9` residual 2 notes
   that `ConformanceBackend` now carries two methods, which widens what an
   out-of-crate backend would have to expose. Is the second method necessary,
   and is the callback's contract stated tightly enough that a backend cannot
   satisfy it while lying about which namespace the pid names?
3. **`sec-2` invariant 11.** Added in round 3 to carry the property row 9
   proves, and now load-bearing for `sec-9` residual 1's dismissal of the
   resolution-time race — a dismissal that rested on the wrong invariant until
   `F-19`. Check that invariant 11 states what the dismissal needs, and that
   what row 9 executes actually establishes invariant 11.
4. **A tenth mutant.** This is the highest-value line and it is not a re-read.
   `R3`'s standing form is that the gap in a property suite is invisible from
   inside it: `F-2` and `F-19` were both found by constructing the backend the
   suite would wrongly pass, never by inspecting the rows. Construct the next
   one. Candidates worth trying: a backend that satisfies every row while
   leaking a file descriptor across the boundary; one that binds correctly but
   makes the working directory nondeterministic in a way row 6 does not see; one
   that reaps the process tree by killing the group and misses a
   double-forked grandchild; one that reports termination facts truthfully but
   observes them from a source the trusted side cannot verify.
5. **`sec-6`, which has never had an external pass at all.** The crate topology,
   the five-item export set, and the two-tree layering gate. Every other
   section's file placement rests on it, and both design-created risks (`R6`,
   the double compilation; `R7`, the package/build divergence) were created
   there. Round 3 touched it only where `sec-8` bore on it.
6. **Table C's split capacity claim (`sec-7`, amended).** `F-24` established
   that `REQ-461`'s only executed leg skipped wherever the capsule root and the
   repository share a filesystem, so the claim was split into an unconditional
   leg and a discriminating one. `REQ-461`'s move to `satisfied` now rests on
   the unconditional leg. Does that leg discriminate — would it fail against a
   `SystemHost` that returned a plausible constant?
7. **The slice scope, now reconciled.** `.doctrine/slice/248/slice-248.md` was
   rewritten against the design after round 3 (commit `d5ab123a9`): `REQ-459`
   dropped from `satisfied` to a contributing change, the touch-set replaced
   with `sec-8`'s, `A3` refined, `R4`/`R5`/`OQ-4` added. `sec-9`'s corrections
   list is what it was reconciled from. Did the reconciliation miss one, or
   introduce a claim the design does not support?
