# ISS-343: doctrine-control is Linux-only, in default-members, and fake-darwin cannot run in the jail

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**Release-blocking.** A forced release tag (2026-08-10) failed CI at compile time
on both Apple targets. `crates/doctrine-control` does not build on macOS at all:

```
error[E0433]: cannot find `linux` in `os`     conformance.rs:57   std::os::linux::net::SocketAddrExt
error[E0432]: unresolved import                conformance.rs:67   rustix::fs::FsWord
error[E0599]: no associated constant TMPFILE   conformance.rs:1410 OFlags::TMPFILE
error[E0599]: no associated fn                 conformance.rs:1775 SocketAddr::from_abstract_name
```

Four unconditional Linux-only APIs. There is **no `cfg(target_os = ...)` gate
anywhere in the crate** — verified by grep across `crates/doctrine-control/src`.

## Why CI builds it at all

The release job runs a bare `cargo build --release --target <t>`
(`.github/workflows/release.yml:91`). Bare `cargo build` selects
`default-members`, and `Cargo.toml:120-123` names both `.` **and**
`crates/doctrine-control`. So every release target builds a Linux-only crate,
including the two Apple ones in the matrix.

`publish = false` on the crate handles publishing. It does not touch building,
and building is what the release matrix does.

The `default-members` entry is deliberate and correct for its own purpose:
`SL-248` `sec-8` added it so the conformance suite could not ship green by never
being compiled. The interaction with the release matrix was not in view.

## Why nothing caught it

`just fake-darwin` (`justfile:118-119`) is exactly the guard for this — and it
**cannot run in the development jail**. Attempted 2026-08-10: no Apple target is
installed, so the run dies compiling third-party dependencies (`memchr`,
`once_cell`, `typenum`, …) long before reaching any doctrine crate. A developer
running it sees a wall of unrelated dependency errors, not the four real ones.

This is `mem.fact.tooling.x-bit-is-not-runnability`'s second rule: a tool that
cannot be *invoked* is a defect of the harness, never a verdict about the
subject. The guard is inoperable where development happens, so it was
inoperable when the Linux-only crate joined the checked set, and the first
signal was a red release tag.

## What a fix has to decide, not just do

1. **Does `doctrine-control` build on macOS, or is it excluded from non-Linux
   targets?** Excluding is cheap (`-p doctrine` in the release job, or a
   `cfg(target_os = "linux")` gate on the crate's contents). Building means
   porting or stubbing four Linux-specific mechanisms, and `SPEC-030` names
   Linux/bubblewrap as *the measured initial backend* — a macOS backend is
   explicitly not evidenced. Exclusion looks right; it should still be decided
   rather than defaulted.
2. **How does the guard become operable?** A `fake-darwin` that cannot run in
   the jail will fail this way again. Either it acquires the target in the jail,
   or it moves to CI where the target exists, or it announces that it could not
   run instead of failing as though it had.

## Not to be confused with

`ISS-342` — the *other* red gate, where `just gate` runs `cargo test --workspace`
and nine live-`bwrap` conformance tests fail off-jail. That one is about test
execution on a host that cannot host a capsule. This one is a compile failure on
a platform the crate was never written for. Different failures, different fixes,
same crate.

## Provenance

Surfaced by a forced release tag while `SL-252`'s design run was in its exploring
stage; captured rather than fixed, per `DEC-184`'s scope discipline.
