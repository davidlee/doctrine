# ISS-336: Spike artefacts do not record their environment

The spike output artefacts do not say which environment produced them, and at
least two environments are mixed in the corpus.

## Evidence

`spike-credentials-output.txt` reads `uid_map 0 0 4294967295`, userns
`user:[4026531837]` and `parent NoNewPrivs: 0` — that is the **init** user
namespace, i.e. produced on the host. Its `bwrap` store hash (`82xr5pn…`) also
differs from the in-jail deltas artefact's (`x4m5ja…`).

But `offjail-prompt.md` frames the credentials spike as jail-measured, and
`notes_10-12.md:509` states this jail's parent reads `NoNewPrivs: 1`. Neither is
consistent with that artefact.

`spike-deltas-output.txt` self-records `cwd=/workspace/doctrine-SL-248` and is
genuinely in-jail.

## Fix — cheap, and it would have made the whole audit unnecessary

Have every spike print, in its `### host` header:

- `uid_map`
- `readlink /proc/self/ns/{user,pid,net}`
- `NoNewPrivs`

so that "measured in this jail" becomes a **fact in the artefact** rather than a
claim *about* it. `spike-mounts.sh` already does this — it is the pattern to
copy.

## References

`SL-248` `notes.md` § *Owed* item 171 · `RV-352` · sibling of `ISS-335` (the
unpinned-`PATH` half of the same "the artefact does not carry its own
preconditions" class)
