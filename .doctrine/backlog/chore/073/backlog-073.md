# CHR-073: Re-attest five memories that document design show as the envelope read

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why this is not optional

`SL-246` reclaims `doctrine design show` for the design document (`DEC-261`);
the turn envelope moves to `--format prompt`. Five committed memories under
`.doctrine/memory/items/` document the *old* meaning.

A stale shipped doc waits to be read. A stale memory is **injected** into an
agent's context by `memory retrieve` and the `memory surface` hook — so it is
read by default, by every agent, without anyone choosing to consult it. That
makes this the worst-placed population in the whole migration, and it is the
one a code sweep and the goldens both miss (`RV-370` `F-16`; the goldens reach
neither skill prose nor the corpus).

## The five

| item | what it says now |
|---|---|
| `mem_019fc255625877e09ba55d5e11d7c5cb` | titled *"Design run state: read via show, not the raw TOML"*; instructs "Read design-run state with `doctrine design show <slice>`" and tabulates `doctrine design show 243`. **Its thesis inverts** — the correct read becomes `--format prompt` |
| `mem_019facc21a1b7c50a6d5b2eb7ec7f3c9` (`:54,58`) | compare a fingerprint against "what `design show` displays" |
| `mem_019fdf95d07979d0a7172f75381ef58c` (`:31`) | same fingerprint-comparison instruction |
| `mem_019ff439bede7fb29c7c09b7fd76d893` (`:2`) | "`design show`'s `declaration_example` shows …" |
| `mem_019fcd1727fa7061b771179b113f5726` (`:17,21`) | references the verb as the envelope read |

Verified present at `RV-370` round 2. Line numbers will drift; the uids will not.

## Why it is a chore and not a phase

Re-attesting a memory is its own verb (`doctrine memory record` / `verify`), the
corpus is not code, and none of `SL-246`'s phases touch it — a phase that edited
memories would be doing unrelated work under a code-shaped exit criterion. So it
rides a `/reviewing-memory` pass instead.

**But it must land with or before the code**, not after: between the reclaim
shipping and these edits, every agent that retrieves one of the five is told to
run a command that now does something different. Sequence it into the slice's
close, not its backlog drift.

## The third population — other slices' authored notes (`SL-246` audit, `RV-372` `F-15`)

`EX-7` priced four emitted strings, one prose line and the test call sites; this
item added five memories. A corpus sweep at the audit found a **third**
population that neither covers: other slices' committed `notes.md`, read by an
agent resuming that slice. Enumerated here so this item can be discharged against
a list rather than a re-derived grep:

| site | what it says | why it is now wrong |
|---|---|---|
| `.doctrine/slice/247/notes.md:93` | *"Live in the design run's inquiry map (`doctrine design show 247`)"* | that verb now renders the design document |
| `.doctrine/slice/253/notes.md:58` | *"`design show` will not display them, so read `[[review.finding]]` in the runtime `design.toml`"* | describes the envelope's behaviour under the old default |
| `.doctrine/slice/253/notes.md:345` | *"`doctrine design show SL-253` carries each finding, the section it concerns, and its resolution"* | renders the document, not the findings |
| `.doctrine/slice/256/notes.md:142` | *"`design show` over all seven runs"* | same |
| `.doctrine/slice/256/notes.md:369` | *"`doctrine design show 244` prints the terse message today"* | same |

**Not affected**, checked and excluded: `SL-233`'s `plan.toml` / `plan.md` /
`notes.md` cite the verb as a *symbol* in a historical record of what `PHASE-03`
landed, and shipped prose is clean — the one live hit,
`plugins/doctrine/skills/handover/SKILL.md:36`, already spells `--format status`.

The repair is the same per row as for the memories: `--format prompt` where run
state is meant, leave it where the document is meant.

## Done when

Each of the five is corrected and re-verified (`doctrine memory verify <key>`),
and none of them instructs a bare `design show` for run state.

## Folded in from `CHR-075` (closed duplicate, SL-246 audit)

The `PHASE-05`/`PHASE-06` capsule orchestrator re-derived this item independently
as `CHR-075` and added three things worth keeping:

- **The concrete repair, per row.** Correct the verb to `doctrine design show
  --format prompt` where *run state* is meant; leave it alone where the
  *document* is meant; then re-attest. The five are not uniformly wrong — one
  (`mem_019fc255625877e09ba55d5e11d7c5cb`) has its thesis inverted, the rest cite
  the verb in passing.
- **It compounds with `ISS-465`** (`memory search` ranking too poor to compete
  with raw grep). Agents already bypass the sanctioned read path, so a wrong row
  is less likely to be caught and corrected in passing than the corpus's design
  assumes.
- **Wrong is a different severity from stale.** A stale row is read, found
  unhelpful, and dropped. A wrong row is retrieved *precisely* when the agent does
  not already know the answer, and sends them to a verb that now returns
  plausible-looking output for a different question.

Also related: SL-246's `R6` (stale skill path), a reconcile input.
