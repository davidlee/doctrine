# Notes SL-245: Inline terminal diagram rendering

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage
<!-- runbook step explore.triage, design run dr-019fd12e -->

### Constraining governance

| authority | how it binds |
|---|---|
| **SPEC-027** (resp. 4) | The DOT emitter must carry "no filesystem or external-renderer dependency". The renderer is therefore sited outside it — settled before scoping, not open. |
| **ADR-001** | leaf ← engine ← command, downward only. `map_server = "command"` (`layering.toml:102`), so a leaf renderer **cannot** reuse the existing `DotRenderer`. This is the slice's central structural constraint. |
| **POL-002** facet (3) | New this week (REV-047). A feature-scoped host capability must be opt-in and must fail naming what was missing and what would satisfy it. Forbids a default-path `dot` dependency and forbids a silent fallback. |
| **STD-001** | The protocol is nothing but meaningful literals — `\x1b_G`, `a=T`, `f=100`, `m=0/1`, the 4096 chunk bound. Every one is a named constant, single-sourced. |
| **POL-001** | Prose in errors stays plain. The absence path is user-facing text. |

Checked and **not** applicable: ADR-013 (no governance dependency remains — REV-047
already landed); ADR-019 (nothing embedded or published); ADR-006/008/011/012/020
(no dispatch or worktree surface).

### Shaping decisions already taken

- **D-A. Renderer sited outside SPEC-027** — user, 2026-08-05, pre-scoping. DOT
  text in, terminal bytes out.
- **D-B. No crate.** `base64 = "0.22"` is already a direct dependency annotated
  "Leaf-legal external" (`Cargo.toml:137`); the encoder is ~30 lines; `kitty_image`
  is v0.1.0 and pins `base64 ^0.21`, which would put two base64 majors in the lock.
- **D-C. Sizing in cells, not pixels.** The protocol's `c=`/`r=` are columns/rows
  and `tty.rs::stdout_terminal_width` already reports cells. The preflight plan of
  chasing the ioctl's pixel fields is dropped.
- **D-D. No capability handshake.** Opt-in is the capability assertion. Env
  sniffing would be actively wrong — in-jail `TERM=xterm-256color` with no
  `KITTY_WINDOW_ID` against a terminal that speaks the protocol.

### Assumptions carried

- **A-1.** The user's terminal (ghostty) implements the kitty protocol's
  transmit-and-display path for PNG. Not probed; opt-in makes it the caller's
  assertion.
- **A-2.** `dot -Tpng` output is not byte-stable across graphviz versions, so no
  golden may contain real rasteriser output. Asserted from general knowledge, not
  measured — cheap to check if a golden is ever proposed.
- **A-3.** `t=d` (direct transmission) is the protocol default; the docs' minimal
  example omits it. Flagged unverified in `research/raw/protocol.md`; verify at
  point of use.

### Open questions (the inquiry map)

Carried into the run's inquiry, not resolved here. The live forks:

1. Extract the `dot` spawn down to a leaf seam and refactor `map_server` onto it,
   or write a second sync spawn and knowingly hold two. Bears on "no parallel
   implementation"; complicated by tokio/async vs sync.
2. CLI shape — separate verb, a `--render` axis beside `--format`, or a `--kitty`
   shorthand. Note `--color` is **global** (`src/main.rs:110`), so it is not the
   straightforward precedent preflight took it for; and facet (3) forces the
   default off, unlike `--color`'s `auto`.
3. Flag set but stdout is not a terminal — hard error, or DOT with a note on
   stderr.
4. Which surface is wired first, and whether SPEC-027 wants a sentence noting a
   peer renders its output.

### Risks

- **R-1.** No raster viewer in the jail and an agent cannot see an image, so
  "the picture is right" is VH-only. Test surface must be framing/encoding with a
  stubbed rasteriser — the `FakeDotRenderer` pattern (`map_server/shell.rs:73`)
  is the in-repo model.
- **R-2.** The escape sequence reaching a pipe or file corrupts every redirected
  invocation. The guard is load-bearing, not cosmetic.
- **R-3.** Scope creep toward the web/TypeScript emitters (`web/map/src/dot.ts`),
  which a Rust leaf cannot serve. Out of reach; say so rather than drift.

### Load-bearing prior art

- `src/map_server/shell.rs:33-70` — the existing `dot` spawn: `NotFound` →
  `ToolUnavailable { tool }`, non-zero → `CommandFailed { status, stderr }`, both
  legs bounded by a 10s timeout. The discrimination facet (3) wants.
- `src/tty.rs` — thin `stdout_*` impure wrapper + pure both-injected decision fn.
- `mem.pattern.design.capability-as-data-seam` (verified) — probe in the shell,
  hand the pure core a *multi-valued descriptor*; never collapse absent /
  unsupported / degraded into a bare `Option` or `bool`.

## Inquiry state — 2026-09-15

Design run `dr-019fd12e-03c3-7c92-9b87-bb48b4845e13`, stage `reviewing`. Re-enter with `doctrine design resume SL-245`. This
section carries only what the run does not.

**All seven inquiries dispositioned, each as an accepted DEC:** inq-1 → DEC-143
(two `dot` spawns held knowingly), inq-2 → DEC-253 (`--render` / `-X` per verb),
inq-3 → DEC-254 (non-terminal or non-DOT format → one descriptive error class),
inq-4 → DEC-255 (`RenderTarget` descriptor; spawn is its own probe), inq-7 →
DEC-256 (pixel-width sizing, provisional), inq-5 → DEC-257 (VH acceptance, VT
scaffold), inq-6 → DEC-258 (graph then concept-map; `-X` implies dot; SPEC-027
resp. 5 note + ISS-242 annotation at reconcile).

**inq-2 reversal, recorded.** The 2026-08-05 recommendation of a pipe verb rested
on (b) importing the renderer into SPEC-027's territory. That was overstated:
resp. 4's no-external-renderer clause binds `catalog::dot::render`, not the verb
shell (resp. 5). The user chose (b) on UX grounds; the boundary holds either way.

**User acts recorded:** `governance-confirmed`; `graph-reviewed` (re-performed
after inq-7 invalidated it — blocking set now inq-1..inq-5 + inq-7).

**Design review — RV-368 (codex gpt-5.6-sol, 2026-09-15).** 8 findings, all
disposed; see the ledger for chronology. Two reopened settled positions with the
user's agreement: a kitty support probe (DEC-259, reverses the scope's "no
capability handshake" non-goal under POL-002 facet 3) and explicit placement
geometry (DEC-256 amended). DEC-143, DEC-257 and DEC-258 amended to match.
ISS-455 captures the incumbent descendant-pipe hang in the bounded spawn.
Later passes raised F-9..F-12, all verified; F-12 moved the tty wait from `poll`
to termios timed reads (DEC-259 amended).

**Further review passes — none planned (2026-09-15, after F-12 verified).** The
RV-368 passes converged: each found fewer, narrower defects, the last a single
platform fact in the tty seam. What remains is empirical, not analytic, and a
desk review cannot settle it: timed reads on macOS `/dev/tty` (sec-9 assumption,
VH step 6), the placement rule on HiDPI (DEC-256 provisional, VH steps 1-2), and
whether ghostty emits reply text for `q=2`. One thing a further pass could still
probe on paper: keystrokes typed during the up-to-2 s probe are read in raw mode
and discarded as noise by the classifier. That is a small, unnamed residual;
raise it at section review of sec-2/sec-9 if it matters.

**Friction:** the apply payload contract does not say which subject kind honours
`dispose` / `resolution` (resolving an `inq-` takes a `cp-` subject with
`disposes`); a new node cannot be declared and disposed in one submission.
Observation recorded.

**Note on `research/`** — `.gitignore:49` excludes `.doctrine/slice/*/research/`,
so `research.md` and its `raw/` threads live on disk in this worktree only. They
are runtime tier by design; if a fresh clone needs them, re-run the round.

## Audit — 2026-09-17 (RV-369)

Implementation audit after the capsule hand-back. Phases ran in an oubliette
capsule (ADR-020) on `capsule/SL-245/c11`; `VH-1..VH-4` could not run there and
were run by the user during the audit. Ledger: **RV-369**, 9 findings, all
terminal. `doctrine check gate` green.

**Two blockers, both found by VH, neither visible to any automated leg.**

- **F-1.** `tty::endpoint` compared `st_rdev` of stdout with `st_rdev` of a
  `/dev/tty` fd. Unsatisfiable — an fd opened from `/dev/tty` fstats as the
  devnode `(5,0)`, never as the pts `(136,N)`. `-X` refused every terminal in
  existence while 17 VTs passed. Fixed: POSIX `tcgetsid` session ids.
- **F-6.** The whole corpus produced 237 rows of blank space: a 61 MiB PNG that
  ghostty silently declines, with `q=2` suppressing the refusal (F-7). `DEC-256`
  bounds *cells*, which is not a resource bound. Fixed: an 8 MiB image budget
  refusing before the send.

**The structural lesson (F-2)** is the one worth carrying: pure decisions with
injected inputs, plus e2e tests that only reach refusal paths, leave the
*adapter* — the thing that measures what the pure rule compares — entirely
uncovered. design sec-8 argued a pty harness was unnecessary *because the logic
was pure and already tested*; that sentence is how this shipped. One `script`
spawn now covers it.

**Note on the capsule path:** the runtime phase sheets
(`.doctrine/state/slice/245/phases/phase-0{2,3,5}.md`) came back as empty
template stubs — the capsule worked in its own tree. There was nothing to
harvest from them, so this audit's harvest is sourced from the ledger and the
evidence run rather than from phase notes.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-17 · reconciled (RV-369 discharged, REV-056 done) · f591accd5

### Produced

- design.md sec-1..sec-9 (run dr-019fd12e…, locked 2026-09-15; reconciled in place 2026-09-17 — direct edit out of band, per mem.pattern.reconcile.edit-design-out-of-band)
- plan.toml / plan.md: PHASE-02, PHASE-03, PHASE-05 after scope cut (PHASE-01/04/06 removed, never reused)
- IMP-451 (concept-map export -X), IMP-452 (bounded subprocess + render timeout; annotated at reconcile), CHR-072 (macOS probe check) — deferred by scope cut
- DEC-253, DEC-254, DEC-255, DEC-256, DEC-257, DEC-258, DEC-259 (accepted); DEC-143 accepted + clarified. DEC-256/DEC-259 amended at reconcile (RV-369 F-1, F-6); DEC-255/DEC-143 corrected for the scope cut
- RV-368 (design review, codex gpt-5.6-sol): F-1..F-11 verified; F-12 verified (termios timed reads)
- ISS-455 (bounded-spawn descendant-pipe hang, incumbent in coverage_verify)
- Implementation: src/{graphviz,kitty,terminal_image}.rs + src/tty.rs render half; `doctrine graph -X`
- RV-369 (implementation audit): F-1..F-9, all terminal — 4 fixed under audit, 1 dissolved, 1 tolerated
- REV-056 (reconcile-sl-245, done): SPEC-027 resp. 5 + REQ-396 acceptance criterion 3 gain the render output mode — the Revision DEC-258 queued at design time
- ISS-456 — closed obsolete at reconcile (dissolved by RV-369 F-1; explicitly NOT macOS clearance)
- ISS-457 — spawn_cwd_convention.rs::bin_refs is blind inside macro invocations (RV-369 F-5 leg 2)

### Learned

- mem.fact.rustix.poll-dev-tty-macos
- mem.fact.tty.dev-tty-fstat-is-the-devnode (RV-369 F-1)
- mem.pattern.testing.injected-probes-leave-the-adapter-untested (RV-369 F-2)
- mem.fact.capsule.phase-state-does-not-ride-the-imported-diff (reconcile: coord read `phases: 0/3` on a fully audited slice)
- mem.fact.design.no-unheaded-preamble (reconcile: the scope-cut banner had to move under the sec-1 heading)
- Friction observation: design apply subject-kind vocabulary (observation f0/01a0a42c)
- Friction observations (audit): `review new` succeeds in a worktree fork while every other review verb refuses it; a fresh linked worktree cannot `cargo build` (gitignored `web/map/dist` absent)
- Friction observation (reconcile): capsule phase state is runtime tier and does not ride the imported diff (observation bc/01a0ae36)

### Open

- CHR-072 still open — RV-369 F-1 makes the code compile on macOS, it does not verify VMIN/VTIME timed reads there. Do not read ISS-456's closure as platform clearance
- PHASE-03 EX-2 pins `endpoint(…)` "over st_rdev values" and is false as written. Plan criteria are immutable-append and are not a reconcile edit surface — recorded as a design/plan divergence, behaviour preserved exactly
- MAX_IMAGE_BYTES = 8 MiB is calibrated to this corpus, not derived from any published terminal limit
- Keystrokes typed during the ≤2 s support probe are consumed (unnamed residual; see Inquiry state)
- q=2 keeps a terminal-side rejection unreportable for any cause other than size (RV-369 F-7, tolerated; sec-9 residual)
