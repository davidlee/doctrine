Measured 2026-08-10, in this repo's bubblewrap development jail and on the
owner's NixOS host. The two have **structurally different `$PATH` shapes**, and
the jail's shape cannot produce the host's failures.

**In the jail**, every `$PATH` entry is already an individual store path —
`/nix/store/…-bubblewrap-0.11.2/bin`, `…-socat-1.8.1.3/bin`,
`…-claude-code-…/bin`, `…-codex-…/bin`, and a dozen more. There are **no symlink
farms at all**, and `/lib` and `/lib64` do not exist.

**On the host**, `$PATH` carries `/run/current-system/sw/bin` and
`/etc/profiles/per-user/…/bin` — symlink farms that resolve *into* the store —
plus `/run/wrappers/…` and `/var/lib/flatpak/exports/bin`.

## Why this is a footgun and not a curiosity

`bwrap` dereferences a bind's **source**, so binding a farm directory gives the
namespace links whose targets are outside the bind. That failure is
**structurally unexercisable in the jail**, because the jail has no farms to
bind.

This is the fourth instance of one pattern in this subsystem, and each was found
only by running somewhere else:

1. `/bin/sh` resolved two ways — the jail supplied `/bin` twice over by
   coincidence (`ISS-339` run 1).
2. row 4's permitted-`/` set, derived from `$PATH` (`ISS-339` run 2).
3. the readable root set itself, `/nix` bound whole (`ISS-341`).
4. this one: farm entries, invisible here by construction.

The general finding is **a derivation that is accidentally correct in the single
environment it has ever run in**.

## What to do about it

A test for `$PATH`-derived behaviour must **not depend on the developer's
`$PATH`**. Inject the entries through the `HostFacts` seam (`FixtureHost` tables
`env_var` and `resolve`) and construct the farm case explicitly — a directory of
symlinks whose targets lie outside every declared bind. A test that reads the
real environment passes here and says nothing about the host.

Related: [[mem.pattern.sandbox.readable-roots-are-top-level-ancestors]] (carries
a correction for instance 3), [[mem.fact.capsule.bwrap-ro-bind-dereferences-source]].
