#!/usr/bin/env python3
"""Live-corpus populations the six still-open design items turn on.

`census.py` measures the claim-surface shape (symlink-bearing, non-matching,
FAL-4 ancestors). This measures the *decision* populations — the numbers that
decide OQ-6's field, E13's fate, objective 7's blast radius, and R-E's liveness.

FALSIFIERS, registered before the probe runs. Each is a way this script could
report a comfortable number without having measured anything:

  FAL-P1  the non-contributing bucket split must be reproducible from the entry
          text alone (prefix match on a fixed root list). If a bucket needs a
          judgement call per entry, the "declared boundary" is not authored-
          decidable and OQ-6's shape is wrong.       -> print every unbucketed entry
  FAL-P2  the ISS-257 population must come from running the REAL guard
          (`merge-base --is-ancestor`), not from asserting that old shas are
          non-ancestors.                             -> compare against Some/None counts
  FAL-P3  the glob-only-attested count must be gated on the SAME predicate the
          code uses (`scope.paths.is_empty()`), not on "declares a glob".
  FAL-P4  R-E's live population must be measured by `ls-files -v` over the whole
          index, not over the memory corpus alone — R-E affects the source leg.
  FAL-P5  a memory's ATTESTED state is `[git].verified_sha` non-empty. If that
          field is absent corpus-wide the denominators are vacuous -> assert > 0.

All figures are stamped with the HEAD they were taken at (RV-313 F-1: a design-
time absolute failed to reproduce at execution purely through corpus growth).
"""

import collections
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path("/workspace/doctrine")

# SIZING ONLY — this is NOT the mechanism. DEC-020 refuted deriving the boundary
# from local repository state three times (F-21 existence, F-25 checkout, F-31
# ref-set), and objective 3's boundary is AUTHORED, not derived. What follows
# estimates how many of the non-contributing entries an author would plausibly
# declare, by asking git one decidable question per entry — is this path ignored
# by this checkout's ignore rules? An ignored path is one git is configured never
# to observe; a merely-absent path is not distinguishable from a moved one, which
# is exactly why the boundary must be declared rather than measured.


def git(*args):
    p = subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True)
    return p.returncode, p.stdout


head = git("rev-parse", "--short", "HEAD")[1].decode().strip()
print(f"HEAD: {head}\n")

# ---- load every tracked memory --------------------------------------------
_, out = git("ls-files", "-z", "--", ".doctrine/memory/items/*/memory.toml")
tomls = [p.decode() for p in out.split(b"\0") if p]

mems = []
for rel in tomls:
    try:
        data = tomllib.loads((ROOT / rel).read_text(encoding="utf-8"))
    except Exception as e:  # noqa: BLE001
        print(f"!! unparsed {rel}: {e}", file=sys.stderr)
        continue
    scope = data.get("scope") or {}
    mems.append(
        {
            "uid": Path(rel).parent.name,
            "paths": list(scope.get("paths") or []),
            "globs": list(scope.get("globs") or []),
            "sha": ((data.get("git") or {}).get("verified_sha") or "").strip(),
        }
    )

attested = [m for m in mems if m["sha"]]
print(f"memories: {len(mems)}   attested (verified_sha non-empty): {len(attested)}")
assert attested, "FAL-P5 TRIPPED: no attested memories — denominators vacuous"
print()

# ---- OQ-2 / QUE-175: which staleness mode does each attested memory get? ---
# FAL-P3: gate on the code's own predicate (retrieve.rs:371, memory.rs Check 2).
path_scoped = [m for m in attested if m["paths"]]
glob_only = [m for m in attested if not m["paths"] and m["globs"]]
unscoped = [m for m in attested if not m["paths"] and not m["globs"]]
print("== OQ-2 / QUE-175: staleness mode of attested memories ==")
print(f"  path-scoped + attested (commit mode)     {len(path_scoped)}")
print(f"  glob-only  + attested (TIME mode: bug)   {len(glob_only)}")
print(f"  unscoped   + attested (time mode)        {len(unscoped)}")
print(f"  scoped-and-attested total                {len(path_scoped) + len(glob_only)}")
print()

# ---- ISS-257: how many anchored memories hit the ancestry guard? ----------
# FAL-P2: run the real guard, not a proxy.
print("== ISS-257: the ancestry guard's live population ==")
ancestor = nonancestor = badobj = 0
for m in attested:
    rc, _ = git("merge-base", "--is-ancestor", m["sha"], "HEAD")
    if rc == 0:
        ancestor += 1
    elif rc == 1:
        nonancestor += 1
    else:
        badobj += 1
total = ancestor + nonancestor + badobj
print(f"  verified_sha IS an ancestor of HEAD      {ancestor}")
print(f"  NOT an ancestor (exit 1)                 {nonancestor}")
print(f"  object absent / error (exit >=2)         {badobj}")
print(f"  -> commits_touching returns None for     {nonancestor + badobj} of {total}")
if total:
    print(f"  -> reach: {100 * ancestor / total:.1f}%")
print()

# Check 2's population is narrower: it is additionally gated on scope.paths.
c2 = [m for m in path_scoped]
c2_none = 0
for m in c2:
    rc, _ = git("merge-base", "--is-ancestor", m["sha"], "HEAD")
    if rc != 0:
        c2_none += 1
print(f"  of which Check 2 (scope.paths-gated) reaches: {len(c2)}, silent on {c2_none}")
print(f"  Check 4 (body drift, all attested) silent on: {nonancestor + badobj}")
print()

# ---- OQ-6: bucket the non-contributing entries ----------------------------
MAGIC = {"paths": ":(literal)", "globs": ":(glob)"}
cache = {}


def matches(field, text):
    spec = MAGIC[field] + text
    if spec not in cache:
        rc, out = git("ls-files", "-z", "--", spec)
        cache[spec] = (rc, [c for c in out.split(b"\0") if c])
    return cache[spec]


noncontrib = []
for m in mems:
    for field in ("paths", "globs"):
        for text in m[field]:
            if not text.strip():
                continue
            probe = text
            if text.startswith(str(ROOT) + "/"):
                probe = text[len(str(ROOT)) + 1 :]
            rc, rows = matches(field, probe)
            if rc == 0 and not rows:
                noncontrib.append((m["uid"], field, text))

def ignored(text):
    """Is this entry's root ignored by this checkout's ignore rules? Walks up
    from the entry to the first component, so `.claude/skills/x/**` is decided by
    `.claude` being ignored. `check-ignore` needs a concrete path, so wildcards
    are truncated at the first component containing one."""
    parts = []
    for comp in text.split("/"):
        if any(c in comp for c in "*?["):
            break
        if comp not in ("", "."):
            parts.append(comp)
    while parts:
        rc, _ = git("check-ignore", "-q", "/".join(parts))
        if rc == 0:
            return "/".join(parts)
        parts.pop()
    return None


buckets = collections.Counter()
by_bucket = collections.defaultdict(list)
for uid, field, text in noncontrib:
    root = ignored(text)
    key = (
        f"ignored by this checkout (root: {root})"
        if root
        else "not ignored — moved, deleted, or never existed"
    )
    bucket = "expected-unobservable" if root else "ordinary"
    buckets[bucket] += 1
    by_bucket[bucket].append((field, text, root))

print("== OQ-6: the 'declared boundary' population (SIZING, not the mechanism) ==")
print(f"  non-contributing entries total           {len(noncontrib)}")
print(f"    root ignored by this checkout          {buckets['expected-unobservable']}")
print(f"    not ignored (moved/deleted/never)      {buckets['ordinary']}")
print(f"  memories carrying >=1 non-contributing entry: "
      f"{len({u for u, _, _ in noncontrib})}")
print()
print("  FAL-P1 — every entry printed with its verdict, so the split is auditable:")
for bucket in ("expected-unobservable", "ordinary"):
    for field, text, root in sorted(set(by_bucket[bucket])):
        mark = f"IGNORED via {root}" if root else "not ignored"
        print(f"    {field:5s} {text:42s} {mark}")
print()

# ---- E13: the outside/escaping population ---------------------------------
print("== E13: entries whose emitted form would leave the repository ==")
outside = 0
for m in mems:
    for field in ("paths", "globs"):
        for text in m[field]:
            t = text
            if t.startswith("/") and not t.startswith(str(ROOT) + "/"):
                outside += 1
                print(f"    absolute-outside: {field} {text}")
                continue
            depth = 0
            esc = False
            for comp in t.split("/"):
                if comp in ("", "."):
                    continue
                if comp == "..":
                    depth -= 1
                    if depth < 0:
                        esc = True
                        break
                else:
                    depth += 1
            if esc:
                outside += 1
                print(f"    escaping: {field} {text}")
print(f"  total: {outside}")
print()

# ---- F-38: argv-unrepresentable / report-splitting values -----------------
print("== F-38: control characters in scope entries ==")
nul = ctrl = 0
for m in mems:
    for field in ("paths", "globs"):
        for text in m[field]:
            if "\0" in text:
                nul += 1
            elif any(ord(c) < 0x20 for c in text):
                ctrl += 1
                print(f"    control char: {field} {text!r}")
print(f"  NUL-bearing: {nul}   other control chars: {ctrl}")
print()

# ---- R-E: index bits that suppress the measurement ------------------------
# FAL-P4: over the WHOLE index, not the memory corpus.
print("== R-E: skip-worktree / assume-unchanged rows (whole index) ==")
_, out = git("ls-files", "-v")
flagged = [
    line for line in out.decode("utf-8", "replace").splitlines() if line and line[0] != "H"
]
print(f"  non-'H' index rows: {len(flagged)}")
for line in flagged[:20]:
    print(f"    {line}")
print()
print(f"(all figures at HEAD {head})")
