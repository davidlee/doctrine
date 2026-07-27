# A negative grep result is untrustworthy without a positive control

`grep` in this harness is a ugrep wrapper with `-I`, which silently skips binary files — so a file containing one stray control byte matches nothing, and even `grep -c` prints nothing rather than 0.

## The wrapper

`grep` here is **not** GNU grep. It is a shell function wrapping `ugrep` with
`-I` (skip binary files), `--ignore-files` and `--hidden`:

```
$ type grep
grep is a shell function from ~/.claude/shell-snapshots/snapshot-zsh-*.sh
   ... exec -a ugrep "$_cc_bin" -G --ignore-files --hidden -I --exclude-dir=.git ...
```

`-I` makes ugrep **skip a whole file silently** if it contains any byte it deems
binary. One stray NUL is enough. The file then matches nothing, and the skip is
never reported.

## The failure mode

Verifying a freshly-written `design.md` (SL-232), three separate sweeps returned
no matches and were read as "clean". All three were **false negatives**: the file
contained one NUL byte, so ugrep classified it binary.

The tell is subtle: `grep -c <pattern> <file>` printed **nothing at all** rather
than `0`. A count of zero is a result; no output is a skip. Diagnosis cost ~6
tool calls, most spent chasing a phantom working-directory bug, because the file
was plainly present (`wc -l` worked).

## The rule

**Never accept a negative grep as evidence without a positive control on the
same file in the same invocation.**

```bash
command grep -c "<something-certainly-present>" "$F"   # must print a non-zero count
command grep -n "<the-thing-hoped-absent>" "$F" || echo "genuinely absent"
```

`command grep` bypasses the wrapper; `command grep -a` forces text mode. To check
the underlying cause directly:

```bash
python3 -c "from pathlib import Path; b=Path('$F').read_bytes(); \
  print(len([c for c in b if c<0x20 and c not in (9,10)]), 'control bytes')"
```

## Why this generalises

The instruction being discharged was *"sweep this, then verify it did"*. The
verification ran, reported success, and was worthless. **An instruction to verify
needs a companion habit of falsifying the verifier** — the discipline routinely
applied to probes and measurements, turned on one's own checks.

Corollary: control characters enter authored files easily and invisibly. While
writing prose *about* NUL bytes, a literal NUL was emitted three times where the
text `backslash-u-0000` was intended. `doctrine validate` passed clean with a NUL
inside a tracked design document — nothing in the toolchain flags it.
