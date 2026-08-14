# REQ-304: Isolation integrity: tier exclusion is guaranteed by construction rather than a trusted check, and the sole-writer guard fails closed on ambiguity (a marker-absent linked worktree is refused, not trusted).

## Statement

> **AMENDED — FALSIFIED IN PART (SL-254, 2026-08-14).** The title line's first
> clause (tier exclusion by construction) still holds and is now stronger. Its
> second clause is falsified: with the disk marker gone (`DEC-207`) there is no
> "marker-absent linked worktree" class, so the guard neither faces that
> ambiguity nor refuses on it. The title is retained unchanged; the statement
> below is what must hold.

Isolation integrity rests on two properties.

1. **Tier exclusion is guaranteed by construction, not by a trusted check** —
   unchanged in substance, and now doubled. Provisioning is the sole copier and
   admits only allowlisted paths, so the coordination/runtime tier is ABSENT from
   a fork rather than present-and-forbidden (REQ-189, REQ-296). SL-254 adds a
   second, independent construction: the worker runs under a kernel jail
   (`--ro-bind / /` on Linux; an SBPL `(deny file-write*)` floor on macOS) whose
   writable set is the fork's worktree **plus the harness config dir**
   (`~/.claude` / `~/.pi` — a deliberate carve-out under `DEC-210`, corrected
   SL-254), so a coordination/runtime path that were present could not be written.
2. **The sole-writer guard is unambiguous rather than fail-closed-on-ambiguity.**
   Worker mode is `DOCTRINE_WORKER == "1"` and nothing else (REQ-192) — a value
   compare, so a bare-set non-`1` value is not worker mode. The variable
   is set by the same confinement argv that establishes the write floor, so it
   cannot be absent where confinement is present. There is no third state to be
   ambiguous about: a linked worktree without the variable is an ORDINARY
   worktree (a solo isolation fork, say) and is correctly not refused; a tree
   with it is a worker and is refused whatever its topology.

**Note the deliberate posture change.** Under SL-056 a marker-absent linked
worktree was refused *because the signal might have been missed* — fail-closed
was insurance against a stamp that never happened. SL-254 removes the failure
mode instead of insuring against it: identity is created by the act of
confinement, so "confined but unidentified" is unreachable. Fail-closed
behaviour survives where a failure is still possible — at SPAWN, where a missing
`bwrap` is a named refusal on Linux with no unconfined fallback (on macOS the
refusal is still fail-closed but unnamed: there is no `sandbox-exec` presence
probe).

## Rationale

An integrity property that depends on a trusted check is only as good as the
check's coverage; one that depends on construction cannot be bypassed by an
uncovered path. The same reasoning that moved tier exclusion from a check to a
construction moved worker identity from a stamped artefact (which can be missing,
stale, or about the wrong tree) to a process property created by confinement
itself — which is why the ambiguity the old clause guarded against no longer
exists to guard against.
