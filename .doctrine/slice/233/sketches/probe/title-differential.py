#!/usr/bin/env python3
"""RV-323 rev 5 — the generated differential behind EX-13(b) and VA-7/VT-6.

This is the PROTOTYPE the sketch's § *The oracle this rule owes its shape to*
reports. PHASE-06 reimplements the property in Rust; this file exists so the
measurement is reproducible and so the next reviewer can attack the instrument
rather than take the numbers on trust.

THE PROPERTY
    for every body B:  derive(B) == derive(format(B))

`derive` is EX-13(b)'s decision procedure. `format` is a Markdown formatter.
The corpus is a PRODUCT, not a list — that is the whole point, after three
hand-written enumerations leaked in a row.

WHAT IT FOUND (39,019 bodies, prettier@3.9.6)
    rev 4 as written                    4,680 divergences
    + leading ws survives the test      4,224   closes `## ###` (RV-323 F-3)
    + refuse bare-`#`-run titles        4,224   closes NOTHING — the guard was wrong
    + strip to exhaustion               1,664   closes the whole hash cascade
    + collapse internal whitespace        832   closes `# a  b`; NOT ADOPTED

ALL FIVE ROWS ARE IMPLEMENTED BELOW AND PRINTED BY A RUN. Rev 5 published the
table with only rows 1, 4 and 5 implemented, which left the load-bearing number
— the guard closing ZERO — unreproducible by the very reviewer invited to
attack the instrument. Rows 2 and 3 were added afterwards; every published
figure reproduces unchanged and no adopted rule moved.
The 832 residual is inline-markup rewriting (`*em*` -> `_em_`), which no
extraction rule can reach — see the sketch for why the property is withdrawn
rather than chased.

USAGE
    npm install prettier@3.9.6 && python3 title-differential.py

POSITIVE CONTROL is mandatory and enforced: rev 4's rule is known-defective, so
a run where it shows ZERO divergences means the harness is broken, not that the
rule is sound. A negative result from an instrument that cannot fire is worth
nothing — the lesson F-6 taught this slice about greps, applied to oracles.
"""
import itertools
import json
import re
import subprocess
import sys

BLANK = " \t"
FORMATTER = ["node", "-e", """
const p = require('prettier');
let raw = ''; process.stdin.on('data', d => raw += d);
process.stdin.on('end', async () => {
  const out = [];
  for (const d of JSON.parse(raw)) {
    try { out.push(await p.format(d, { parser: 'markdown' })); }
    catch { out.push(d); }
  }
  process.stdout.write(JSON.stringify(out));
});
"""]


# ── EX-13(b): the decision procedure ─────────────────────────────────────────

def is_blank(line):
  return all(c in BLANK for c in line)


def atx_split(line):
  """The recogniser: 0-3 spaces, 1-6 '#', then EOL or space or tab.
  Returns the content region, or None if `line` is not an ATX heading line."""
  i = 0
  while i < 3 and i < len(line) and line[i] == " ":
    i += 1
  if i < len(line) and line[i] == " ":
    return None                                   # a 4th leading space
  j = i
  while j < len(line) and line[j] == "#":
    j += 1
  if not 1 <= j - i <= 6:
    return None
  if j < len(line) and line[j] not in BLANK:
    return None
  return line[j:]


def extract_rev4(region):
  """As rev 4 wrote it. RETAINED AS THE POSITIVE CONTROL — do not delete."""
  r = region.lstrip(BLANK).rstrip(BLANK)          # <-- eats the delimiter first
  stripped = r.rstrip("#")
  if len(stripped) < len(r) and stripped and stripped[-1] in BLANK:
    r = stripped.rstrip(BLANK)
  return r


def extract_leading_ws_survives(region):
  """Rev 4 + the F-3 repair alone: the leading whitespace survives until after
  the closing-sequence test, but the strip still runs ONCE. Closes `## ###`
  (4,680 -> 4,224) and nothing else — the cascade is still here."""
  r = region.rstrip(BLANK)
  stripped = r.rstrip("#")
  if len(stripped) < len(r) and (stripped == "" or stripped[-1] in BLANK):
    r = stripped.rstrip(BLANK)
  return r.strip(BLANK)


def extract_plus_bare_run_guard(region):
  """The fix that looked obviously right for F-3's family, and closed NOTHING.

  WHERE THE GUARD SITS IS THE WHOLE RESULT, so it is pinned here rather than
  left to a reader's reconstruction. The guard tests the heading's CONTENT
  REGION — which is where anyone repairing `## ###` would naturally write it —
  and by then the row above has already refused that family, so it fires only
  where refusal was happening anyway: 4,224 -> 4,224.

  Two neighbouring placements do NOT reproduce that, and mistaking one for this
  reads as 'the published number is wrong' rather than 'the guard was wrong':
      guard on the DERIVED TITLE, '#'-run only        4,224 -> 2,748
      guard on the derived title, '#'-and-whitespace  4,224 -> 1,664
  Both close real divergences, so either would have shipped as a working repair
  while leaving the cascade — which is the point the adopted rule rests on."""
  raw = region.strip(BLANK)
  if raw and all(c == "#" for c in raw):
    return ""
  return extract_leading_ws_survives(region)


def extract_rev5(region):
  """The adopted rule. Two differences from rev 4, each load-bearing:
  the leading whitespace survives until after the closing-sequence test, and
  the strip runs to exhaustion rather than once."""
  r = region.rstrip(BLANK)
  while True:
    stripped = r.rstrip("#")
    if len(stripped) == len(r):
      break                                       # no trailing '#' run
    if stripped != "" and stripped[-1] not in BLANK:
      break                                       # run not preceded by whitespace
    r = stripped.rstrip(BLANK)
    if r == "":
      break
  return r.strip(BLANK)


def extract_rev5_plus_collapse(region):
  """NOT ADOPTED. Kept because it closes a real family (`# a  b`) and the
  sketch owes an explanation of why a working repair was declined: its only
  justification was one formatter's behaviour, and that justification is
  withdrawn. Retained so a reviewer can re-measure the tradeoff."""
  return re.sub(r"[ \t]+", " ", extract_rev5(region))


def derive(body, extract):
  f = next((ln for ln in body.split("\n") if not is_blank(ln)), None)
  if f is None:
    return "!SectionBodyEmpty"
  region = atx_split(f)
  if region is None:
    return "!SectionBodyHeadingMissing"
  title = extract(region)
  return "!SectionTitleEmpty" if title == "" else title


# ── the corpus: a product, not a list ────────────────────────────────────────

def corpus():
  indents = ["", " ", "  ", "   ", "    "]
  runs = ["#", "##", "###", "######", "#######"]
  delims = ["", " ", "  ", "\t", " \t "]
  contents = ["", "T", "Title", "Title#", "#", "##", "###", "# #", "# # #",
              "#hashtag", "a b", "Title\\", "\\#", "\\##", "T ##x", "-", "1.",
              "Ünïcødé", "日本語", "a  b", "*em*", "`code`", "[l](u)", "T\\#"]
  trailers = ["", " ", "\t", "#", " #", " ##", "  ###", "\t##", " \\#", " \\##",
              "## ", " ## ", "  "]
  for parts in itertools.product(indents, runs, delims, contents, trailers):
    yield "".join(parts)
  # non-heading first lines, so arms 1 and 2 are exercised too
  yield from ["", "Title\n===", "Title\n---", "```\n## In a fence\n```",
              "~~~\n# tilde fence\n~~~", "    ## indented", "<div>\n\n## After",
              "\n\n## Late", "> ## quoted", "- ## in a list", "1. ## in ol",
              "\n \t \n## After blanks", "## A\n## B", "#", "##", "######",
              "\t## tab indented", "## Trailing hash run ####", "## \\## esc"]


def fmt(docs):
  p = subprocess.run(FORMATTER, input=json.dumps(docs),
                     capture_output=True, text=True)
  if p.returncode != 0:
    sys.exit("formatter failed — is prettier installed here?\n" + p.stderr[:400])
  return json.loads(p.stdout)


def main():
  bodies = [b + "\n\nbody text\n" for b in corpus()]
  formatted = fmt(bodies)

  rules = [("rev 4 (positive control)", extract_rev4),
           ("+ leading ws survives the test", extract_leading_ws_survives),
           ("+ refuse bare-`#`-run titles", extract_plus_bare_run_guard),
           ("rev 5 (adopted)", extract_rev5),
           ("rev 5 + collapse (declined)", extract_rev5_plus_collapse)]

  results = {}
  print(f"corpus: {len(bodies)} generated bodies\n")
  for name, extract in rules:
    div = [(b, f) for b, f in zip(bodies, formatted)
           if derive(b, extract) != derive(f, extract)]
    results[name] = div
    print(f"{name}: {len(div)} divergences")
    for b, f in div[:3]:
      print(f"      {b.split(chr(10))[0]!r} -> {f.split(chr(10))[0]!r} | "
            f"{derive(b, extract)!r} => {derive(f, extract)!r}")

  if not results["rev 4 (positive control)"]:
    sys.exit("\nHARNESS BROKEN: rev 4 is known-defective and MUST diverge. "
             "A clean run here means the oracle cannot fire.")
  print("\npositive control fired — the oracle can detect a bad rule.")

  # the formatter's own fixed point, which rev 4 asserted was reached in one pass
  cur, history = formatted, []
  for i in range(2, 8):
    nxt = fmt(cur)
    moved = sum(1 for a, b in zip(cur, nxt) if a != b)
    history.append(f"pass {i} moved {moved}")
    cur = nxt
    if not moved:
      break
  print("formatter idempotence:", ", ".join(history))


if __name__ == "__main__":
  main()
