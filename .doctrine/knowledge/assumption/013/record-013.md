# ASM-013: Pi and codex follow symlinked skill dirs under .agents/skills

SL-284 replaced an npx-written **copy** of the skill catalog with a relative
**symlink** into the embed (`codex`/`pi`/`universal` → `.doctrine/skills/<id>`,
linked at `.agents/skills/<id>`). Claude has always worked this way; the two
non-Claude harnesses have not.

Nothing in the repository can prove the assumption. It is a claim about the
behaviour of two external programs, and every unit test in the slice asserts what
doctrine *wrote* — that the link exists, resolves, and points where intended —
never that pi or codex *read* it. If the assumption is false, the slice's whole
premise fails silently: install succeeds, the links are correct, and the harnesses
load nothing.

## What would falsify it

- Either harness does not follow symlinks for skill discovery (it enumerates real
  directories only, or refuses links).
- Pi emits a duplicate-skill warning because a second copy is reachable.
- Pi fails to load the generated extensions from the runtime tier
  (`.doctrine/state/pi/extensions`), which would also falsify the
  `.pi/settings.json` registration mechanism independently of skills.

## Why the cleanup must come first

This checkout still holds the pre-migration state: 40 real npx skill copies under
`.agents/skills/`, 33 under `.pi/skills/`, a tracked `skills-lock.json`, and the
old `.pi/extensions/doctrine/`. Validating against that state is worthless — a
stale real directory at `.agents/skills/<id>` shadows the doctrine link (the
`Shadows` notice exists precisely for this), and the duplicate is exactly what a
"no duplicate warning" check would trip on. The one-off cleanup (design sec-7
VH-1) has to land before the observation means anything.

## Note on `.pi/skills`

The settled input behind this slice (IMP-406 `OQ-1`) is that pi warns when skills
are present in *both* `.pi/skills/` and `.agents/skills/`, which is why doctrine
deliberately never links into `.pi/skills`. That is a separate, already-decided
fact — not part of what this assumption asserts.


## Validation (2026-10-20, SL-284 close, VH-2)

User-run live sessions after the VH-1 cleanup (stale npx copies under
`.agents/skills` and `.pi/_skills`, and `.pi/extensions/doctrine/`, removed;
`doctrine install` relinked `.agents/skills/<id>` into `.doctrine/skills/<id>`).
User report: "live pi works fine" and "codex is checked" — both harnesses load
doctrine skills through the symlinks; pi loads its extensions from
`.doctrine/state/pi/extensions`.
