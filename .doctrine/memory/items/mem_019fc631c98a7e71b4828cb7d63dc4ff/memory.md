Running any git command with an untrusted repository as its context puts that
repo's `.git/config` on the effective config cascade. **Git protects some
execution-vector keys from repo-level config and not others**, so "git defends
against hostile repo config" is not a claim you can make in general — you must
check the specific key.

Measured directly (git 2.54.0, SL-241 PHASE-06 F-P06-11):

| key | honoured from repo-level config? |
|---|---|
| `uploadpack.packObjectsHook` | **NO** — protected config (system/global) only |
| `core.fsmonitor` | **YES** — fires on index refresh |

`core.fsmonitor` fires on `git status` and does **not** fire on
`git rev-parse --verify <ref>`. So a harness that touches an untrusted repo only
with `rev-parse` and `fetch` is safe **by virtue of which commands it runs**, not
because git defended it. Add a `status`, `diff`, `add` or `checkout` against that
repo and the same config becomes arbitrary code execution.

## Two consequences worth carrying

1. **A local-path `git fetch` DOES spawn `upload-pack` in the source repo.**
   Verified under `GIT_TRACE=1`: `git fetch -- ../src` runs
   `git-upload-pack '../src'`, which then spawns `pack-objects` — which is
   exactly what `uploadpack.packObjectsHook` would replace. The `--local`
   hardlink optimisation is a `clone` behaviour, not a `fetch` one. Reading a
   hostile repo "just to get an OID" is not free of it.

2. **Count the trusted-side commands, not the transferred bytes.** "Config and
   hooks are repo-local, never objects" is sound about what *travels*, and says
   nothing about the trusted side *going to* the hostile config. When comparing
   ingestion designs, the number of git invocations against the untrusted repo is
   a first-class part of the attack surface.

## How to check a key yourself

Don't reason about it — two repos and a hook script settle it in seconds. Set the
key in the source repo, run the command you actually run, and see whether the
payload fires. A payload that does not fire proves the vector is closed *for that
command*, which is weaker than proving git refused the key.

Related: [[mem_019fa18161f47651af7687d8dccbbc67]] (a negative result needs a
positive control).
