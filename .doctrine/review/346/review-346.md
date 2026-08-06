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

### Round 5 — the remediation's own surface, and one ruling

**Subject.** `.doctrine/slice/248/design.md` at design-run revision 73,
watermark `9397c87c`, materialised. Round 4's four findings (`F-25`…`F-28`) are
remediated, and the remediation was then extended by an author-side pass
(`12c6c4095`, `ceab419bb`) that found five further defects by reading the whole
document cold rather than by adversary. `sec-2`, `sec-3`, `sec-6`, `sec-7` and
`sec-8` moved; `sec-1`, `sec-4`, `sec-5` and `sec-9` did not.

**What is closed, so you do not re-spend a round on it.** Round 4's brief has
had its pass: table A row 9 and `sec-2` invariant 11, `execute_observed` and its
`HostPid` callback, table C's split capacity claim, and the scope reconciliation
are all read. `sec-6` was read too — and immediately produced a third
build-level trap, which is why it returns below rather than retires. A further
read of `sec-1`, `sec-4` or `sec-5` buys nothing.

**Standing discipline, unchanged.** Build-level and confinement claims are
verified by **execution on a minimal reproduction**. A negative read is not
evidence, and a probe that fails in your own sandbox has not yet said anything
about the subject — establish which cage produced the failure before raising it
(`F-20`).

**Lines of attack.**

1. **The descriptor mechanism (`sec-2`, `design.md:730`).** `F-26` raised the
   missing table *row*; the *implementation* was written after round 4
   concluded and has been read by nobody. The parent enumerates `/proc/self/fd`
   before the fork and sets `FdFlags::CLOEXEC` on everything above 2 through
   `rustix::io::fcntl_setfd`. Three attacks. Is enumerate-and-mark in the parent
   actually equivalent to the post-fork close it substitutes for — `rustix`
   1.1.4 carries no `close_range` and allocation between `fork` and `exec` is
   unsafe, so the alternative is unavailable, but unavailability is not
   soundness. Can table A row 10 fail? Rust opens its own handles `O_CLOEXEC`,
   so a payload enumerating descriptors proves nothing unless the decoy is
   opened *deliberately without* `O_CLOEXEC` — the design says so at `:852`;
   check the row's own control text forces it rather than assuming it. And does
   anything open or inherit a descriptor between the sweep and `exec`?

2. **`sec-6`'s rustix edge, and `sec-6` as a class (`sec-6`, `sec-8`).**
   `rustix`'s `std` feature reaches the root package only by feature
   unification — the manifest declares `default-features = false, features =
   ["fs"]`, and `std` arrives from `crossterm` and from dev-only
   `which`/`tempfile`. `doctrine-control` depends on none of the three, so it
   must declare its own edge; without it `rustix::fd::AsFd` is a `no_std`
   polyfill that `std::fs::File` does not implement and every call site fails
   `E0277`. That is the third defect this section has hidden that only
   *executes* into view, after `R6`'s double compilation and `R7`'s
   package/build divergence. So the line is not to re-read `sec-6`. It is to ask
   what else in the crate topology is asserted from the root package's manifest
   and does not transfer to a second workspace member — and to build it.

3. **Whether any other control is unfalsifiable rather than merely unremoved
   (`sec-7`, `sec-3`).** Table B's `B4` targeted `/tmp` for four rounds. `/tmp`
   is `--tmpfs`: anonymous, fresh per `execute`, backed by nothing under the
   transaction root. `SharedRoot` re-points a placement's root and so reaches
   only *declared* writable entries — it cannot reach `/tmp` at all. Both arms
   saw an empty tmpfs, so the row could only ever read `Unproven` and
   `Admission::Admitted` was unreachable. B4 now targets the retained scratch at
   `/capsule/tmp`, and the two areas are named *transient* and *retained*
   (`design.md:1067`–`1083`) because one name covering both is what hid it. The
   class is the mirror of `F-25`: a rule checked in one direction only is half a
   rule. Sweep tables A, B and C for the same shape — any row whose control
   delta cannot change what its payload observes.

4. **A ruling the design declines to make, and hands to you.** Table A row 1 and
   B1–B4 overlap on `/capsule/tmp` and `/agent` under the same `SharedRoot`
   delta (`design.md:3280`). The design argues the overlap is intended: the two
   tables discharge different requirements — row 1 is `SPEC-030`'s backend
   property, B1–B4 are `REQ-450` criterion 1's named axes — and the non-overlap
   rule binds a row's *control* to a unique mechanism, not its payload to
   untouched ground. Row 1 was **not** narrowed, deliberately, because narrowing
   it would undo `F-27`'s round-4 strengthening. This is put to you as a ruling
   rather than left as an author's assertion: is that distinction real, or a
   rationalisation that leaves two rows unable to say which of them failed?

5. **The next mutant.** Permanent, highest-value, and now with a sharper prompt
   than *build the backend the suite would wrongly pass*. B4 shows a suite has
   two ways to fail, not one: a gap that admits a bad backend, and a control
   that changes nothing and so can never withhold admission from anything. Build
   both — the backend the suite wrongly admits, and the row whose admission the
   suite is incapable of refusing.

6. **Process credentials — the channel ledger's own blank cell (`sec-2`,
   `sec-9` residual 5).** Round 4's channel ledger closes the enumeration of
   ways authority crosses `execute`, and its residual names three channels it
   lists without rowing: `argv` (closed by construction rather than by probe),
   process reachability (split across row 7 and `B5`), and **process
   credentials** — uid/gid mapping, supplementary groups, capabilities,
   `no-new-privs` — for which `SPEC-030` states no clause at all. A capsule that
   inherited the host uid with capabilities retained would satisfy all eleven
   rows. The design nominates this as where the next reviewer should start.

### Round 6 — verification of round 5, and one row that cannot fail

**Subject.** `.doctrine/slice/248/design.md` at design-run revision 75,
watermark `ec973c1e`, plus the author-side commits made after it and before this
brief: `48bc59a44`, `9ffe8a1d6` and `b504edc95`. Round 5's five findings
(`F-29`…`F-33`) are all remediated and answered. This is primarily a
**verification pass** — the remediations are the subject, not the sections
around them — with one substantive open decision at 1.

**What round 5 changed.** `Execution` gains a `stdio` field over a closed
`CapsuleStdio` vocabulary; `sec-2` gains invariants 14 and 15 and two channel
ledger entries; `sec-7` gains table A rows 12 and 13, `PropertyRemoval`'s
`StdioOwned` and `CredentialsConfined`, and a widened row 10; `sec-8`'s
`REQ-459` claim drops *in full* for *over the channels the ledger names*;
`sec-9` gains a `SPEC-030` Revision as deferred follow-up. `DEC-156` moves
eleven → thirteen, and the ceremony around moving it is retired.

**What the post-revision commits did, and why you are being told.** A pre-brief
sweep found **ten** defects and fixed them; four of the commits above are that
sweep. Eight are `F-28`'s class — a round-5 correction reaching the sections
under review and missing ones that were not, repeated inside the section that
documents `F-28`. `sec-2`'s properties table stopped at row 11; `sec-2` still
read *rows 2, 9, 10 and 11 are clause 2*; `sec-2` had no `CapsuleStdio` pure test
while `sec-8` claimed it did; a test title asserted the opposite of its own row;
`sec-9`'s `R3` was stale in three ways; `DEC-156`'s record still ended at eleven;
the `RowId` comment still said *clause 2 carries four*; and `sec-7`'s executed
table A list had no `_is_proven` title for rows 12 or 13, so a literal
implementation would have built a suite that never ran either new row. The other
two are at 1 and 2a below and are older than round 5. **Do not re-spend the
round finding any of these.** The questions worth your time are the remedy at 1,
the rate at 2a, and the standing question at 6.

**A note on what that implies about this subject.** Ten defects found by an
author-side sweep, in a document that had just had five external rounds, is
itself evidence — `sec-9` `R3` says the gap in a property suite is invisible from
inside it, and the same appears to be true of this design's bookkeeping. Weight
your reading accordingly: the prose arguments have been read hard and repeatedly;
the tables, lists, counts and citations that those arguments quantify over have
not.

**Standing discipline, unchanged.** Build-level and confinement claims are
verified by **execution on a minimal reproduction**. A negative read is not
evidence, and a probe that fails in your own sandbox has not yet said anything
about the subject — establish which cage produced the result before raising it
(`F-20`).

**Lines of attack.**

1. **Row 13's control cannot fire — recorded in `sec-7`, and the remedy is
   yours.** This is not a discovery you are being asked to make; it is a
   measurement you are being asked to refute or build on.
   `CredentialsConfined`'s delta drops `--unshare-user`, and unprivileged
   `bwrap` creates a user namespace regardless, so the flag is a no-op. Measured
   on bubblewrap 0.11.2: both arms byte-identical on uid, gid, supplementary
   groups, `CapPrm`/`CapEff` and `NoNewPrivs`; both created a **new** userns with
   an identical `uid_map` differing only in inode, which is the control that
   makes the result bubblewrap's rather than this jail's (`F-20`). Row 13 can
   only read `Unproven`, so `Admission::Admitted` is unreachable — `B4`'s class,
   in the row added one round after `B4`. `F-32`'s argument for the *property* is
   untouched and is not reopened. **The open question is the mechanism**: what
   delta falsifies a credential posture under a backend that will not let you
   drop the namespace the confinement rests on? Candidate shapes the design has
   not evaluated — a probe-side `--uid`/`--gid` the control omits; a setuid-free
   capability assertion the control cannot satisfy; a second backend stub. Rule
   on one, or say the row must be re-cut to a property bubblewrap can fail.

2. **Row 13's probe conditions, same session, also recorded.** Two of four are
   false against the design's own backend — the profile passes no `--uid`/`--gid`
   so the uid is the host's rather than a mapped identity, and `--unshare-all`
   cannot clear supplementary groups since a userns may not `setgroups` — and a
   third, `no_new_privs`, is a bubblewrap default carrying no signal for a
   namespace delta. Verify, and treat this as bounded evidence about the *probe*:
   if the remedy at 1 re-cuts the row, these conditions are re-cut with it.

2a. **The `EVD-013` over-attribution, and whether the sweep behind it was
   complete.** `sec-7` captioned all ten deltas as measured and then attributed
   `EVD-013` to five of them; `EVD-013` measures one, and there were six rows
   above the last four, not five. Four deltas rode a citation that never touched
   them, and `R1` — the risk that says every measured claim is attributed where
   it was measured — asserted the opposite of what was true. Both are corrected;
   the split is now one measured, nine reasoned. **The line of attack is the
   class, not the instance.** Every other evidence citation in this document
   (`SL-241` `EX-N` references, `sandbox.sh` line cites, the `16/16` summary,
   `jail.rs`/`git.rs`/`pretooluse.rs` line numbers) is a claim of the same shape.
   Read a sample against the records and tell us the rate.

3. **Row 12's second leg may not be well defined (`sec-2`, `sec-7`).**
   `CapsuleStdio::EmptyInputCapturedOutput` makes 1 and 2 *one-way endpoints the
   parent created and reads*, and the stated reason for one-way is that a socket
   pair would carry *bytes back into the trusted side*. But a one-way capture
   pipe carries bytes into the trusted side too — that is what it is for. The
   property that actually distinguishes a socket pair is that the capsule can
   **read** descriptor 1. Row 12 then holds when *the write-back does not reach
   the parent*, while the parent reads `stdout` by construction. Is the row
   observing a distinction that exists? Should the probe assert that descriptor
   1 is not readable rather than that a write does not arrive?

4. **`F-31`'s rule, turned on the rest of the tables.** The remediation states
   it generally — *a row must observe the property, not a consequence of the
   property that happens to be convenient to observe* — and applies it to row 10
   alone. Round 5 punished exactly that shape in `F-29`. Sweep tables A, B and C
   for other rows whose holding condition is a consequence with innocent
   explanations rather than the property itself. Row 3's *both reads fail* and
   row 9's *every write fails* are the obvious candidates: a read that fails
   because the path is absent, and a read that fails because the payload was
   built wrong, are not distinguished by the observation.

5. **`F-29`'s remediation is a table, and the design says not to trust it.**
   `sec-6` now enumerates four edges — `doctrine`, `rustix`, `serde`, `toml` —
   and instructs the phase to derive the list from the crate's own `use`
   statements and treat disagreement as the table being stale. Build the
   manifest against what `sec-2`, `sec-4`, `sec-5` and `sec-7` actually specify
   the modules calling, and report what the table is still missing. `sec-7`'s
   conformance harness is the least-read consumer.

6. **Whether the drift is fixable structurally, which is the standing
   question.** Eight instances in one round, after `F-28` raised the same class
   and after the count was moved to a single source to prevent it. The
   single-source rule worked for the *numeral* and did nothing for the row
   contents, the invariant list, the test titles, or the record. The design's
   own proposed guard — `every_row_id_is_covered_by_exactly_one_table` — is a
   test that does not exist yet, so it protected nothing. Is there a structural
   answer available at design altitude, or is the honest ruling that a document
   of this size maintains parallel enumerations by reader discipline and the
   design should say so where it currently implies otherwise?
