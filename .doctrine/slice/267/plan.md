# Plan — SL-267 Shipped-corpus conformance

Executable phase plan for the locked design (run `dr-01a0d8ff-b311`, revision 35).
Authoritative rationale lives in `design.md`, not here; this file says **why these
phases, in this order, with these boundaries**, and records what the plan's own
critical pass found. The structured criteria are in `plan.toml`; runtime progress
lives under `.doctrine/state/`.

## What one change buys

One reader — a client agent with no doctrine corpus — is failed in three ways
(citation, accuracy, sufficiency) and all three rest on one question: *what may a
shipped claim be grounded on, and at which tier*. So the plan settles the rule
first, then applies it, then proves it against that reader.

## Sequencing

| phase | delivers | why here |
|---|---|---|
| 01 | ADR-024 + two published reference docs + manifest rows | The rule and its addresses must exist before anything sweeps against them, and before any replacement can point at a resolvable home. Also the smallest phase that can be reviewed for *whether the rule is right* before 239 sites are rewritten against it. |
| 02 | `install/` sweep + the two hard cases + the boot pointer | The rule's largest sub-corpus, and the one that reaches clients through five seams. The hard cases (`inquiring.toml`'s repoint, `doctrine.toml.example`'s inlined whys) are *of* this corpus and land in the same phase that touches their files, so no `install/` file is edited twice across phases. |
| 03 | `memory/` + `plugins/` sweep | The other two sub-corpora, with their own rebuild paths (masters: hand-edit → build → sync → install; skills: recompile the embed). Kept separate from 02 because the rebuild discipline and the evidence (materialised copy) differ. Also the phase that closes a rule already written and violated in 74 places. |
| 04 | accuracy pass | Reads the *swept* corpus: a claim corrected before the citations moved would be verified twice. Running it after 02–03 means every claim it verifies is a claim in its final text. |
| 05 | sufficiency pass | Adds orientation to the same finished text. Last content phase, so its new prose is not itself swept by 02–03 (it is authored to the rule at 01). |
| 06 | client-read verification + F-7 control | The acceptance test is the reader, not the diff; it must run on the finished corpus. |

**One deliberate departure from per-file ordering.** The design asks for
per-changed-file coherence, and 02–05 otherwise honour it. Three files are
touched by two phases: `install/using-doctrine.md` (swept in 02, gains the
corpus-health/reports/config/facets sections in 05), `install/claude-activation.md`
(swept in 02, its `SL-250`-narrated claim adjudicated in 04), and
`mem.signpost.doctrine.dispatch` (swept in 03, whose sweep includes its dangling
key — axis A). Each of the three commits is coherent *by axis*, which is what the
evidence sets need; forcing per-file would either fragment an axis across two
phases or pre-empt a later phase's pass. Recorded, not accidental.

## Verification posture

This slice changes prose, and there is no automated oracle for it that is not the
deferred drift gate — which design R7 puts out of scope and `ISS-309` part 2 owns.
So the plan does not pretend otherwise:

- **VA carries the semantic weight**, and its evidence is re-derivable from
  committed artefacts: a disposition ledger in `notes.md` (`file:line | class |
  disposition | resolution`), the per-channel control log, and
  `doctrine library show <address>` for every repoint.
- **VT anchors only what is mechanical**: the two manifest rows; the three
  first-party tests that already guard shipped guidance
  (`e2e_claude_install.rs`'s fragment-store allowlist, retired-verb sweep and two
  command-acceptance tests); the presence of the named replacements this plan
  commits to in `inquiring.toml`, `routing-process.md`, the knowledge signpost,
  `glossary.md`, `using-doctrine.md` and `model-band.md`.

A VT mandate asserts *presence*, never absence, so no VT row can stand in for a
sweep. That is why the sweep phases' verification is VA plus the negative control
in 06, and why "the id is gone" is never recorded as evidence (DEC-314).

## Critical pass (what this plan assumes, and where it is thin)

| finding | disposition |
|---|---|
| **PHASE-02 is the plan's largest phase** — 141 raw sites across docs, templates, prompts, config and integration assets. | Accepted, bounded: the ledger is pre-enumerated site-by-site, so the work is classification plus edit, not discovery. If `/phase-plan` finds it still oversized, split along the seam already named — published docs vs projected/rendered/served/installed assets — without renumbering. |
| **The sufficiency homes are the design's thinnest area.** Design sec-6 gives a *tier* per gap, and sec-7's surface table does not list the sufficiency deliverable at all. | Resolved at plan level within the design's rule: `using-doctrine.md` for the verb-group sections, `model-band.md` for `prompt`, and the existing domain doc per surface (EN-3). VT-1/VT-2 pin those files. A new published doc is the recorded fallback and needs a manifest row — not a silent third home (EX-5). If it becomes a genuine fork, `/consult`. |
| **`customization = "customizable"` on both new rows is a plan-level call**, made for consistency with the reference family. `design-run-obligations.md` is load-bearing for a `fixed` runbook, so a project could override the rationale a sealed runbook points at. | Noted, not treated as blocking: the same is already true of `using-doctrine.md`, which `review-ledger.md` points at. The phase confirms the value against the manifest's own convention; if the runbook-points-at-overridable-doc coupling is judged a defect, that is a manifest-policy question for a follow-up, not for this slice. |
| **The `F-7` control lives in the final phase**, so a vacuous read is discovered late. | Mitigated by the control's own failure semantics: any channel whose planted id goes unflagged is recorded as a *failure* and the read is repaired in-phase (EX-2), so the phase cannot exit green over a blind read. |
| **Three embeds, three rebuild paths**, and a skipped rebuild makes the change invisible while every gate stays green (R5). | EN-2 in PHASE-03 exercises both paths on one file before the bulk edit, and EX-5 requires the materialised copy to differ from the pre-phase copy. PHASE-04/EX-5 repeats the requirement for corrected docs. |
| **`publication validate` proves a declared address has a backing, not that the prose stands** (design sec-2). | Stated so green gates are not mistaken for conformance: the gates are a floor, and 06's read is the evidence. |
| **`IMP-484`, `ISS-309` and `CHR-080` are evidence, not instruction** — one of IMP-484's claims was already false. | EN-1 of PHASE-04 and PHASE-05 require live re-derivation before any instance is trusted; EX-1 of PHASE-05 records every gap, so a re-verification that changes the set is visible rather than silent. |

## Out of scope (carried, not absorbed)

- The drift gate (`QUE-227`, `ISS-309` part 2) — the only durable defence against
  re-drift; this slice raises the floor and leaves the gate.
- `ISS-215` (boot-index defect) — engine code; PHASE-05/EX-7 keeps it with its owner.
- `CHR-081` — consolidating the two local memories that partly restate the rule
  (local-memory health is a non-goal corpus).
- `CHR-036` — distilling project-local memories into shipped.
- No `src/**` change: the sweep cannot ride a new lint, and none is added.

## Closure

Done is a client agent reading the shipped corpus and finding every citation
resolving or standing alone, every CLI claim true of the binary, and every
surface it is told to use documented at a reachable tier — evidenced by the
three per-axis sets and the per-channel control log, with `doctrine doctor`,
`doctrine check gate`, `publication validate` and `e2e_claude_install.rs` green.
