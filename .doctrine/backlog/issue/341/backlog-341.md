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

## The `/nix` root is the serious one, not the junk roots

`--ro-bind` is read-only, **not `noexec`**. A capsule with `/nix` bound can
execute anything in the store. Measured in the development jail:

```
store paths visible: 691
RAN git:   git version 2.54.0
RAN curl:  curl 8.20.0 … OpenSSL/3.6.2 … libssh2/1.11.1
RAN gcc:   gcc (GCC) 15.3.0
```

A compiler, a version-control client, and a TLS-capable HTTP client, in a capsule
whose defining property is a *bounded input set*. `/run` and `/var` are noise
next to this; `/nix` is the finding.

## What makes it a defect and not a tradeoff: production already refuses it

This is not the platform's posture. `readable_set` in `backend/bubblewrap.rs`
states the opposite rule and enforces it:

> There is **no host-shaped default and no fallback**. Only the two declared
> lists ever become readable inputs (`EX-8`), which is why no arrangement of
> configuration — including the branches taken when a list or the resolver is
> absent — can make a host-wide artefact store readable whole.

Production binds `readable-roots` as declared plus each `closure-roots` entry
expanded through an operator-declared `closure-resolver` — on a store host, a
binary's *runtime closure*, which for a shell is a handful of paths rather than
the whole store. The vocabulary for doing this correctly already exists and is
already wired.

`system_readable_roots` is exactly the host-shaped default `EX-8` rules out. The
conformance fixture therefore builds its capsules under the rule the thing it is
testing exists to refuse — and those are the capsules in which `BoundedInputSet`
is demonstrated. The row's own subject is compromised by the fixture that hosts
it: "executes only what is bound" is a much weaker claim when *bound* means the
entire system toolchain.

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

**Make the fixture declare what production would make an operator declare.** The
target is `closure-roots = [<the shell>]` with a `closure-resolver`, so the
fixture binds the shell's runtime closure rather than the top level it happens to
live under. That is the shape `EX-8` already contemplates, it needs no new
config vocabulary, and it makes the fixture's capsules an honest instance of the
thing under test instead of a permissive special case.

A weaker interim step, if the above proves to have a tail: bind a root only where
some `PATH` entry beneath it actually bears an executable — the same predicate
row 2 applies from inside. That fixes `/run` and `/var` and dissolves `ISS-340`
(their entries leave the inner `PATH`, and row 2's coverage set collapses to
`/nix`, which is covered), but it leaves `/nix` bound whole and so leaves the
serious half of this item open. Do not mistake it for the fix.

Invariant 7's existing exclusions (fixture root, `$HOME`, cwd) are a *deny* list
over a permissive derivation. What this wants is the derivation to stop being
permissive — a root has to earn its place.

## The tension to resolve, which is why this is not a small edit

A `closure-resolver` is host-shaped by nature — on this host it is a Nix
invocation. Production escapes that by making it *operator-declared* config, so
the platform never names Nix (`POL-002`). A fixture has no operator to declare
it, and must not grow a Nix dependency of its own. So the fixture needs a way to
be closure-correct on a store host and still run on one without a resolver, and
the fallback must not quietly reinstate the whole-top-level bind this item is
about.

That is a design question, not a patch. It is also the question `SPEC-030`'s
readable-input contract implicitly answers for operators and leaves open for the
suite.

## Loose end worth noticing while here

`/run/wrappers/bin` canonicalizes to `/run/wrappers/wrappers.SpYzxokvnp`, whose
final component is **regenerated per boot**. A readable root named by a path that
changes across boots is a poor thing to bind and a worse thing to record in
evidence. Narrowing per the direction above removes it, so this needs no separate
treatment — but if the direction is not taken, it does.

## References

`ISS-340` (the red row this dissolves) · `ISS-339` (the off-jail runs that
surfaced both) · `SL-248` `PHASE-10` · `sec-9` residual 3 · invariant 7 / `VA-2`
