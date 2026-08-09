# SL-248 — execution record, PHASE-10 … PHASE-12

Shard per `LOOP.md` § *Notes, sharded*. What was done, what diverged from the
phase sheet, what was measured. Cross-phase items and anything owed to the
reconciliation brief stay in `notes.md`.

**Minted late, and deliberately.** `PHASE-10`'s sheet nominated this shard from
`T1` onward, and also forbade minting one silently — so four sittings (`T1`,
`T2`, `T4`, `T9`) correctly took the sheet's own fallback and appended to
`notes_07-09.md`, each restating the same finding (`F-13`, `F-27`, `F-50`).
Four identical restatements is the signal that the *nomination* was the defect,
not the sittings. The orchestrator minted this shard and moved `PHASE-10`'s
records across whole; nothing was rewritten, and the section headings below are
the ones those sittings wrote.

## PHASE-10 `T1` — `VA-2`: the four reasoned deltas, measured

**Shard note.** `phase-10.md` nominates `notes_10-12.md` as PHASE-10's shard and
forbids creating one silently. No such file existed at this sitting, so PHASE-10
appends here, per the sheet's own fallback.

`weakening_for` (`conformance.rs:1736`) carries `InputsWritable`,
`DescriptorsClosed`, `EnvCleared` and `StdioOwned` as **reasoned** (`EX-18`).
Rows 9–12 each stake a control arm on one. `VA-2` requires each shown to fire
*before* the row depends on it, and `S3` says a delta that does not produce its
row's control failure means the row is wrong.

Measured through the shipped seam — `provision_capsule` → `harness_execution` →
`ConformanceBackend::execute_noticing` under `Under::Confining` versus
`Under::Removing(…)` — so the argv exercised is the one `weakening_for` actually
emits, not a shell approximation of it. **All four discriminate.** `S3` is not
triggered and no row is withdrawn.

### The tally (`C14`, `F-36`): 5 runs per delta per arm, 32 spinners on 32 cores

| delta | probe reading | control reading | *n*/5 |
|---|---|---|---|
| `InputsWritable` | `WROTE-NOT /source` (`Read-only file system`), `NOT-WRITABLE /bin`, `NOT-WRITABLE /nix` | `WROTE /source`, `WRITABLE /bin`, `WRITABLE /nix` | 5/5 both arms |
| `DescriptorsClosed` | `/proc/self/fd` = `0 1 2 3` (3 is the enumeration's own handle) | `0 1 2 3` **plus** `5 → decoys/descriptor`, `6 → #NNNNNN (deleted)`, `7 → socket:[…]` | 5/5 both arms |
| `EnvCleared` | 10 variables — exactly `CapsuleEnv` | 42 variables — the whole trusted-side environment | 5/5 both arms |
| `StdioOwned` | `STDIN[]` | `STDIN[TRUSTED-SIDE-DECOY-INPUT]` | 5/5 both arms |

Every arm exited `Exited { code: 0 }`; no run went indeterminate, and no 4/5.
Wall clock per spike run under load: 0.85–1.86 s.

### The exact argv delta each produced

Measured off `confinement_argv` (`bubblewrap.rs:1106`), confining versus
weakened, over one real placement — plus `SpawnOptions::under`, which is where
the two parent-side axes live.

| delta | argv words | change |
|---|---|---|
| `InputsWritable` | 40 → 40 | three `--ro-bind` → `--bind`, at positions 14, 17, 20. **No word added, none removed, no path changed.** |
| `DescriptorsClosed` | 40 → 40 | **none — the argv is byte-identical.** `SpawnOptions.descriptors_closed` `true` → `false`. |
| `EnvCleared` | 40 → 39 | `--clearenv` removed. The `--setenv` list is untouched. |
| `StdioOwned` | 40 → 40 | **none — the argv is byte-identical.** `SpawnOptions.parent_owned_stdio` `true` → `false`. |

Two of the four change **no argv byte at all**. That is not a weakness: it is
what makes `the_descriptor_control_changes_no_mount_no_env_and_no_argv_byte`
(`VT-3`) and `the_stdio_control_changes_nothing_above_descriptor_two` (`VT-5`)
assertions about a measured fact rather than restatements of an intention. And
`InputsWritable`'s 40→40-with-three-substitutions is
`the_writable_inputs_delta_changes_no_mount_and_no_path` (`VT-2`) measured: the
mount set, its count, its inner destinations and its host paths are all
identical across the arms, and only the attachment moves.

### `F-9` — the descriptor decoys must be opened **after provisioning**, not merely per arm

`F-6` predicted a vacuous row 10 and named the probe arm's sweep as the cause.
The measured cause is one layer broader and it matters for `D1`.

Opening the decoy set *before* `provision_capsule` and holding it across both
arms — which is what "per fixture" and a naive "per arm" both amount to — reads:

    STALE  probe   = 0 1 2 3
    STALE  control = 0 1 2 3

The control sees **nothing**. Opening a fresh set *after* provisioning and
before the spawn reads:

    probe   = 0 1 2 3
    control = 0 1 2 3 5 6 7

Provisioning itself runs capsules, and every one of those runs goes through
`fork_within_the_descriptor_window` with the sweep enabled, marking every
descriptor above 2 close-on-exec process-wide. So a decoy opened before
`(Arm.capsule)()` is already `CLOEXEC` by the time the arm forks, whichever arm
it is. **A per-arm hook is necessary and not sufficient; the hook has to sit
between the capsule closure and the spawn.** That is the placement `T2` adopts.

### `F-10` — `bwrap` does not close inherited descriptors, so the mechanism is the parent's sweep alone

Checked directly, because the alternative explanation for `F-9`'s stale reading
was that bubblewrap closes what it inherits and the delta could never fire:

    exec 9</tmp/decoy; bwrap --unshare-all … /bin/sh -c 'ls /proc/self/fd'
    → 0 1 2 3 9=>/tmp/decoy

Descriptor 9 survives into the capsule. Bubblewrap 0.11.2 passes inherited
descriptors through untouched, so the whole of row 10's confinement is
`mark_inherited_descriptors_close_on_exec` on the trusted side — and the whole of
its control is skipping that call. Worth knowing before anyone reads a
non-discriminating row 10 as a bubblewrap fact.

### `F-11` — `InputsWritable`'s control arm makes the **system readable roots** writable, outside the fixture

The delta is *"`--ro-bind` becomes `--bind` for `/source` **and every declared
readable entry**"*. On this host the declared readable entries are
`system_readable_roots`' output: `/bin` and `/nix`. Under the control arm a
payload that writes through them writes onto the operator's filesystem, outside
the fixture's `TempRoot`, where nothing reclaims it — `Fixture::Drop` reaches
only its own root and this slice adds no unlink (invariant 8).

Measured, not reasoned: the first run of this spike wrote through every readable
mount and left `/nix/va2-write` and `/bin/va2-write` on the host, owned by the
operator. **Swept by hand**, and the spike was then narrowed to write only into
`/source` — this run's own export, inside the fixture root — and to probe the
other entries non-mutatingly with `[ -w ]`, which discriminates just as cleanly.

This is `T3`'s problem, and it is exactly what
`the_writable_inputs_control_writes_only_to_this_runs_own_export` (`VT-2`) is
for. Note the tension it names: a `Row` carries **one** `ArmShape`, so the probe
and the control run the *same* payload. `a_write_through_every_readable_mount_fails`
therefore cannot be satisfied by a payload that literally writes through every
readable mount — that payload, on the control arm, writes into `/nix`. Row 9's
payload has to write only into the source export and establish the other entries'
read-onlyness by a non-mutating read. Recorded before `T3` starts so it is a
design constraint rather than a discovery made by an operator finding files in
their nix store.

### `F-12` — `EnvCleared`'s control arm hands the capsule live operator credentials

Probe: 10 variables, exactly `CapsuleEnv`'s set. Control: 42, the trusted side's
whole environment. On this host that set includes several third-party API keys
belonging to the operator, in full, readable by the capsule.

No action is owed on the *delta* — leaking is precisely what the control exists
to demonstrate, and it discriminates beautifully. What is owed is on the
**payload**: row 11's probe is set equality against `CapsuleEnv`, and its
reporting must be **names and counts, never values**. A payload that printed
`env` verbatim on failure would put the operator's credentials into the test log
and into CI output, which is a worse disclosure than the one the row exists to
prevent. The spike was rewritten to print names only after the first run proved
the point.

### `F-13` — reopening `/dev/fd/1` does not test whether the capsule can read descriptor 1

The first stdio spike asked `head -c 1 /dev/fd/1` and read `FD1-READABLE` under
**both** arms, which would have looked like row 12's second leg failing to
discriminate. It is an artefact twice over: opening `/proc/self/fd/N` on a pipe
*reopens the pipe* rather than duplicating the descriptor, so a read end is
obtainable even where the descriptor held is write-only; and the payload had
already written its own output into that pipe, so the read returned the payload's
own bytes.

`F-30`'s claim — that what a socket pair confers and a capture pipe's write end
does not is the capsule's ability to **read descriptor 1** — is a claim about the
inherited descriptor, so `T6` must read descriptor 1 *itself* (`<&1`), never
`/dev/fd/1`, and must bound the read: under the control arm descriptor 1 is a
socket whose peer the trusted side holds open across the run, so an unbounded
read blocks until the wall bound rather than returning (`R1`). The stdin leg is
unaffected and was measured clean; this bears only on the descriptor-1 leg.

## PHASE-10 `T2` — the per-arm trusted-side setup seam (`D1`)

*Same shard-fallback note as `T1`: `notes_10-12.md` does not exist and the sheet
forbids minting one silently, so `T2`'s record lands here.*

### `D1`, settled — the setup rides the capsule closure, keyed on the delta

`arm_over` composes `trusted_side_setup(&row.delta, fixture)` onto the capsule
closure it hands to `Arm`, holding the result in a `RefCell<Option<
InheritableDecoys>>` scoped to its own frame. `Arm`, `run_arm`, `Row`, `Delta`
and `PropertyRemoval` are all untouched; `run_row` is still the only route to a
backend.

Three things decided it, and the first is the one that matters:

1. **Placement is the load-bearing constraint, not per-arm-ness** (`F-9`).
   Provisioning runs capsules, and every capsule fork sweeps this process's
   descriptors close-on-exec — so the state has to open *after* the arm's
   capsule closure returns. Composing onto that closure puts the ordering in the
   structure rather than in a comment above it.
2. **The delta is the right key.** What a control arm needs in order to be able
   to fail belongs to the thing being removed, not to the row carrying it. Both
   arms therefore get a set from the same call, and the probe's non-vacuity
   (`EX-3`) is a consequence rather than a convention.
3. **Scope gives the lifetime, so there is no teardown call to forget.**
   Replaced per capsule; the last set drops when `arm_over` returns.

Rejected: a sixth field on `Arm` — identical ordering, reads more explicitly,
but widens a struct four hand-built test arms construct, so `C10` would be paid
in edits to tests `A4` says establish nothing (this is the migration if a later
phase wants the hook visible). A row-side hook keyed off `RowId` — wrong key per
(2), and no live key today. A `SetUp`/`TearDown` pair — converts a guarantee
scope already gives into a call that can be forgotten or mis-ordered.

### `F-15` — the `EBADF` floor is met by construction, and the floor as written cannot be built

The rule the sheet cites governs a `/proc/self/fd` **walk** — flag mutation over
descriptors the caller does not own. The seam has none and cannot have one:
`BorrowedFd::borrow_raw` is the only route from a raw number to an `AsFd`, it is
`unsafe`, and both budgeted sites are spent (`S7`). Every descriptor the seam
touches is owned, so teardown is `close(2)` and there is no `fcntl` for `EBADF`
to race. A `BADF`-tolerant restore here would be a guard that cannot fire.

The floor that *is* live, and landed in the same commit ahead of `T4`:
`trusted_side_setup` holds `hold_descriptor_window()` across the open, because
inheritability is process-wide state and this is code that opens an inheritable
descriptor and needs it to stay inheritable. It releases before returning —
`fork_within_the_descriptor_window` takes the same lock, so holding it across
the spawn deadlocks the arm at its own fork.

### `F-16` — the one residual race, measured rather than reasoned

Between the window's release and the arm's own fork, another thread's capsule
run can sweep the decoys and leave the control arm nothing to inherit. Closing
it needs a re-entrant window, which production rejects for a better reason than
this one. Narrowed, not closed, and tallied: **5/5** alone under 32 spinners on
32 cores, **8/8** inside the whole conformance suite at `--test-threads=16`
under the same load. `T4` should re-tally when row 10 carries it for real.

### `F-17` — `A7`'s inherited `F-35` set confirmed and named

Three of five whole-suite multi-threaded runs were red, and every red was one of
PHASE-09 `F-35`'s four row-B5 tests — nothing else failed in any run. The seam
cannot be the cause: `trusted_side_setup` returns `None` for every delta a
shipped row carries, so no shipped row's behaviour changed, and the
single-threaded suite is 236/236 green unchanged.

## PHASE-10 `T4` — row 10: `ClosedDescriptorSet` / `DescriptorsClosed`

Shard note (`C9`): the sheet nominates `notes_10-12.md` and forbids minting one
silently. Still no such shard at this sitting, so this record joins `T1`'s and
`T2`'s here under the sheet's own fallback — the third time that instruction has
been overridden, which is itself worth correcting.

### The payload — resolution, not counting; presence, not readability

`ls -1 /proc/self/fd` in a clean capsule reports `0 1 2 3`. The fourth entry is
the enumeration's **own** directory handle, so *no descriptor above 2 appears* is
literally unsatisfiable, and the round-4 row failed before any decoy was ever
inherited (`RV-346` `F-36`). Row 10 therefore resolves rather than counts, and
excludes exactly one descriptor **by identity**: the handle whose target is
`/proc/*/fd`. Everything else above 2 is printed as `FD-RESOLVED-<target>` and
the row says `DESCRIPTOR-INHERITED`.

Three things had to be true at once and only one shape gets all three:

- **One process, not two.** A `readlink` per entry resolves *`readlink`'s* own
  descriptor table, not the capsule's — `F-13`'s trap one level up. A glob over
  `/proc/self/fd/*` lists a directory handle that is already closed by the time
  the shell reads it. `ls -l` enumerates and resolves in the same process, in one
  read.
- **Exactly one own handle, or no answer.** The payload requires `own -eq 1` and
  otherwise prints neither token, so a host whose `/proc` answered strangely
  reads `NoObservation` → `Indeterminate` rather than passing.
- **Named diagnostics.** A failure names what crossed the `exec`; a count would
  only say the table was longer than someone expected.

The shell was validated standalone before any Rust was written: clean → `LIVE` /
`NO-DESCRIPTOR-ABOVE-TWO`; one readable decoy at fd 9 → `LIVE` /
`FD-RESOLVED-…` / `DESCRIPTOR-INHERITED`; unchanged under a pipe.

### Isolation — the decision `F-19` left open, and it was taken

`F-19` closed `F-16` for the seam's own discriminator by measuring it in a child,
and recorded that the residual stays open for **any shipped row carrying the
descriptor delta**. Row 10 is that row. Every executed row-10 claim here is
measured in a child process — `--exact <helper> --ignored --nocapture`, the
parent requiring the marker line positively because a selector matching nothing
exits 0 having run nothing.

The alternatives were all settled upstream and none was reopened: a re-entrant
descriptor window (rejected by `fork_within_the_descriptor_window`'s own
contract), carrying an already-held window into the fork (`S6`, a signature
change), a `#[cfg(test)]` mutex (a known-bad shape that deadlocks row B5), an
idempotent re-check (laundering), and weakening the assertion (the vacuous pass
`EX-3` forbids).

**The residual is the multi-threaded test binary's, not `verify`'s.** `verify`
runs the rows sequentially in one thread, so nothing of its own can sweep this
process between a row's setup and its fork. The window that made `F-18` a 1-in-3
is a property of the runner, and removing the runner removes it.

### The three authority modes

A test-local row (id borrowed, as `descriptor_shaped_row`'s is) that holds when
**none** of the three kinds is present, fails when **all three** are, and prints
nothing in between — so a partial set reads `Indeterminate` and two-of-three can
never be mistaken for *the modes are exercised*, which is the failure a `-ge 1`
threshold would have had. Each mode is recognised by what the descriptor
resolves to: the readable decoy by its path under the decoy directory, the
write-only one by the kernel's ` (deleted)` suffix, the socket end by `socket:`.
Row 10 itself is deliberately blind to kind — it must fail on any of them — so
this is the complementary claim the shipped row cannot make about itself.

### The write-only decoy, both directions

`F-31`'s lesson stated as a test. The decoy is `O_TMPFILE` and write-only: a
capsule that inherits it reads zero bytes through it, exactly as it would
through a closed one, which is how the round-4 row passed a real leak. So the
trusted side reads the file's **size**, never its contents, and reads it in both
directions — `[Some(0), Some(23)]` against arms `Held` / `Failed`. One direction
alone proves nothing: *mutated after the control* would pass for a harness that
wrote on every arm, and *unmodified after the probe* for one that never wrote.

Two things this cost, both recorded as findings. The reading has to happen inside
`arm_over`, because the set is per-arm state dropped there and the file is named
by nothing (`F-26`). And the writing payload is the phase's one destructive
instrument, so its containment landed ahead of the aim (`C13`): the capsule
writes only to a descriptor whose target it has already resolved to an unlinked
file, never to a number chosen blind; there is exactly one redirection; and where
that write lands was already established by
`the_write_only_decoy_is_reachable_by_no_name`.

### The mutants

`F-26`'s (one readable descriptor) and `F-31`'s (one write-only descriptor), both
`Violated` — a probe arm that *fails* convicts regardless of the control, so a
backend leaking on both arms cannot launder the leak into *the removal changed
nothing*.

Their stdout is **observed, not predicted**: row 10's shipped script is run under
`/bin/sh` with exactly one decoy left inheritable and the other two swept by
hand, and what it really prints is what the stub answers with. A discriminating
leg runs the same pipeline with nothing leaked and requires a verdict that is not
`Violated`. Two limits, both findings: no stub can reach `run_row` at all
(`F-24`), and a *real* partial-leak backend is not constructible without a
production visibility change or a third `unsafe` site (`F-25`).

### Tally

Five sequential full runs of `doctrine check gate` — the channel the hazard runs
on, per `F-36`; a green `cargo test` or a CPU spinner would not have seen `F-16`
either. All five `exit=0`, all five `248 passed; 0 failed; 6 ignored`, wall
91.45–91.55s. **5/5.**

That number is only worth stating against its predecessor: the same channel
measured `F-16`'s window at roughly 1 in 3 before the isolation, so five clean
runs is evidence the arms no longer race a sweep rather than a lucky streak.
The suite moved 236 → 248 passing with 6 ignored, the growth being the
`#[ignore]`d child instruments; a log reading `236 passed; 2 ignored` is from a
previous session and counts for nothing here.

## PHASE-10 `T4` residual — the mirror the isolation left

`T4`'s tally above was 5/5 and honest, and the leg still redded on the next
agent's first run. That is the whole lesson of this sitting: **a tally of greens
is not a reproduction, and it cannot stand in for one.** 27 further unaggravated
runs were also green — 21 filtered to the decoy subset at `--test-threads 32`,
then 6 full suites at 91.4–91.6s. The flake was never once observed by waiting
for it.

### The agent, named

The brief's diagnosis was "another test inside its own
`hold_descriptor_window()`". Checked against the source before acting on it, and
it is not possible: `payload_output_leaking` **takes that same lock** and holds
it across its open, its selective CLOEXEC sweep and its `Command::new(SHELL)`
fork. A thread inside the window is the one thing that provably cannot interfere.

The real agents were the callers that took **no** window —
`the_write_only_decoy_is_reachable_by_no_name`,
`a_decoy_set_is_three_descriptors_of_three_kinds`, and
`a_decoy_set_opened_before_provisioning_is_already_closed_by_it`, the last
holding an inheritable set across an entire `provision_capsule`. The generalising
mistake is worth keeping: a lock *taken by the victim* reads very easily as a
lock *contended over*, and the hazard is always the path that never takes it.

### The reproduction, by construction

Two arms, one variable, `--test-threads 4`, victim and aggressor only:

| arm | `the_write_only_decoy_is_reachable_by_no_name` | result |
|---|---|---|
| A | un-windowed (pre-repair shape), set held open 20s | **RED**, first run |
| B | windowed, **same** 20s sleep | green |

Arm A's panic is the reported panic: three `FD-RESOLVED-` entries above the
standard streams, being the fixture's `decoys/descriptor`, a `(deleted)`
`O_TMPFILE`, and a `socket:[…]` — the three kinds `InheritableDecoys` opens, in
one payload. Both arms were reverted before the repair commit; neither is in the
tree.

### Why not the prescribed route

The route offered was `F-19`'s: move the victim's payload into a process of its
own. Measured before adopting it —
`exec 9< /etc/hostname && sh -c 'sh -c "ls -1 /proc/self/fd"'` prints
`0 1 2 3 9`. **An inheritable descriptor survives two `exec` levels.** So a
re-executed test-binary child inherits whatever was inheritable at the instant of
spawn and hands it to its own `/bin/sh`; the aggressor's window shrinks from
"the whole sweep-and-fork" to "the spawn instant" and is not removed. `F-19`
refused that trade in its own words — "it converts a ~13% flake into a smaller
flake" — and it would have been the same laundering here.

The asymmetry worth carrying forward: **a child is a closure when it holds the
aggressor, and only a narrowing when it holds the victim.** Applied to the
aggressor, nothing else shares the process and there is nothing left to inherit.

### The repair

One seam, `decoys_under_the_window(&fixture) -> (MutexGuard<'static, ()>,
InheritableDecoys)`, now the only route a test in this process opens an
inheritable descriptor by. It returns the guard *with* the set, so a caller
cannot bind the descriptors without binding the lifetime that protects them —
structural, not remembered. The victim holds that same guard across open, sweep
and fork, so no other thread can hold an inheritable descriptor while it forks.

Two sites cannot ride it, both commented where they sit:

* `a_decoy_set_opened_after_a_sweep_is_inheritable_again`'s second set is opened
  under a guard already held and `std::sync::Mutex` is not re-entrant — it is
  under the same window by lexical scope, which is the property that matters;
* `a_decoy_set_opened_before_provisioning_is_already_closed_by_it` must keep its
  set across a `provision_capsule` that forks through
  `fork_within_the_descriptor_window`, so windowing it deadlocks. **Moved into a
  process of its own** — the child applied to the aggressor.

`trusted_side_setup` is left alone deliberately and is the standing residual
(`F-30`): it cannot hold the window across the arm's fork either (`F-16`), and it
is safe only because every descriptor-delta row currently runs in a child.

Instrument shape, per the brief's ask: **one** instrument, one prefix
(`PRE-PROVISION=`), one line per decoy carrying `before after` —
`MUTATION_HELPER`'s single-prefix precedent, chosen over a helper per value
because the two readings are one measurement of one set, not two claims. The
child asserts nothing. The parent holds the claim and requires the lines
positively, `assert_eq!` against a vector of exactly `InheritableDecoys::COUNT`
elements, so a selector that matches nothing cannot pass by exiting 0. No
mutant's meaning moved: the shipped row's `shape` and `Observed`, `run_arm` over
both real arms, `under_for(&row.delta)` and `row_verdict` over the pair are
untouched.

### Tally

Six sequential `doctrine check gate` runs, all after the repair commit
(`24155506e`), batched three per call so the harness's 600s clamp could not
truncate the tail. All six `exit=0`, all six `248 passed; 0 failed; 7 ignored`,
wall 135–138s. **6/6.** The ignored count moves 6 → 7 — the new
`PRE-PROVISION` instrument, and nothing else.

State it against what it is worth: `T4`'s 5/5 was the same evidence and did not
hold. What carries the claim here is arm A — the aggressor named, made
deterministic, and then shown to be excluded by the seam rather than out-waited.
The tally corroborates; it does not convict.

## PHASE-10 `T9` — the two unrowed credential observations (`EX-11`, `EX-12`, `VA-5`)

Shard fallback again, for the fourth time: the `PHASE-10` sheet nominates
`notes_10-12.md` and forbids minting a shard silently; no such shard existed at
this sitting either, so this record goes here beside `T1`, `T2` and `T4`'s under
the sheet's own fallback (`F-13`, `F-27`, and now `F-50`).

### The channel, and why it is a third one

`sec-9` `R8`'s two credential mechanisms — the supplementary group list and
`no_new_privs` — are captured, displayed, and reach **no verdict at all**. That
is a stronger claim than table C's, and table C could not carry it: `AuxOutcome`
is `Passed`/`Failed`/`Skipped`, which *is* a verdict. So the observations get a
third channel whose types have nowhere to put an outcome —

```
struct Unrowed { section, name }            // :2673
enum Reading { Read { value, caveat }, Unread(reason) }   // :2696
AdmissionVerdict.observations: Vec<(Unrowed, Reading)>    // :2603
```

— and the property that matters is structural rather than remembered: a verdict
cannot be attached to an observation by accident, because attaching one is a
type change, and a type change is a diff a reader sees.

The reporter (`:4076`–`:4280`) sits **immediately after `tables()`**, which is
`VA-5`'s "where the rows are". It carries the sentence the task exists for: *a
deliberate weakening that is not recorded is indistinguishable from an oversight
to the next reader.*

### The absence, asserted in an order

The test (`:12397`) is an absence probe, the class `T8` had just caught a live
vacuous pass in (item 134). "Nothing attached" and "nothing produced" are the
same empty reading, so the body is ordered and the order is load-bearing: every
positive claim first — both surfaces named, the read-set compared **whole**
against `UNROWED_FIELDS`, the unread side asserted empty, `65534` present, the
group list's **length** equal to the trusted side's, `no_new_privs` reading `1` —
and only then the absence (no shipped payload reads either key; admission is
unmoved in both directions, via `verdict_observing`). The disjointness leg is
guarded twice for non-vacuity: the payload sweep is proved to have found real
payloads (some payload contains `STATUS_UID_KEY`), and the observation payload is
proved to contain both keys, so `contains` is a predicate that can say yes.

The sweep itself is exhaustive **by `match`** over `ArmShape`, not by a filter —
a new variant is a compile error rather than a row the sweep silently walks past.

### The middle claim: why these are unrowable, made checkable

A row needs a delta that moves its reading. These have none — and that is not an
opinion about bubblewrap, it is asserted: the two readings taken either side of
table A's only *granting* control (`--cap-add ALL`) are **identical**. Rowing
either would manufacture two more instances of the `B4` defect (a control that
cannot fire) the round-6 split exists to remove. Recorded in the test's own doc:
if that equality ever reds, the mechanism became rowable and it wants a consult,
not a repair (`S4`).

### `A6` measured, not apologised for

This jail's own parent reads `NoNewPrivs: 1`, so an in-jail run cannot tell
whether bwrap set the bit or the capsule inherited it. The value is right; the
provenance is not establishable from in here. Rather than hard-code an apology,
`no_new_privs_provenance` (`:4166`) is **pure and parameterised on the trusted
side's own reading**: caveat when the host already reads set, none when it reads
unset, and a distinct `PROVENANCE_UNREADABLE` when the host could not be read at
all. So the caveat appears here and disappears on a host that can attribute —
a measurement, not a constant that is wrong on half its hosts.

That wiring is the reason the test asserts `status_field` **positively** against
the real `/proc/self/status` before it asserts anything about caveats. A
silently-failing parse would attach `PROVENANCE_UNREADABLE` on every host
forever: a permanent apology wearing a measurement's clothes, and green.

### `EX-11` written where the row is

`the_identity_is_exactly` (`:3710`) gained a section recording that **setuid
regain is neither probed nor rowed**, and why: the obvious payload cannot be
built by a suite that must not be privileged — a setuid-*root* file requires
root, and a setuid file the test user owns confers the uid the payload already
has. It is not deferred work. `no_new_privs` stood in for it in an earlier draft
and is now retained as an observation and withdrawn as a row.

`tables()`'s doc was stale in the same breath ("sixteen rows … table A's first
eleven") and is corrected to nineteen — all fourteen of table A and all five of
table B — with a paragraph naming what is deliberately *not* there.

### The mutation battery

Six arms, all convicting, each reverted from a pristine copy and `diff -q`
verified identical before the next. Two carry lessons past this task:

- **The group-list under-read initially did not convict.** Dropping
  `${rest:+ $rest}` from the payload truncates the list to its first token —
  which still contains `65534` and passed every assertion written at that point.
  "Unmapped rather than dropped" was the claim and nothing checked it, so the
  count comparison against the trusted side's own `Groups` length was added; the
  arm then convicted (`left: 1 right: 9`). Same family as item 134: the probe
  agreed with an under-read because both produce a *plausible* reading.
- **One arm produced a lint, not a conviction, and was re-run.** Removing the
  unread path made `SURFACE_NOT_NAMED` dead code, so the mutant failed to
  *compile* under `-D dead-code`. That is a lint conviction, not a test
  conviction, and accepting it would have credited the guard with evidence it
  never produced. Re-run in a compiling shape (`Reading::Read` with an empty
  value and the reason moved into the caveat), it convicted at `:12520`.

### Tally

`doctrine check gate` exit **0**; `269 passed; 0 failed; 9 ignored` for
`doctrine-control`, +1 over `T8`'s 268 and matching the one title claimed. Read
by seeking forward to the `doctrine_control-` binary header, never `head -1` /
`tail -1` — that line is neither first nor last in a gate log (`T8`'s parting
tip, friction record `6ed8022b6`). `cargo fmt` and `cargo clippy -p
doctrine-control` clean. Landed as `78ee8f8b6`, path-limited to
`conformance.rs`, +727/−7.


---

## PHASE-10 `T10` — `backend verify` in `main.rs` (`EX-13`)

Landed `42937f4db`, path-limited to `crates/doctrine-control/src/main.rs`,
+450/−13. Gate exit **0**; `274 passed; 0 failed; 9 ignored` for
`doctrine-control`, +5 over `T9`'s 269 and matching the five tests added.

### The verb, and the two things it does not take

`doctrine-control backend verify`. `backend` is a **noun with verbs under it**,
not a verb: `DEC-160` spells the entry point `backend verify`, and the noun is
what leaves room for a second mechanism's verbs without renaming this one. Bare
`verify` stays an unknown verb rather than quietly aliasing the expensive one.

It takes **no options**, and that is `EX-3` rather than an unfinished parser.
The suite synthesizes its own `CapsuleConfig` over its own fixture root, so
admission depends only on the backend's availability and a working shell.
Taking `--repository` would offer the operator's `[capsule]` table as an input
to a verdict about the *backend* — the confusion `verify`'s three-parameter
signature exists to prevent. For the same reason the backend is built without
`with_kill_grace`: that bound comes from the table this verb does not read.

The **wall-clock read lives in the shell**, per `EX-13`: `doctrine::today()`
(exported at `src/lib.rs:62`) is called in `run_backend_verify` and passed in.
`verify` keeps its three parameters and owns no clock.

### The exit code is a number, not a discriminant

`ExitCode` is opaque — no `PartialEq`, no accessor — so a test written against
it can only assert the `Result`'s `Ok`/`Err` and *claim* the mapping to a
status. `EX-13` makes the exit code itself evidence, so `main` now routes
through `exit_status(&Result) -> u8` and returns `ExitCode::from(status)`; the
tests compare the byte. Behaviour is unchanged — `EXIT_ADMITTED = 0`,
`EXIT_REFUSED = 1`, reported identically on both arms.

That mattered: the sheet's warning is that "exits nonzero" passes equally
against a binary that panicked or refused for an unrelated reason. Asserting the
byte **and** the whole rendering is what separates those.

### Rendered through derived `Debug`, deliberately

`render_verdict` renders every row — including the proven ones — plus table C's
claims and the two unrowed observations, one fact per line. Variants go through
`{:?}` rather than a match arm per variant: a hand-written name table over
`Property` would be a **second unchecked enumeration of table A**, which `EX-15`
and `sec-9` `R9` forbid this phase from adding. `Debug` is derived from the enum
itself, so a variant arriving without a name here is impossible rather than
merely unlikely.

The outcome line for a row refusal **names the rows that were not proven**. A
refusal reading only `not-admitted reason=rows` would be true and useless: the
operator's next question is always *which row*.

### `clippy::use_debug` fires on `write!` and not on `format!`

The first gate failed with four `use_debug` errors, all on `write!(rendered,
"…{x:?}")`, while `render_refusal`'s `format!("provision refused: {refusal:?}")`
three functions below has passed every gate since PHASE-06 — and so did my own
`format!("{id:?}={row:?}")` inside the outcome line, in the same commit that
failed. The lint is about debugging remnants reaching an **output handle**, so
it targets the `write!`/`print!` family and leaves `format!` alone.

Repaired by building a `Vec<String>` of lines and `join("\n")`-ing them, which
is better code than the buffer-and-`write!` shape it replaced and drops four
`let _written =` bindings. Recorded as
`mem.fact.rust.clippy-use-debug-write-not-format` and a friction record.

### The mutation battery — twelve arms, and three false convictions first

All twelve applied to a pristine copy, run, and reverted by `cp` with `diff -q`
verifying identical (`C11`). **12/12 convicted by a test.**

Getting there took two passes, and the first pass is the lesson. `M3`, `M8` and
`M9` initially reported CONVICTED — **by the compiler, not by a test**. Each
left a binding unused (`unproven`, `remedy`) and died on `-D warnings` before
any assertion ran. This is item 138's lesson recurring one task later, and the
harness had to be taught to say so: the runner now checks the output for
`error[E` / `could not compile` and reports `<did not compile>` instead of
crediting the guard. Rebuilt in compiling shapes:

- `M3` truncates the unproven list to its **first row** rather than emptying it;
- `M8` **transposes** `missing` and `remedy` rather than dropping one;
- `M9` truncates the remedy to its **first word**.

All three then convicted on assertions, which is the evidence that was wanted.
`M3`/`M7`/`M9` are the `T9` `F-50` part-4 shape — a *degraded* reading defeats
an absence probe as easily as an empty one — and they are in the battery
precisely because that defect has now bitten this slice twice.

The two arms worth keeping in mind:

- **`M5`**, narrowing the filter from `!Proven` to `== Violated`, convicts only
  because the fixture carries all three non-`Proven` verdicts (`Unproven`,
  `Violated`, `Indeterminate`). A fixture with violations alone would have
  passed it.
- **`M4`**, dropping the filter so the outcome line names *every* row, is what
  `the_outcome_line_names_whichever_row_failed_rather_than_a_fixed_one` exists
  for: it renders the same table twice with the failure in different positions,
  so a renderer naming a fixed row and a renderer naming all of them each
  satisfy exactly one of the two assertions.

### Measured: the verb on this tree

`./target/debug/doctrine-control backend verify` → **exit 1**, as the sheet
predicted and for the predicted reason. Nineteen rows `Proven`; the single
refusing row is

```
Property(ProcessTreeTeardown)=Indeterminate { arm: Probe, detail: NoObservation }
```

which is `F-24`/`F-25` exactly — row 7's two arms produce a byte-identical
`Observation` and neither token is printed. Note the reading is
**`Indeterminate`, not `Unproven`**: the sheet's `T13` entry says row 7 "is not
`Proven`", which is what `admission()` tests, and the specific verdict it
carries is the indeterminate one. Four table C claims `Passed`; both unrowed
observations `Read`, `no_new_privs` carrying its `EVD-014` `A6` provenance
caveat. Wall clock ~95 s, essentially all of it the suite.

`T13` is unblocked by nothing here: this verb reports row 7's state, it does not
resolve it.

### What these tests do not cover, said rather than implied

The clock read is **not** asserted. `run_backend_verify` calls
`doctrine::today()` inline, and replacing that with a frozen constant would pass
all five tests — the fixtures supply their own date. What holds `EX-13`'s
clock half up is structural, not tested here: `verify` takes `today` as a
parameter and has no clock to reach for, which `conformance.rs` owns. Injecting
a clock into `run_backend_verify` to close this would add a seam whose only
consumer is its own test, and the sheet's instruction is to keep `verify`'s
signature as it is.

Likewise the five tests never run the suite. That is deliberate: a dispatch
assertion that provisioned ~50 capsules would double `T12`'s budget for no
evidence the conformance suite does not already produce. Every dispatch case
refuses before reaching a backend; the end-to-end reading above was taken by
hand, once, and is recorded rather than automated.
