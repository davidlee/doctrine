# CHR-076: Repoint open backlog items that name design show as the envelope read

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The fourth population

`SL-246` / `DEC-261` reclaimed `doctrine design show <SLICE>` for the design
document and moved the turn envelope to `--format prompt`. The migration's
population has now been counted four times and been short three of them:

| counted by | covered | missed |
|---|---|---|
| `design.md` § 3.3 `F5`, priced into `EX-7` | 4 emitted strings, 1 prose line, the test call sites | the memory corpus |
| `CHR-073` (from `RV-370` `F-16`) | + 5 committed memories | other slices' authored notes |
| `RV-372` `F-15` (the audit's corpus sweep) | + 5 `notes.md` sites | this item |
| `CHR-073`'s discharge (2026-09-20) | + 2 more authored sites the `F-15` sweep filtered out by looking only at `notes.md` | — |

`mem_01a0b2a8431371539e7911821e9c8da4`, recorded from `RV-370` `F-16`, says a
design that prices a change must **name its population** or a reviewer cannot
check the figure. Its own worked example is this migration. Four passes in, the
lesson keeps being re-learned rather than applied, which is the more useful
version of it.

## Why these are not historical citations

A review ledger, an observation record or a closed slice's notes cite the verb as
a **symbol in a record of what happened** — those are correctly untouched, and
`RV-372` `F-15` excluded them on exactly that ground (`SL-233` is the worked
example).

An **open** backlog item is different. It is read by whoever picks up the work,
and it describes a surface that has since moved. Its proposal may already be
satisfied, may no longer parse, or may now need a flag it does not name.

## The enumerated sites

Listed so this can be discharged against a list rather than a re-derived grep.
Line numbers drift; the ids do not.

| item | site | what it says | why it moved |
|---|---|---|---|
| `ISS-298` (*`design show --full` widens nothing*) | `:12`, `:13`, `:22`, `:29` | the repro is a bare `design show 243` / `design show 243 --full` | `--full` now **requires** `--format prompt\|json\|status`; the bare repro no longer reaches the code under test |
| `ISS-320` | `:43`, `:115` | *"`doctrine design show` / `resume` print each section's fingerprint"*; proposes `--emit-adoption` on `design show` | fingerprints render under `--format prompt`; a flag proposed on the bare verb now lands on the document render |
| `ISS-348` | `:32`, `:62` | *"`doctrine design show` and `show --full` print only …"*; proposes a `--authored` flag on `design show` | same two reasons |
| `ISS-357` | `:9` | *"`design resume` / `design show` print a `governance-confirmed — current` line"* | envelope-only; `SL-254`'s `design.md` § carried the same claim and was repaired under `CHR-073` |
| `IMP-430` | `:9`, `:39` | *"There is no `design show --section <id>`"*; proposes one | needs to say which rendering the section read belongs to — the document render is now the obvious home, which may make the item cheaper than it reads |
| `CHR-067` | `:9` | the `/design` skill's Recovery block writes `design resume` and `design show` | the skill prose it is about has since been migrated; re-check whether the chore is already discharged |
| `IMP-393` | `:16`, `:30`, `:78` | *"`design show` is the exception. It renders the **writer's turn envelope**"*, and `:78` proposes precisely the reversion `SL-246` shipped | **partially fulfilled** by `SL-246` (`fulfils`, degree `partial`). Its premise paragraphs are now the history, not the present; what stays open is the slice's other three documents and the fuller reader-facing render |

## Done when

Each of the seven reads correctly against the post-`DEC-261` surface: the verb
carries `--format prompt` where run state is meant, is left alone where the
document is meant, and any proposal keyed to the old default states which
rendering it now targets. `IMP-393` additionally records what `SL-246` already
discharged so the next reader does not re-propose it.

## Not in scope

Historical records — review ledgers (`RV-370`, `RV-345`, `RV-364`, `RV-366`),
observation records, knowledge records (`DEC-260`, `DEC-261` are the decision
itself), `RFC-011` / `RFC-026` notes, and closed or `done` slices' artefacts
(`SL-233`, `SL-244`, `SL-248`, `SL-259`). These cite the verb as it was and are
correct as they stand.

`plan.toml` criteria are immutable-append and are not an edit surface —
`SL-259`'s `VA-2` (*"`doctrine design show 244` reads"*) is left alone. Checked:
`SL-259` is `done` and `ISS-315` is `resolved · fixed`, so nothing live depends
on it.

Related: `CHR-073` (the memories and the authored notes), `RV-372` `F-15`,
`SL-246`, `DEC-261`, `mem_01a0b2a8431371539e7911821e9c8da4`.
