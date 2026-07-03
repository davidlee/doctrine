# SL-193 — Exposed-slot self-replaces projection

> Reference forms: entity ids padded (SPEC-023, REV-019, SL-191); doc-local refs
> bare (D1, OQ-1). Governing design **locked by REV-019** (done, approved)
> against **SPEC-023** — this doc is the implementation design, not a re-litigation
> of the mechanism choice.

## Problem

Every **exposed** hymn slot double-emits. `prompt explain` (any matching
context) shows each exposed slot's Framework snippet and its projected User twin
both surviving:

```
harness/claude                   prov=Framework spec=(1,0) rank=1
harness/claude                   prov=User      spec=(1,0) rank=2
role/worker                      prov=Framework spec=(1,0) rank=1
role/worker                      prov=User      spec=(1,0) rank=2 ★ WINNER
```

`★ WINNER` is the **provenance tiebreak in the precedence key** — an *ordering*
term, never suppression. INV-2: only `replaces` suppresses. The projected User
twin carries no `replaces`, so it **appends** to its Framework origin instead of
overriding it. SPEC-023's narrative promised override ("the user wins the
same-slot tiebreak — the legitimate customisation"); the mechanism delivered
append. ISS-206 reported the `role/worker` instance (visible in the baked worker
def); the defect is **corpus-wide** across all exposed slots.

Not the accepted double-emit valve: SPEC-023's "double-emit at box intersections
… wasteful, never incorrect" covers **author-chosen overlapping selector boxes**.
A seal/expose **projection twin** is not that — it is a projection artifact, and
the customisation case (`B` framework + `B'` user edit) is incorrect-against-
intent, not merely wasteful.

## Locked design (REV-019)

Deliver override through the **existing** suppressor. Expose becomes the
single-emit **mirror of seal**:

- **seal** → drop the *user* twin before matching → framework wins (delivered,
  REQ-323).
- **expose** → keep the user twin; it carries `replaces = <own slot>` → it
  suppresses its *framework* origin → user wins (**this slice**, REQ-322).

`replaces` stays the sole suppressor; the precedence key stays ordering-only;
INV-2/INV-3 are untouched. REV-019 pressure-tested the self-`replaces` path
against unchanged `src/hymns.rs`:

- `is_unique_top_of_slot(U)`: same slot ⇒ same band/specificity/alpha; provenance
  `User > Framework` ⇒ `U` is strict-max ⇒ unique top. **INV-3 passes.**
- self-edge `own == target` excluded from the cycle graph ⇒ no false cycle.
- suppression loop keeps `j == carrier`, drops the rest ⇒ suppresses `F`, keeps
  `U`. Output = `U` only.

Rejected (REV-019): *implicit same-slot override* (equal-specificity higher-
provenance suppresses without `replaces`) — mutates the core compose rule to fix
a projection-specific problem. Not taken.

**Engine (`src/hymns.rs`) is therefore unchanged.** The fix lives entirely in
the projector (`src/install.rs`): make it *emit* the self-`replaces` sidecar and
give it a live call site.

## Current state (why the fix has two parts)

1. **The producer is dead code.** `project_starters` (`src/install.rs`) is
   `#[expect(dead_code, reason = "reserved: SL-187 …")]` — never called. SL-187
   is `done` but never wired disk-projection of exposed starters. The sidecar has
   no live producer to attach to.
2. **The twins are orphan starters.** `.doctrine/hymns/**` holds byte-identical
   copies of all 5 exposed slots (`harness/claude`,
   `model/anthropic/claude-sonnet-4`, `model/deepseek/_default`,
   `role/orchestrator`, `role/worker`) — now **tracked/authored** (gitignore
   fixed to include hymns; user edits persist). None carries a sidecar → all
   double. (Disk `preamble/core.md` is a *sealed*-slot orphan — seal drops it at
   resolution; harmless. `README.md` is not a slot — loader skips it.)

So conformance to REV-019 requires **wiring the producer (A)** *and* the produced
sidecar; a sidecar-less producer would leave the requirement inert.

## Target behaviour

After this slice, for every exposed (non-sealed) slot:

- projection writes `.doctrine/hymns/<band>/<label>.md` (starter) **if absent**
  and `.doctrine/hymns/<band>/<label>.toml` carrying `replaces = "<band>/<label>"`
  **if absent** — independently per file;
- the resolver emits the slot **once** (user body; framework origin suppressed by
  the sidecar's self-`replaces`);
- an edited starter (`B'`) wins outright — framework `B` suppressed, not appended;
- an unedited starter (`B`) dedups to a single `B`.

Verified corpus-wide: `prompt explain` shows no exposed slot doubling.

## Design decisions

- **D1 — Wire `project_starters` as a live install forward-step producer (A).**
  REV-019 REQ-322 says *the projector* writes the sidecar; that requires a live
  projector. A one-shot sidecar backfill (B) satisfies REQ-322 only in letter and
  leaves no durable producer. Minimal cut: wire exactly the exposed-starter +
  sidecar projection; broader SL-187 projection concerns (README, boot
  integration) stay out.
- **D2 — Independent write-if-absent per file (i).** Decouple the current
  whole-slot `dest.exists() → continue` skip into per-file checks: `.md` written
  if absent (preserves committed user edits), sidecar `.toml` written if absent
  (backfills the existing sidecar-less twins). Idempotent, non-clobbering,
  self-healing. Rejected: pair-atomic skip (ii — never repairs existing twins),
  always-clobber (iii — destroys user edits / hand-tuned sidecar axes).
- **D3 — Sidecar single-sourced off the slot.** Body is `replaces =
  "<slot.path()>"` where `slot.path()` (`hymns.rs`) is the one source of the
  `band/label` string (STD-001, no magic string). Sidecar is doctrine-owned
  mechanism, host-agnostic (POL-002 clean).
- **D4 — Engine untouched.** `src/hymns.rs` production code does not change
  (REV-019 verified). The behaviour-preservation gate is the existing
  resolver/loader suite staying green **unchanged**.
- **D5 — Reconcile existing twins by running the wired producer, not by hand.**
  The forward step backfills the 5 sidecars; commit them. No bespoke migration.

## Code impact

- **`src/install.rs`**
  - `project_starters`: remove `#[expect(dead_code)]`; split the whole-slot skip
    into independent `.md` / `.toml` write-if-absent; add sidecar emission
    (`replaces = "<slot.path()>"`). Signature already carries
    `(disk_root, embedded, sealed, exposed_slots, dry_run)`.
  - Add `embedded_expose_set() -> SealSet`-shaped `BTreeSet<Slot>` from
    `manifest.hymns.expose` (mirror of `embedded_seal_set`).
  - `run_forward_steps`: add forward step 4 — *"Project exposed hymn starters?
    [y/N/a]"* — calling `project_starters` with the disk hymns root, embedded
    hymns, seal set, expose set. Non-fatal on error (matches sibling steps).
- **`src/hymns.rs`** — tests only (expose/seal-symmetry golden). No production
  change.
- **`.doctrine/hymns/**`** — 5 new sidecar `.toml` files (authored, committed),
  produced by running the wired step.

## Verification

- **Unit — `project_starters` (`src/install.rs`):**
  - VT: sidecar written with `replaces = "<slot>"` for each exposed slot.
  - VT: **backfill** — `.md` present, `.toml` absent ⇒ `.md` preserved (byte-for-
    byte), sidecar written.
  - VT: **idempotent** — both present ⇒ no write, no error.
  - VT: **preserve edits** — edited `.md` present ⇒ never overwritten.
  - VT: **seal respected** — sealed slot ⇒ neither `.md` nor sidecar written.
  - VT: **dry_run** — nothing written for either file.
  - `project_starters` is currently **untested** dead code — these are new
    tests (tempdir-scoped), not an extension.
- **Golden — resolver expose/seal symmetry (`src/hymns.rs`):**
  - VT: exposed slot, user twin with self-`replaces` ⇒ **single emit** (user
    body); framework suppressed.
  - VT: edited user body `B'` ⇒ output `B'` only (framework `B` suppressed).
  - (seal disk-twin-drop golden already exists — symmetry is the new half.)
- **E2E — `prompt` verbs:**
  - VT: after projection, `prompt resolve --role worker` emits `role/worker`
    once; `prompt explain` shows framework `role/worker` suppressed, not `rank`-
    ordered-but-present.
- **Behaviour-preservation:** full resolver/loader/e2e suites green **unchanged**
  (D4 gate).
- **Corpus check:** in-repo, `prompt explain` across a full context shows **no**
  exposed slot double-emitting.

## Invariants / boundary conditions

- INV-2 (append-unless-`replaces`) and INV-3 (`replaces` legal only on unique-
  most-specific active snippet) unchanged; self-`replaces` satisfies INV-3 by the
  provenance strict-max argument above.
- Seal precedence unchanged: a slot in **both** seal and expose is impossible by
  construction (manifest lists are disjoint) — but the projector guards it anyway
  (`if sealed.contains(slot) continue`), so seal wins if ever mislisted.
- Sidecar naming: loader pairs `<stem>.md` with `<stem>.toml` (`sidecars` map by
  stem, `install.rs`); the emitted `<label>.toml` matches.

## Non-goals

- Implicit same-slot override (REV-019 rejected).
- The **hand-authored-no-sidecar** general case — a user hand-creating an
  exposed-slot snippet with no sidecar still doubles (REV-019 documented known
  gap). This slice fixes the *projection* path + the existing projected twins,
  not arbitrary hand authoring. Follow-up only if demand appears.
- Set-valued trait selection (SPEC-023 FR-004/5/7/9) — **SL-192**; engine-
  independent of this slice.
- Worker-contract hymn content / deepseek patterns / bake generalization / funnel
  check-cadence — **SL-191** (which `after`s this slice; its overlay authoring
  depends on single-emit exposure).
- The gitignore→tracked tier change for `.doctrine/hymns` is done (user); the
  SPEC-023 persistence framing is now consistent. No further tier work here.

## Adversarial review (internal pass)

- **Load-bearing risk, CLEARED.** Does the *disk* loader wire sidecar `replaces`,
  or only the embedded loader? Verified `load_disk_corpus` (`src/install.rs`)
  reads `<stem>.toml`, parses `Sidecar`, `overlay_selector` → sets
  `Selector.replaces`, tags `Provenance::User`. So a disk sidecar suppresses the
  framework twin at resolve. **No loader change needed** — the "engine + loader
  untouched" claim holds.
- **Decline-the-step is safe.** If a user declines forward-step 4: no disk twins,
  resolver sees framework-only ⇒ single emit. If accepted: user twin
  self-`replaces` ⇒ single emit. The *only* broken state is disk-twin-without-
  sidecar — which arises solely from legacy orphans (fixed here) or hand-authoring
  (out-of-scope known gap). The wiring closes the orphan case.
- **Slash-labels round-trip.** `model/deepseek/_default`: sidecar path
  `.../model/deepseek/_default.toml`, `replaces = "model/deepseek/_default"`;
  `parse_slot_ref` splits on first `/` ⇒ band `model`, label `deepseek/_default`.
  Correct.
- **No step-count regression.** `run_forward_steps` is not unit-tested for step
  count (only `prompt_step` is tested in isolation); adding step 4 breaks nothing.
- **Write order.** New slot writes `.md` (creates dir via `write_atomic`) then
  sidecar; backfill writes only the sidecar into the existing dir. `write_atomic`
  must ensure the parent dir for the sidecar-only path — confirm at execute.
- **Corrected:** `project_starters` has **no** existing test (fully untested dead
  code); the VTs above are net-new, not an extension.

## Open questions

- **OQ-1 (minor) — orphan sealed twin + README.** Disk `preamble/core.md`
  (sealed-slot orphan, harmless) and `README.md` (non-slot) predate this work.
  Tidy them under this slice, or leave? Non-blocking; lean leave-README,
  optional-remove-`preamble/core.md`.
- **OQ-2 — forward-step gating.** Step 4 runs on `doctrine install`. Should it
  also be reachable standalone (e.g. a `prompt` subverb) for repos that want to
  re-project without a full install pass? Out of scope unless needed; the install
  forward-step is the REV-019-named home.
