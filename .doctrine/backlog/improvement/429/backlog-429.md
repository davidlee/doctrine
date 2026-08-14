# IMP-429: Give the confinement prefix a home, and macOS parity with it

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced during `SL-254`'s design inquiry (2026-08-13), against the
`sufficiency-accepted` judgement, and deliberately kept out of that slice.

## The two halves

**Placement is the real question.** The bwrap confinement prefix currently lives
at the script tier — `scripts/pi-spawn-confined.sh:113-131`, an eight-token
`PREFIX` array — with a macOS `sandbox-exec` sibling beside it. `SL-254`
generalises it past `pi` and stops there, on `DEC-209`'s grounds: no
`doctrine-control` dependency, no hardening ported. That leaves unanswered where
it should *live*:

- a script, as today;
- part of the library / install surface, seeded like the other shipped assets;
- a Rust wrapper in the binary;
- or something else.

The tension is `POL-002` on one side (harness specifics stay out of the engine
core) and single-sourcing on the other (`STD-001` — the same confinement facts
are currently expressed twice, once per host OS, in shell). `jail.rs` already
owns `bwrap_core_argv` / `bwrap_argv` / `validate_policy` / `select_jailer`, and
`crates/doctrine-control` owns a much fuller `confinement_argv`
(`backend/bubblewrap.rs:1110`) that `SL-254` declined to depend on — so the
placement question is really *which of three existing homes wins*, not where to
build a fourth.

**macOS parity is wanted, and is awkward to verify.** `SL-254` objective 1
asserts the `sandbox-exec` sibling is kept at parity, and `DEC-206` re-homes
`write_seatbelt_profile` among the four jail primitives it moves to `jail.rs`
before any deletion — so the seatbelt path is inside the surface. But no
decision covers whether the collapsed claude arm actually *reaches* it, and
testing it needs a different host, which is painful to arrange mid-slice.

## Why it is not `SL-254`'s

`SL-254`'s posture is to avoid building new things: it collapses two arms onto
the incumbent one and deletes the apparatus that existed only for the arm being
removed. Choosing a new home for the prefix is construction, and the macOS leg
cannot be verified on the slice's own host. Both are better done deliberately
than folded into a collapse.

## What a mac-equipped session should actually test

Folded in at `SL-254`'s reconcile from `RV-356` `F-2`, so the next session with
mac access tests hypotheses rather than re-running the arm blind. `SL-254`'s
`PHASE-09` recorded `VA-2` as **UNVERIFIED on hardware** per `DEC-212` — no mac
in the capsule — but confirmed the core claim at the code level, which is as far
as a Linux host can take it.

**The confirmed code-level chain.** `worktree jail-prefix`'s `--network` flag
defaults to **deny** (`worktree/mod.rs:230`); `spawn-confined.sh`'s Darwin arm
calls `jail-prefix` **without** `--network`, so `network == false`; that appends
`(deny network*)` to the Seatbelt profile (`jail.rs:238`, `:488`). Therefore a
macOS `claude -p` worker is network-denied and cannot reach the API at all. The
Linux inline `bwrap` array carries no `--unshare-net`, so it is network-open —
which is why the Linux live fire worked, and why run 5 *needed* that network.

- **`H1`** — a confined `claude -p` on Darwin fails to reach the API, with a
  *network* error rather than the `--verbose` / login errors seen on Linux. The
  fix is a **policy decision** (does the Darwin arm pass `--network`?), not a typo.
- **`H2`** — Darwin has **no `sandbox-exec` presence probe**. `RV-355` `F-6`'s
  named-refusal fix (`command -v bwrap` → `REASON_NO_BWRAP`) landed on Linux only,
  so a missing macOS backend still fails *unnamed*, and only after a fork has been
  minted. Same class, never carried across.
- **`H3`** — Darwin points `TMPDIR` at `<wt>/.tmp`, i.e. **inside the worktree**
  whose working-tree delta the orchestrator imports, so harness scratch lands in
  every imported delta. Benign in this repo only because `.gitignore:14` (`*.tmp`)
  happens to match; a client project without that line leaks. The Linux fix
  deliberately used `/tmp` (tmpfs) to avoid the class outright.
- **`H4`** — the framing itself. §5.1/§5.2.1's *"the two profiles differ in
  exactly two tokens"* is **retired, not repaired**: falsified in both directions,
  by `DOCTRINE_WORKER` (Darwin lacked it, `RV-355` `F-2`) and `TMPDIR` (Linux
  lacked it, `PHASE-09`). A mac census must **enumerate every asymmetry**, not
  spot-check against a claim of parity. `SL-254`'s `design.md` §5.2.1 now carries
  that enumeration on two axes — harness and platform — and both lists are floors.

`H4` also restates what `SL-254`'s `PHASE-09` `VA-2` criterion should have asked.
The criterion itself is immutable and `PHASE-09` is complete, so the corrected
intent lives here instead.

## Related

- `RV-356` `F-2` — the audit finding that folded `H1`–`H4` in.
- `SL-254` — the collapse that surfaces this; `DEC-209` declines the
  `doctrine-control` dependency and defers hardening to `IMP-428`.
- `IMP-428` — harden the worker confinement prefix. Adjacent but distinct: that
  one is about *what the prefix contains*, this one about *where it lives* and
  whether the macOS sibling holds.
- `POL-002`, `STD-001` — the two rules in tension over the answer.
