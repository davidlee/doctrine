# SL-056 — worker-mode floor decision (C vs A→B)

**Status:** OPEN — staged for adversarial review (codex GPT-5.5) before lock.
**Owner steer:** lean **C** (preserve solo-in-worktree + implementation simplicity);
test it adversarially vs the A→B alternative before committing.
**Surfaced by:** PHASE-05 dispatch — the worker built to the plan/design §3, which
conflicts with the locked ADR-006 D2a fail-closed amendment. See `notes.md`
PHASE-03 lock + this slice's design §3.

---

## 1. The conflict (newly surfaced; the G2 lock never weighed it)

The SL-056 G2 amendment (commit 742d839) rewrote **ADR-006 D2a** to a **fail-closed**
signal:

> `worker_mode = (is_linked_worktree && marker_present) OR env DOCTRINE_WORKER` …
> **a linked worktree whose marker is *absent* is treated fail-CLOSED** — the
> Orchestrator/Hook-mint/write classes are **refused** there … The legit orchestrator
> is unaffected: it runs at the coordination root (`!is_linked_worktree`).

This makes **location** gate: *any* linked worktree refuses privileged writes,
marker or not. But **ADR-006 D6a** (unchanged) says the opposite, verbatim:

> Solo `/execute` … worker-mode is OFF, and it **writes doctrine state directly** …
> The worktree is merely *where* each runs; the **mode, not the location, decides who
> may write.**

Both cannot hold for the cell **`is_linked + marker_absent + no env`**. D2a says
refuse; D6a says allow (it's a solo writer). **The G2 adversarial review (codex +
Opus) closed the stamp-failure hole but never recorded that the fix contradicts D6a
and kills solo-in-worktree direct-writing.** That is the new cost this note puts on
the table.

## 2. The hole the fail-closed rule defends — and why the belt already contains it

The hole: on **claude**, the marker is stamped by a `SubagentStart` hook that is
**not fail-closable** (read-only event; PHASE-03 finding 2). On the happy path the
stamp is *mechanically present before the worker's first action* (the hook blocks;
PHASE-03 finding 1). The hole opens only on **hook failure** (broken binary,
provision error mid-stamp, disk error) → claude spawns the worker **unstamped**.
This is a **low-probability operational accident, not an adversary.**

Blast radius of an unstamped worker doing doctrine writes — traced to where they land:

| write target | fate | contained by |
|---|---|---|
| authored `.doctrine/{slice,adr,memory…}` | committed in the fork, but **R-5 import belt rejects any `.doctrine/` path in `B..S`** → report+halt | **import belt** |
| runtime/withheld `.doctrine/state/**` | gitignored, **never in the import delta**, reaped at `gc` | **withheld tier** |
| id minting (trunk-side, D3) | `validate`/reseat detects collisions | validate/reseat |

The PHASE-03 notes already concede the point: *"the fence against an unstamped
worker is the **import belt + worker-mode guard + the pre-distilled prompt**, never a
hook exit status."* **The belt is the real fence.** Fail-closed-on-marker-absent is
belt-and-suspenders against an accident whose blast radius the belt already bounds —
at the cost of the D2a/D6a contradiction and the solo-in-worktree workflow.

## 3. Option C — drop fail-closed; the belt is the fence (RECOMMENDED)

**Rule:** `worker_mode = (is_linked_worktree && marker_present) OR env
DOCTRINE_WORKER`. Marker **present** in a linked worktree ⇒ refuse (confirmed
worker). Marker **absent** ⇒ allow (solo/orchestrator — D6a's mode-not-location).
The unstamped-worker accident is caught at **import (R-5 belt)**, not at the CLI
write seam.

**Why it's the smart bet:** preserves the useful solo-`/execute`-in-a-worktree
workflow, keeps D6a's clean "mode not location" model, and is the **simplest**
implementation. **It is also exactly what the PHASE-05 worker already built**
(`ec81b5e` on `sl056-coord`) — so C keeps the delta and most of design §3 intact.

**Proposed amendments under C:**

- **ADR-006 D2a — REVERT the fail-closed clause.** Replace the "a linked worktree
  whose marker is absent is treated fail-CLOSED … refused" sentence with: marker is
  the **positive** worker signal; **marker-absent is NOT worker-mode** (solo /
  orchestrator may write — D6a). State explicitly that **the import R-5 belt
  (`.doctrine/`-touch rejection) + the env worker-on-main catch + the pre-distilled
  prompt are the fence** against an unstamped claude worker; the marker is the
  agnostic *positive* identity, not a fail-closed floor. Keep the `marker
  --stamp-subagent` verb-identity exemption note (still valid). **Record the D6a
  conflict + the belt-containment rationale as the reason for the revert** (so the
  re-amendment isn't a silent flip of reviewed governance).
- **ADR-006 D6a — UNCHANGED** ("mode, not location, decides"). The revert restores
  consistency with it.
- **ADR-011 D6 (φ cell) — ALIGN (minor).** It already confesses the SubagentStart
  fail-open-on-hook-failure altitude; reframe it as the **accepted** posture (belt is
  the fence), not a regretted residual. No structural change.
- **design.md §3 — UNCHANGED in substance.** The original marker-primary model
  (direct-writer entry via `status --assert`; `marker --clear` cures a self-brick;
  solo-isolated writes freely) is **consistent with C**. Light edit only: drop the
  "fail-closed on marker-absent" language if any leaked in from the G2 pass, and add
  a one-line pointer to the belt-as-fence rationale.
- **plan.toml PHASE-05 — UNCHANGED.** EX-1/VT-1c ("solo neither-signal writes
  freely") are correct under C.
- **PHASE-05 worker delta `ec81b5e` — KEPT.** It implements C. Re-verify against the
  amended ADR; resume the funnel (record + flip completed).
- **`marker --clear` — KEPT** (its self-brick-cure purpose holds under C).

**Residual risk accepted under C:** an unstamped claude worker that does doctrine
writes wastes effort (rejected at import) and could mint a stray id (caught by
validate/reseat); a *malicious* worker is not CLI-fenced at the write seam (only at
import). Judged acceptable: workers are our own pre-distilled agents, not adversaries.

## 4. Option A→B — keep fail-closed now, add a writer-marker later (ALTERNATIVE)

**Rule (A):** refuse iff `is_linked_worktree OR env` (any linked worktree). **(B,
deferred):** add a distinct **writer-marker** the orchestrator/solo arms →
`is_linked + writer-marker → allow`. A is the fail-closed floor; B is a strict
additive superset (A = B with no writer-marker ever armed), so A→B later is cheap and
non-breaking; B→A would be a capability removal.

**Implied edits under A:**

- **ADR-006 D2a — KEEP** the fail-closed clause as locked.
- **ADR-006 D6a — AMEND.** "Mode, not location" must yield: *in a linked worktree,
  location gates (fail-closed); solo writes doctrine state at the coordination root;
  isolated solo code funnels via `land`.* (Resolves the contradiction by overturning
  D6a's worktree case.)
- **design.md §3 — REWRITE** the direct-writer/assert/marker-clear mechanics:
  `status --assert` refuses any linked worktree (names the coord-root remedy);
  **`marker --clear` is DROPPED** (clearing doesn't re-enable writes under
  fail-closed — its self-brick-cure purpose is void); solo-isolated direct-write
  removed.
- **plan.toml PHASE-05 — REWRITE** EX-1/VT-1c/VT-3/VT-4: marker-absent-linked →
  refused; drop the `marker --clear` criteria; `status --assert` linked → refused.
- **PHASE-05 worker delta `ec81b5e` — DISCARD + RE-DISPATCH** with the fail-closed
  rule (`refuse iff is_linked OR env`), no `marker --clear`, rewritten `status`/
  `--assert`.
- **`backlog new`** the B writer-marker as a deferred additive option (capture, don't
  lose).

**Cost:** higher churn (re-dispatch + rewrite §3/plan + amend a second ADR clause),
loses solo-in-worktree direct-writing until B lands.

## 5. The decision to adversarially test

> **Is the marker fail-closed-on-marker-absent floor worth keeping (A/B), or does the
> import R-5 belt already contain the realistic blast radius such that dropping it (C)
> — preserving D6a, solo-in-worktree, and the existing worker delta — is the better
> trade?**

Recommended: **C.** Test it before lock.

## 6. Adversarial charge list (for the codex pass)

1. **Belt-containment claim (load-bearing for C).** Is it *actually* true that **no**
   doctrine-mediated write by an unstamped worker can reach the coordination branch
   except via the R-5 belt? Probe: doctrine-driven **commits** the worker might make;
   memory **anchoring** to a fork commit; any write that targets the coordination
   root rather than the worker's cwd; `gc`/`land` paths; symlink/`.doctrine` aliases
   the belt's name-only check might miss. If any uncontained path exists, C weakens.
2. **Minting pollution.** Can an unstamped worker mint/reseat ids with effects that
   survive (collisions, wasted ids, anchor drift) that validate/reseat does **not**
   cleanly catch? Quantify the real harm.
3. **Self-clear under C.** Under C, `marker --clear` re-enables writes — does that
   reopen a meaningful attack/accident the G2 review closed? Or is it moot because the
   belt fences the result anyway?
4. **D6a consistency.** Does C *fully* restore D2a/D6a consistency, or does some other
   clause (D2b, D7, D8, ADR-011 D6) still assume the fail-closed floor?
5. **Reversibility.** Is A→B genuinely additive/non-breaking, and is C→(fail-closed)
   genuinely cheap to add back later if the belt proves insufficient? If C is hard to
   reverse, that raises the bar for choosing it now.
6. **Re-amending fresh governance.** Is reverting a just-locked, adversarially-reviewed
   D2a clause justified solely by the newly-surfaced D6a-conflict + workflow cost — or
   is there a substantive security reason the G2 review had that this note understates?
7. **codex/pi path.** C leans on the env catch for worker-on-main. Does dropping
   fail-closed change anything for the codex/pi subprocess path (which *does* have a
   reliable env seam), or is the whole question claude-specific?
