# IMP-316: doctor check: source anchor liveness and identifier form

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Gap

A tech spec's `[[source]]` anchors are the binding between governance and the
code it governs — and **nothing checks them.** `doctrine spec validate` proves
FK integrity on the relational spine (`descends_from`, `parent`, members,
interactions) and says nothing about whether an `identifier` still names a path
that exists. The failure is silent and it inverts the signal: a spec with a dead
anchor validates *clean* and reads as covered while governing nothing.

Measured during the CHR-046 census: **9 of 72 anchors (12.5%) were broken across
6 of 26 tech specs**, every one of those specs `active` and validating clean. A
one-line existence check found all 9.

## The check has two legs, because the failures were two kinds

**Leg 1 — liveness.** For each `[[source]]`, assert the `identifier` resolves
against the project root. This catches genuine rot: code moved or deleted with
the anchor left behind. Both real instances were exactly that — SL-082 PHASE-05
deleted the `doc/` tree (its earlier phases repointed prose references but missed
five anchors), and IMP-226 removed `src/skills.rs` into `src/install.rs`.

**Leg 2 — identifier form.** The corpus convention is **path-form in
`identifier`, Rust-path in `module`** (`identifier = "src/map_server/mod.rs"`,
`module = "doctrine::map_server"`). SPEC-020 was the sole deviation across 26
specs, writing the module path into `identifier` and duplicating it in `module`
for all three of its anchors — so they read as dead to a liveness check while the
code was present the whole time. Without leg 2 this class reports as rot and
gets "fixed" by repointing a file that was never missing.

Leg 2 is a convention with no written home; it should be stated (SPEC-017 is
the natural site, alongside the `descends_from` rules) as part of doing this.

## Caveat the implementation must not get wrong

Anchors are declared at **module-root granularity**: SPEC-001 anchors only
`src/priority/mod.rs` for 21k loc, SPEC-025 anchors `web/map/src/app.ts` for the
whole SPA. So the check is *"does each declared anchor resolve"*, **not** *"is
each source file anchored"*. A naive per-file coverage audit generates a flood of
false gaps — sibling files inside an anchored module are covered, not dark.

Anchors are also legitimately non-Rust (`language = "directory"` for `plugins/`,
`"json"`, `"toml"`, `"markdown"`), so the check is a filesystem-existence probe,
not a Rust-module resolution.

## Sibling prior art

[[IMP-309]] proposes the same shape of check one surface over — publication
entries that declare a `backing` which no longer resolves to bytes, invisible
until `library show` fails at runtime. Same defect class (a declaration
validated for well-formedness but never against reality), same fix shape (a
`doctor` leg asserting resolution). Worth designing the two together, or at
least sharing whatever reporting shape lands first.

## Provenance

The census method that found this is the deterministic support [[IMP-295]] wants
under `/spec-coverage-assessment` — that skill currently depends on an agent
remembering to run an ad-hoc existence check. Making it a `doctor` leg makes it
mechanical and moves it out of the skill's judgement budget.

The 9 anchors themselves are fixed: SPEC-020's 3 at `92d46941`, the remaining 6
under [[CHR-046]] (corpus now 44 specs, 70 anchors, 0 dead). This item is the
check that stops them recurring, not the repair.

Durable context: `mem.pattern.spec.source-anchor-liveness-unchecked`.
