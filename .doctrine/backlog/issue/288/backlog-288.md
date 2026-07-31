# ISS-288: Runbook verifier resolves its binary on PATH

A runbook step's `verify` argv is executed as given (SL-233 PHASE-16, sketch §4:
"an arbitrary executable, zero on success"), so the shipped `explore.research`
step's `doctrine` resolves through `PATH` like any other program. That contract
is right for a *project-supplied* verifier. It is arguably wrong for a check
Doctrine ships and invokes from inside itself, and it bites concretely here:

- In this repo's bubblewrap jail `~/.cargo/bin/doctrine` is READONLY and stale.
  A developer running `./target/debug/doctrine design apply` spawns the stale
  binary, which has no `verify` subcommand, so `explore.research` can never be
  discharged as `verified` — it fails with clap's "unrecognized subcommand"
  rather than with anything about research.
- More generally: any invocation by path (nix store, a worktree's
  `target/debug`, `DOCTRINE_BIN`) spawns a *different* doctrine than the one
  running, or none at all.

The e2e suite dodges this by leading `PATH` with the binary under test
(`tests/design_fixture/mod.rs::path_leading_with`), which is correct for a test
and does nothing for a user.

Options, none taken yet:

1. Resolve `argv[0] == "doctrine"` to `std::env::current_exe()`. Cheap; makes
   one name magic, which cuts against "arbitrary executable".
2. Add a `{doctrine}` placeholder to the closed `PLACEHOLDERS` vocabulary and
   use it in shipped runbooks. Explicit, no magic, no special case — but it
   changes `exploring.toml`'s step digest, so it wants doing before anything
   persists discharges in anger.
3. Leave it and document that a shipped verifier requires a current `doctrine`
   on `PATH`.

Option 2 looks right. Raised while implementing PHASE-16 T6; the phase does not
depend on the answer, and the shipped runbook is not yet load-bearing for any
real run.
