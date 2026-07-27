#!/usr/bin/env python3
"""Three narrow counts over the live corpus, to settle paragraphs the design
currently asserts without measurement:

  1. symlink-rooted / symlink-bearing scope entries  -> does the index-first rule
     actually add resolved targets for real entries? (the design claims "no glob
     declaration in this corpus is currently symlink-rooted")
  2. outside-shaped / escaping entries               -> the number E13's fate turns on
  3. path-through-a-symlinked-directory entries      -> the FAL-4 class

Matching is delegated to git itself. No pathspec semantics are re-implemented.
"""
import subprocess, sys, tomllib, os, collections
from pathlib import Path

ROOT = Path("/workspace/doctrine")

def git(*args, check=False):
    p = subprocess.run(["git", "-C", str(ROOT), *args],
                       capture_output=True)
    return p.returncode, p.stdout

def ls_files(spec):
    """Return (exit, [(mode, path)]) for a pathspec. 128 == abort."""
    rc, out = git("ls-files", "-s", "-z", "--", spec)
    rows = []
    for chunk in out.split(b"\0"):
        if not chunk:
            continue
        meta, _, path = chunk.partition(b"\t")
        rows.append((meta.split()[0].decode(), path.decode("utf-8", "surrogateescape")))
    return rc, rows

# ---- gather scope entries from every tracked memory.toml -------------------
rc, out = git("ls-files", "-z", "--", ".doctrine/memory/items/*/memory.toml")
tomls = [p.decode() for p in out.split(b"\0") if p]

entries = []          # (uid, field, text)
for rel in tomls:
    try:
        data = tomllib.loads((ROOT / rel).read_text(encoding="utf-8"))
    except Exception as e:
        print(f"!! unparsed {rel}: {e}", file=sys.stderr); continue
    scope = data.get("scope") or {}
    uid = Path(rel).parent.name
    for field in ("paths", "globs"):
        for text in (scope.get(field) or []):
            entries.append((uid, field, text))

print(f"memories (tracked memory.toml): {len(tomls)}")
print(f"scope entries total:            {len(entries)}   "
      f"(paths={sum(1 for e in entries if e[1]=='paths')}, "
      f"globs={sum(1 for e in entries if e[1]=='globs')})")
print()

# ---- lexical pre-classification -------------------------------------------
def lexical(text):
    if text is None or text.strip() == "":
        return "empty"
    if "\0" in text:
        return "nul"
    if any(ord(c) < 0x20 for c in text):
        return "control"
    if text.startswith("/"):
        return "absolute-inside" if text.startswith(str(ROOT) + "/") else "absolute-outside"
    # does it escape the root lexically?
    depth = 0
    for comp in text.split("/"):
        if comp in ("", "."):
            continue
        if comp == "..":
            depth -= 1
            if depth < 0:
                return "escaping"
        else:
            depth += 1
    return "inside"

lex = collections.Counter()
for _, _, t in entries:
    lex[lexical(t)] += 1

print("== COUNT 2: lexical classification (E13's domain) ==")
for k, v in lex.most_common():
    print(f"  {k:20s} {v}")
print()

# ---- git-measured expansion ------------------------------------------------
MAGIC = {"paths": ":(literal)", "globs": ":(glob)"}

stat = collections.Counter()
symlink_bearing = []     # entries whose direct match set contains a symlink
ancestor_recover = []    # entries that match nothing but have a symlink ancestor
aborts = []

cache = {}
def cached_ls(spec):
    if spec not in cache:
        cache[spec] = ls_files(spec)
    return cache[spec]

for uid, field, text in entries:
    cls = lexical(text)
    if cls in ("empty", "nul", "control"):
        stat[f"pre-rejected:{cls}"] += 1
        continue
    probe = text
    if cls == "absolute-inside":
        probe = text[len(str(ROOT)) + 1:]
    elif cls in ("absolute-outside", "escaping"):
        stat["pre-rejected:outside"] += 1
        continue
    spec = MAGIC[field] + probe
    rc, rows = cached_ls(spec)
    if rc == 128:
        stat["ABORT(128)"] += 1; aborts.append((uid, field, text)); continue
    if not rows:
        # FAL-4: ancestor walk for a tracked symlink
        anc = probe
        found = None
        while "/" in anc:
            anc = anc.rsplit("/", 1)[0]
            arc, arows = cached_ls(":(literal)" + anc)
            if arc == 0 and arows and arows[0][0] == "120000" and arows[0][1] == anc:
                found = anc; break
        if found:
            stat["non-matching: symlinked-ancestor (FAL-4)"] += 1
            ancestor_recover.append((uid, field, text, found))
        else:
            stat["non-matching (E7 report)"] += 1
        continue
    modes = {m for m, _ in rows}
    if "120000" in modes:
        stat["matches, SYMLINK in match set"] += 1
        symlink_bearing.append((uid, field, text, [p for m, p in rows if m == "120000"][:3]))
    else:
        stat["matches, no symlink"] += 1

print("== COUNTS 1 & 3: git-measured expansion of every entry ==")
for k, v in stat.most_common():
    print(f"  {k:44s} {v}")
print()

print("== COUNT 1 detail: entries whose match set CONTAINS a symlink ==")
print("   (these are the entries where index-first adds a resolved target and")
print("    today's probe is blind — i.e. live F-20/F-37-class exposure)")
if not symlink_bearing:
    print("   (none)")
for uid, field, text, links in symlink_bearing[:40]:
    print(f"   {field:5s} {text}")
    print(f"         -> symlink match: {links}")
print(f"   total: {len(symlink_bearing)}")
print()

print("== COUNT 3 detail: non-matching entries with a tracked symlink ancestor ==")
if not ancestor_recover:
    print("   (none)")
for uid, field, text, anc in ancestor_recover[:40]:
    print(f"   {field:5s} {text}   (ancestor symlink: {anc})")
print(f"   total: {len(ancestor_recover)}")
print()

if aborts:
    print("== entries that ABORT git even after lexical guard ==")
    for uid, field, text in aborts[:20]:
        print(f"   {field:5s} {text!r}")
    print(f"   total: {len(aborts)}")
