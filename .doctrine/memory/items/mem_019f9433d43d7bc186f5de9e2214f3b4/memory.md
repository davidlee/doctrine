# Landing a dispatched slice: admit the close_target BEFORE the one integrate, or a candidate-less integrate bakes a sticky code-only trunk row

`dispatch sync --integrate --trunk main` is the ONLY committing step, and it is
**idempotent-replay over the journal**, not a fresh computation. Two ways this
bites at close:

## The failure (SL-204)

Running `--integrate --trunk main` with **no admitted `close_target`** (candidates:
0) does not refuse. It falls back to projecting the **code-class `phase/<slice>-NN`
chain** — whose commit trees deliberately omit `.doctrine/`, `install/`, `memory/`,
etc. (ADR-012 class-routed projection). So `main` fast-forwards to the phase-04
code tip and the **entire authored corpus vanishes from trunk** (SL-204: 7246 files,
−400674), yet `slice status done` still passes — the status machine is lineage-blind.
This is the "never a raw phase/*/review/* tip" case /close skill 3a warns against.

## Why a second try (admitting the close_target) does NOT fix it

The first integrate committed a **`verified` trunk row** into
`dispatch/<slice>:.doctrine/dispatch/<slice>/journal.toml` with
`planned_new_oid = <phase-04 tip>`. Integrate replays that verified row idempotently
— admitting a `close_target` afterward writes only `candidates.toml`, never
re-targets the baked row. Main goes code-only **again**, no CAS refusal.

## The recovery (order is load-bearing)

1. `git branch -f main <good-old-tip>` — undo the code-only advance (fully
   reversible; main is checked out nowhere but a scratch worktree).
2. `git branch -D review/<N> phase/<N>-*` → `dispatch sync --slice N
   --prepare-review` — **re-prepare regenerates `journal.toml` with ZERO trunk rows**
   (verified: a prepare-review journal has only review+phase `target_ref`s). This is
   the sanctioned journal reset; there is no "clear trunk row" verb.
3. `dispatch candidate create --role close_target --payload impl_bundle --base
   refs/heads/main --source refs/heads/review/N` — **`impl_bundle`, not `code`**:
   only impl_bundle layers the authored `.doctrine/` corpus onto trunk (skill 3a's
   `--payload code` template would re-strip it). 3-way merge preserves main's
   authored state.
4. `dispatch candidate admit --role close_target --candidate <ref> --review RV-NNN`
   — **before** the integrate.
5. `dispatch sync --slice N --integrate --trunk refs/heads/main` — now the clean
   journal + admitted close_target make integrate compute a fresh trunk row at the
   close_target; main lands the full bundle (code + `.doctrine/` + slice delta).
   Verify: `--show-journal-trunk-oid` == main, and `.doctrine/` file count on main
   is the full corpus, not 0.

## Prevention

A guard — integrate refusing `--trunk` (or loudly warning) when no `close_target`
is admitted, rather than silently projecting the raw phase chain — would dissolve
this. Until then: **always create+admit the `close_target` first**; treat a
candidate-less `--integrate --trunk` as a corpus-stripping footgun.

Canonical sequence: [[mem_019ec912f7fd746284bfaef00717443e]]. Related recoveries:
[[mem_019f2d1532747d11847bd7498b1f9491]] (refresh-base deadlock),
[[mem_019f06a18bf97b23bf771740e427b639]] (pre-FF trunk),
[[mem_019f6e3a108b78c1b0da16871feb9a40]] (reconcile-truth-on-edge split-lineage).
Born SL-204 close, 2026-07-24.
