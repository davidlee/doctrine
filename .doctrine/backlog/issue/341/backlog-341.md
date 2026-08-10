# ISS-341: Conformance fixture binds whole host top-level roots

`system_readable_roots` declares the **top-level ancestor** of every
canonicalized `PATH` entry as a readable root, and readable roots are
identity-mapped. On the owner's NixOS host that measured as:

```
/nix   <- /nix/store/…-devshell-dir/bin        (wanted: loader and libraries)
/run   <- /run/wrappers/wrappers.SpYzxokvnp    (setuid wrappers; no executable the capsule uses)
/var   <- /var/lib/flatpak/exports/bin         (Flatpak export farm; likewise)
/bin   <- the literal `SHELL`                  (wanted: the entry point to exec)
```

So every conformance capsule on that host has the host's **entire `/run` and
`/var`** bound read-only — `/run/user/1000` and its keyring, gnupg and pipewire
sockets, `/var/lib` in full — because two unrelated directories happen to sit on
`PATH`.

## Why this is worse than it looks

**No shipped row can convict it.** Row 3 reads planted decoys under the fixture
root, so it is silent about the host's real state. Row 4 derives its permitted-`/`
set from `$PATH`, so `/run` and `/var` are *permitted by exactly the fact that
put them there*. The two rows whose job is bounded visibility are structurally
blind to this, and both read `Proven` on that host.

The suite is therefore proving `BoundedFilesystemVisibility` inside a capsule
whose filesystem visibility is not bounded in the way a reader of that verdict
would assume. The verdict is not false — it says the undeclared decoy is
unreachable and `/` holds only permitted entries, and both hold — but what it is
worth off-jail is much less than what it is worth in the jail, and nothing in the
transcript says so.

**In-jail this is invisible.** The jail's `PATH` roots are `/nix` and `/bin`
only, so the derivation looks exact there. This is the same shape as the
`/bin/sh` defect `ISS-339` run 1 found: a derivation that is accidentally correct
in the one environment it has ever run in.

## Why the current rule exists

The top-level ancestor is deliberate and the reason is good:

> a dynamically linked executable needs its loader and libraries, which on a
> store-based host live under sibling directories of the same top level; a
> measurement on this host showed `git` runs under `--ro-bind /nix/store` and
> cannot under its own `bin` directory alone.

That argues for `/nix` being broad. It does not argue for binding a top level
that contributes no executable at all. The rule generalised one host's real
constraint into an unconditional one.

## Direction

Bind a root only where some `PATH` entry beneath it actually bears an executable
— the same predicate row 2 applies from inside. Then the fixture and the payload
agree by construction instead of by coincidence, and `ISS-340` dissolves: `/run`
and `/var` stop being roots, their entries leave the inner `PATH`, and row 2's
coverage set collapses to `/nix`.

Invariant 7's existing exclusions (fixture root, `$HOME`, cwd) are a *deny* list
over a permissive derivation. What this wants is the derivation to stop being
permissive — a root has to earn its place.

## Loose end worth noticing while here

`/run/wrappers/bin` canonicalizes to `/run/wrappers/wrappers.SpYzxokvnp`, whose
final component is **regenerated per boot**. A readable root named by a path that
changes across boots is a poor thing to bind and a worse thing to record in
evidence. Narrowing per the direction above removes it, so this needs no separate
treatment — but if the direction is not taken, it does.

## References

`ISS-340` (the red row this dissolves) · `ISS-339` (the off-jail runs that
surfaced both) · `SL-248` `PHASE-10` · `sec-9` residual 3 · invariant 7 / `VA-2`
