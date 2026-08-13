# IDE-051: Oubliette: enforce worker-forbidden-writes via bwrap read-only binds, not a post-import belt

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

`SL-254` PHASE-06 retired `worker_commit` (the in-session arm's only commit
path). `worker_commit` was also the *only* production reader of
`DispatchConfig::worker_forbidden_writes` — the `[[ForbiddenWrites]]` matcher
compiled from the `worker-forbidden-writes` TOML key (`src/dispatch_config.rs`).
The retained path (`worktree::import::classify_import`) never read that config;
it only ever enforced the two hard-coded code floors, `.doctrine/**` and
`.claude/**`. So `worker-forbidden-writes`'s *configurable* tail —
`.agents/**`, `install/agents/**`, `flake.nix` (the entries
`install/doctrine.toml.example` calls "the highest security leverage in the
repo") — now has no enforcement point at all. `ForbiddenWrites`/`is_forbidden`
are kept but marked `expect(dead_code)` naming the gap; the example file's
comment was rewritten in PHASE-06 to say so honestly rather than silently
claiming enforcement that no longer exists. No config in this repo actually
sets the key today, so there is no live exposure — but the feature as shipped
no longer does what it documents for any project that does set it.

Owner decision (2026-08-13, mid-`SL-254` Batch B escalation): accept the gap
for `SL-254`'s own scope — reopening it here would reverse PHASE-06/VT-1 ("the
import belt is UNTOUCHED") and INV-2 with a same-slice `DEC`, which is a worse
trade than carrying it forward — and resolve it in `SL-255` instead, which is
already building the clone-provisioning arm's new commit/import machinery and
so can add a reader without contradicting a criterion `SL-254` already closed.

## The refinement — enforce at the kernel, not a post-hoc belt

Don't just re-home `ForbiddenWrites::is_forbidden` as a Rust-side check that
runs *after* a worker has already written and is now asking to commit — that
is strictly weaker than what the rest of this slice's confinement model does.
`PHASE-02`'s own verification standard (`VT-3`, design `VT-2`/`VT-3`) is "a
worker under the prefix cannot write outside its directory, refused by the
KERNEL not a hook." `worker-forbidden-writes` should get the same treatment:

Extend the confinement prefix builder (`scripts/spawn-confined.sh`'s inline
bwrap array, and the Rust `sandbox_exec_argv`/`bwrap_argv` builders in
`src/worktree/jail.rs`) to accept an optional list of extra paths to bind
**read-only** inside the worker's mount namespace — on top of the existing
`--bind "$D" "$D"` rw grant for the worker's own worktree. A path named in
`worker-forbidden-writes` becomes a `--ro-bind <path> <path>` (or, for paths
inside `$D` itself, a nested nested-ro-bind carving a hole in the rw grant) in
the spawn-time argv, so a write attempt fails at the syscall level regardless
of which tool or code path the worker uses — no import-time belt to bypass, no
config a worker-side process could route around by writing through a different
verb.

This composes with `SL-255`'s clone-provisioning work rather than competing
with it: the confinement prefix is spawn-time machinery either way, and
`SL-255` is where the worker's write surface is being redefined regardless
(writable clone `.git` vs today's linked-worktree model).

## Open questions for whoever picks this up

- Precedence with the existing rw grant: does a `worker-forbidden-writes` entry
  nested inside `$D` need to carve a `--ro-bind` *after* the `--bind "$D" "$D"`
  in argv order (bwrap is order-sensitive — later binds win), or should such an
  entry be refused as a config error (a worker forbidding writes to its own
  worktree is likely a mistake, not intent)?
- Whether to keep the Rust-side `ForbiddenWrites`/`is_forbidden` matcher as a
  defense-in-depth belt-and-suspenders check alongside the kernel enforcement,
  or retire it once the kernel leg lands (STD-001 — don't carry two
  enforcement paths for one rule without a reason).
- macOS/Seatbelt parity — the mac profile is generated (`jail_prefix.rs`'s
  `.sb` materialisation), not an inline array; the equivalent there is an
  additional `(deny file-write* (subpath "..."))` clause per extra path.
