## The decision

The fixture declares `closure-roots` and a `closure-resolver`, and the resolver is a **short POSIX `sh` script the fixture writes into its own temp root** and marks executable (`write_file` / `make_executable` already exist — the decoy executable uses both). The script sets `LD_TRACE_LOADED_OBJECTS=1`, execs the queried binary, and emits absolute paths one per line.

No host tool is acquired. `LD_TRACE_LOADED_OBJECTS` is dynamic-loader behaviour, available wherever a dynamically linked binary runs, so `POL-002` facet (3) has nothing to declare.

## Why not `ldd`, and why the first refutation was too strong

`ldd` is absent from the development jail (measured 2026-08-10), which I first offered as a refutation. It is not one: that is a fact about our jail, and a flake line dissolves it. The owner's read — proportional — was right.

What actually decided it is that **a wrapper script is required either way**. `closure_members` (`bubblewrap.rs:837`) demands every returned line be an absolute path, and `expand_closure_root` (`:910`) refuses any returned path that does not exist. `ldd` emits `libc.so.6 => /nix/… (0x…)` and a pathless `linux-vdso.so.1`, so `closure-resolver = ["/path/to/ldd"]` cannot be declared directly. Once the wrapper exists, `ldd` versus the loader variable is one line inside it — `ldd` is itself a shell script that sets that variable and execs.

So the question was never which mechanism, but whether to acquire a declared host capability when the loader answers for free. It does, so we do not.

## `ldd` still earns its flake line, as a dev dependency

Adding `ldd` to the jail is worth doing so a test can **differentially check the resolver's output against glibc's own reference implementation** on a host where both exist. That is a stronger use of the dependency than making it load-bearing, and it keeps it out of the shipped path.

## Why route through `closure-roots` at all

The alternative was cheaper: have the fixture compute the same set in Rust and declare every path as a plain `readable-root` — same readable set, no script file, no resolver admission.

It was refused because routing through `closure-roots` builds the fixture's capsules **through the code path production uses** — `resolved_closure_root` → `expand_closure_root` → `closure_members`. That path today has *zero* live exercise: it is driven only by `FixtureQuery` doubles in unit tests. `REQ-459` criterion 2 was ruled partial by `REV-051` partly for want of production evidence, so giving the closure path its first real exercise is worth more than the script costs.

## What the resolver script must do, from the code that consumes it

1. **Echo `$1` itself.** `expand_closure_root` binds only what the resolver *returns*; it never adds the realised root. Omit this and the queried binary is unbound.
2. **Drop pathless lines.** `linux-vdso.so.1` has no path and does not exist as a file; returning it is `ProfileRefusal::ResolverPathAbsent`.
3. **Exit 0.** `closure_members` checks status *before* parsing, deliberately, so a resolver that failed and printed plausible paths refuses as a failed resolver.
4. **Stay under `RESOLVER_OUTPUT_LIMIT`** (1 MiB). Measured output for the whole toolset is 25 lines.

## Measured, on this host, 2026-08-10

| | store paths |
|---|---|
| today's fixture, `/nix` bound whole (`ISS-341`, off-jail) | 691 |
| toolset closure — 6 real binaries, 25 files | 16 |

The sixteen names in the payload inventory are only **six files**: `cat head env cut ls tr sleep true` are symlinks into one coreutils multicall binary, and `echo printf pwd kill` are shell builtins that need files at all only because row 2 stats `true` as a file.

## Hazards carried into the plan

- **The mechanism is *exec the tool and trust the loader to intercept before `main`*.** True for dynamically linked ELF; false for a static binary or a script, where the tool would actually run. Check exit 0 and output shape.
- **`git` may want more than libraries** — `libexec/git-core`, templates. Table B row 2 uses only `hash-object`, `update-ref`, `cat-file` and `show-ref`, which are builtins of the single binary, but that wants verifying in the first phase rather than assuming.


---

## Withdrawn from SL-252, 2026-08-11

**`SL-252` does not implement this decision.** `DEC-188` takes `DEC-185`'s S3
fallback — the readable set is declared and wide — so no `closure-roots` are
declared, no resolver is admitted, and the mechanism this record chose has
nothing to compute. It is withdrawn from the slice rather than deferred inside
it.

That is the cheap reason. An external critical review supplied better ones, and
they are recorded here because they must not be lost if the mechanism is ever
revived.

### What the review refuted

1. **"Emits absolute paths one per line" understates the work.** Loader-trace
   output is `name => /path (0x…)`, an address-bearing diagnostic format, while
   `closure_members` accepts only a bare absolute path occupying the whole
   trimmed line. The decision therefore owns a *parser* from glibc loader
   diagnostics into Doctrine's closure protocol, and that parser is load-bearing
   rather than implementation trivia.

2. **"Drop pathless lines" is unsafe, and this is the serious one.** It conflates
   a legitimate pathless runtime object (`linux-vdso.so.1`) with a dependency the
   loader *could not resolve* (`libfoo.so => not found`), which is also pathless.
   Dropping both turns "runtime dependency missing" into "closure computed
   successfully", and Doctrine's downstream existence checks cannot recover the
   information because the missing entry was discarded before they ran. Any
   revival needs four classes — filesystem dependency, known virtual object,
   unresolved dependency, unrecognised form — with the last two failing closed.

3. **"`ldd` is the same mechanism" is not established.** This record asserts that
   `ldd` merely sets the variable and execs. Modern glibc `ldd` deliberately
   avoids executing the inspected file directly, invoking it through a verified
   dynamic linker instead, precisely because executing an arbitrary file is
   unsafe. `ldd` is absent from this jail, so the claim was never verified here;
   it must not be treated as settled. See the correction appended to
   `mem.fact.linker.ld-trace-loaded-objects-is-the-closure`.

4. **`POL-002` facet (3) does have something to declare.** "No host tool is
   acquired" is true and does not imply "no host capability is depended on". The
   mechanism needs a particular loader honouring a particular environment
   variable and emitting a particular format — glibc behaviour, not a generic
   Linux dynamic-linking contract. A `sh` resolver script additionally depends on
   its shebang interpreter existing on the trusted host. Absence should produce
   an explicit unavailable outcome, not an execution whose result is interpreted.

5. **The safety check runs after the risk.** Checking exit status and output
   shape cannot be the boundary against a static binary or script, because by
   then the target has run. A direct-exec design must establish the target is a
   supported dynamic ELF *before* executing it — at which point the wrapper is
   acquiring ELF inspection logic of its own, which is what rejecting `ldd` was
   meant to avoid.

### What survives, and should be carried forward

- **The objective.** `resolved_closure_root` → `expand_closure_root` →
  `closure_members` has zero live exercise; it is driven only by `FixtureQuery`
  doubles. Giving it a real one is worth doing, and `REV-051` is part of why.
  Carried as `IMP-425`.
- **The root-echo requirement** (§1 of *What the resolver script must do*) is a
  true statement about production as it stands today, independently confirmed by
  two reviewers. It stops being true only if closure semantics are separately
  revised.
- **Status dominates plausible stdout** — `closure_members` checks exit status
  before parsing, deliberately. Preserve that ordering in any successor.
- **Differential testing against `ldd`** remains worthwhile if a glibc-specific
  resolver is ever revived — but comparing canonical dependency *sets*, not
  textual output. Note the cost argument this cuts both ways on: if correctness
  needs `ldd` in the dev environment plus a parser plus a differential test plus
  target classification plus capability preflight, avoiding `ldd` in the shipped
  path has bought little.
- **No silent broadening.** If closure resolution cannot be established, the
  readable set must not widen to compensate.

### Why the reopen is not being worked here

The review's cost argument is `RSK-231`'s argument at one level down: a mechanism
presented as a thin wrapper over free loader behaviour in fact owns a parser,
capability assumptions, unsupported target classes, failure semantics and an
execution boundary. That is the same failure mode the capsule programme is under
review for, so reviving it belongs to that architectural revision, not to a slice.
