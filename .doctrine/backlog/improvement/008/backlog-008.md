# IMP-008: Reconcile skill + audit/reconcile seam disentanglement

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Enact the `audit → reconcile → close` seam as built machinery, not discipline.
Named-but-deferred follow-on of ADR-003 §7/§11 + ADR-009 — not a fresh idea.

## What canon already decided

- **ADR-003 §7** — `/audit` and `/reconcile` are distinct steps with a hard edge.
  Audit *identifies* spec changes + assembles reconciliation context; it does NOT
  write them. `/reconcile` *writes* the identified spec changes against observed
  truth — the **sole explicit writer** of reconciled spec truth (§5), never
  derive-by-precedence (doctrine's differentiator).
- **ADR-009 §1** — the `reconcile` FSM state + closure-seam topology are already
  BUILT (`slice status` refuses `→reconcile` except from audit, `→done` except
  from reconcile); `reconcile` conduct defaults to `gate`.

## The recorded violation this closes (ADR-003 §7, amended by ADR-009)

Today `/audit` **over-reaches**: it writes spec/governance fixes in place (the
"design was wrong → reconcile `design.md`" disposition) instead of identifying
only. `/close` reconciles only slice *status* vs the phase rollup, never specs;
§8's spec-coherence closure gate is discipline-only. The seam is doctrine-by-
discipline, unenforced.

## Scope (deferred pieces, ADR-003 §11 / ADR-009 §11)

- **`/reconcile` skill** — the writer half of the seam; sole spec-reconciliation writer.
- **Reconcile artefact** (≈ spec-driver *revision*) — durable record of what
  reconcile changed and why. Name provisional; schema deferred (ADR-003 Neutral).
- **`slice reconcile` / spec-patch CLI** — the verb surface the skill drives.
- **Retune `/audit`** — strip spec/governance writing; identification + context-prep only.
- **Retune `/close`** — reconcile owning *specs*, not just status; the §8 closure gate.
- **Routing wire** — add the `/reconcile` row to `boot.md` routing table ONLY when
  the skill lands (F2/F14 shipped-not-reachable — never point routing at a deferred skill).

## Concrete stale prose found at SL-043 close (2026-06-12)

The "Retune `/close`" scope line above has a concrete, shippable backlog: the
`close` skill (`.claude/skills/close/SKILL.md`) prose predates SL-040 (lifecycle
verb + RV-ledger) and actively misdirects. Fix when this lands:

- **Tooling-gaps callout (lines ~10–14) is false** — claims "no lifecycle
  transition verb, status is hand-edited." SL-040 shipped `doctrine slice status
  <id> <state>`: classifies the move (advance/back-edge/skip/abandon), enforces
  the closure seam (→reconcile only from audit, →done only from reconcile) AND
  the D-C9b close-gate (refuses →reconcile/→done while an RV targeting the slice
  carries an unresolved `blocker`). Terminal set is `{done, abandoned}` (ADR-009),
  and the verb refuses *leaving* a terminal status — not `is_terminal_status` v1
  `{"done"}`.
- **Step 3 "hand-edit `slice-nnn.toml` status" is wrong** — use the verb
  (`doctrine slice status <id> done`, bare number); the verb writes the file, do
  NOT hand-edit. If authored status lagged (the `⚠` case), the path must pass
  `…→audit→reconcile→done`. Note done slices are hidden from default `slice list`
  — confirm via `slice show <id>` / `slice list --all`.
- **audit.md references (input line ~19; pre-check ~28–30) retired (SL-040)** —
  input is the RV reconciliation ledger (`RV-NNN`); pre-check is every finding
  *terminal* (`verified`/`withdrawn`), ledger `done · await=none` (`review status
  RV-NNN`); harvest target is `notes.md` + the RV `## Synthesis`, not `audit.md`.
- **Step 4 close-gate is now mechanical** — "do not close if a blocker remains"
  is partly enforced by the binary (the transition refuses); frame the skill check
  as believe-and-verify, not the sole guard.

## Sequencing

- **After IMP-001** (RV review-ledger + `/review` family) — per the user; review
  and reconcile are the two halves of the §6/§7 seam tuning, and the `RV-` ledger
  (ADR-007) is the shared record mechanism reconcile's artefact may build on.
- Likely splits across spec-machinery prerequisites (tech specs, coverage blocks)
  still deferred in ADR-003 §11 — scope at `/slice` time, don't presume one slice.

## Refs

ADR-003 §5/§7/§8/§11 · ADR-009 §1/§3/§11 · spec-driver ADR-004/ADR-008 (ancestors)
· IMP-001 (sibling `/review`, lands first) · `doc/slices-spec.md` § Forward compatibility.
