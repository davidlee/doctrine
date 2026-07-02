# IMP-230: De-duplicate pi-spawn-confined.sh NUL-array reader from its e2e test copy

The confinement PREFIX reader — the portable `while IFS= read -r -d '' tok ||
[ -n "$tok" ]; do PREFIX+=("$tok"); done < "$OUT"` loop — currently lives in
**two** places:

- `scripts/pi-spawn-confined.sh` (Darwin arm) — the SOURCE OF TRUTH.
- `tests/e2e_worktree_jail_prefix.rs` `READER_SNIPPET` — a hand-kept copy that
  the e2e reader tests drive against a real `jail-prefix --out` file.

They must stay byte-faithful by discipline, which is the parallel-implementation
smell the project forbids (CLAUDE.md, STD-001 single-source ethos). Surfaced in
SL-185 PHASE-04 when the design's `mapfile -d ''` had to be replaced with the
portable loop (macOS bash 3.2 has no `mapfile`) — the fix had to be applied to
both copies.

**Improvement:** have the e2e test extract the reader from the script itself
(e.g. source the script's reader function, or slice the snippet out of the file
at test time) so there is a single source of truth. Requires factoring the
reader into an addressable unit in the script first.

Originates from SL-185 PHASE-04.
