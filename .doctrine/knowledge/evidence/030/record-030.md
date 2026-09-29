# EVD-030: Byte comparison detects every content-conversion and freshness-suppression route the three legs miss

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## What was measured

The parked handover of **SL-232** registered a spike as decision 3 and never ran
it: *does comparing raw worktree bytes against the HEAD blob replace the
three-leg git-semantic dirt observation?* It is now run —
`probes/byte-compare.sh`, git 2.54.0, 2026-09-29.

For every hazard where the three legs (`diff HEAD --binary`,
`ls-files --others --exclude-standard`, `diff-index --quiet --cached`) read
*clean*, the byte primitive (`hash-object --no-filters` vs `rev-parse
HEAD:<path>`) reads *divergent*:

| hazard | legs (tracked / untracked / cached rc) | bytes |
|---|---|---|
| committed `.gitattributes` `text eol=crlf` | 0 / 0 / 0 | diverges |
| committed `clean` filter (arbitrary content exposed) | 0 / 0 / 0 | diverges |
| `.git/info/attributes` `clean` filter | 0 / 0 / 0 | diverges |
| same-size mtime-preserved edit, `core.trustctime=false`, tag `H` | 0 / 0 / 0 | diverges |
| `GIT_WORK_TREE`-redirected worktree | 0 / 0 / 0 | diverges |

The control arm (a different-size content edit) reads dirty on **both**
instruments, so the arms discriminate. All seven registered falsifiers held.
Two failed on the first run and both were **probe fixture bugs, not hypothesis
failures** — an empty "clean" worktree, and a mis-stated symlink expectation —
recorded in the probe rather than smoothed over.

## The limits, also measured

1. **Bytes do not enumerate the set.** A deletion fails `hash-object` (rc 128)
   and must be called divergent; an extra untracked file is in `--others` with
   no HEAD blob; a *staged add* is in `--cached` and **absent from `--others`**,
   so an `--others`-only enumeration misses it. The surface needs one call
   spanning `--cached` + `--others` + HEAD.
2. **`hash-object` follows symlinks.** On `link -> f` it returns `f`'s content
   oid, not the blob oid of the target string; it errors on a dangling link. It
   is therefore **not** the mode-`120000` primitive — link equality is `readlink`
   versus `cat-file blob`. The live corpus carries 3,398 symlinks under
   `.doctrine/**`.
3. **The mode bit is outside the byte claim.** `100644 -> 100755` reads
   `tracked=51` on the legs with byte-identical content, so byte comparison
   alone *regresses*. "Replace the three legs" must mean **bytes + mode + set**.

## Cost

Batched `hash-object --stdin-paths --no-filters` hashes the whole live
`.doctrine/**` surface (13,138 paths, 3,398 links) in ~80 ms; per-file spawning
is ~70× slower. Cost is not the constraint; set, symlink and mode correctness
are.

## Reproducibility

`probes/byte-compare.sh` exits 0 and is idempotent across two runs with timing
normalised. Absolute byte counts are fixture-dependent; the durable claim is the
**discrimination** (zero vs non-zero, exit 0 vs 1). Live corpus counts are
context, not a fixture claim, and must be re-measured before being quoted.
