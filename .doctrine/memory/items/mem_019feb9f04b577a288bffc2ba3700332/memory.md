Setting `LD_TRACE_LOADED_OBJECTS=1` and exec'ing a dynamically linked ELF makes
the **dynamic loader** print every transitively resolved library and the loader
itself, then exit 0 without running the program. Measured on NixOS in the
development jail, 2026-08-10:

```
$ LD_TRACE_LOADED_OBJECTS=1 "$(command -v git)"
        linux-vdso.so.1 (0x0000703a12370000)
        libpcre2-8.so.0 => /nix/store/…-pcre2-10.46/lib/libpcre2-8.so.0 (0x…)
        libz-ng.so.2   => /nix/store/…-zlib-ng-2.3.3/lib/libz-ng.so.2 (0x…)
        libgcc_s.so.1  => /nix/store/…-gcc-15.3.0-lib/lib/libgcc_s.so.1 (0x…)
        libc.so.6      => /nix/store/…-glibc-2.42-61/lib/libc.so.6 (0x…)
        /nix/store/…-glibc-2.42-61/lib/ld-linux-x86-64.so.2 (0x…)
```

**This is what `ldd` does.** `ldd` is a shell script that sets this variable and
execs. So "use `ldd`" and "use the loader" are the same mechanism, and the
difference between them is one line inside whatever wrapper parses the output —
never a reason to acquire `ldd` as a dependency.

## Why this matters on a store-managed host

Nix store paths bear **no lexical relation to their dependencies**: `bash` and
its `glibc` are unrelated directories. So there is no cheap route from a binary
to its libc, and reaching the loader without binding the store whole requires an
ELF read, a store query, or this. There is no fourth option.

Measured for one small toolset (`sh cat head env true cut ls tr sleep socat
setsid git`, which is six real files — the coreutils names are symlinks into one
multicall binary):

| | store paths |
|---|---|
| binding `/nix` whole | 691 |
| the toolset's closure | 16 |

## The hazards, both real

1. **The mechanism is *exec the tool and trust the loader to intercept before
   `main`*.** True for dynamically linked ELF. **False for a static binary or a
   script**, where the program actually runs. Check exit 0 and that the output
   has the expected shape; do not assume interception.
2. **`linux-vdso.so.1` has no path** and is not a file. Any consumer that
   requires returned paths to exist must drop pathless lines.

## Not available everywhere

`ldd` was **absent from this project's development jail** while every store path
it would have reported was present — a reminder that "the tool is missing" is a
fact about an environment, not about a mechanism. The loader variable needed
nothing installed.

Related: [[mem.fact.tooling.x-bit-is-not-runnability]] — a binary whose loader is
absent is `-x` and exits 127, which is the failure this closure exists to
prevent.
